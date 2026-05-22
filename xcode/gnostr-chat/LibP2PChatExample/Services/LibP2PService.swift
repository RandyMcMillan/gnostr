//
//  LibP2PService.swift
//  LibP2PChatExample
//
//  Created by Brandon Toms on 5/29/22.
//

import Foundation
import Crypto
import Combine
import LibP2P
import LibP2PNoise
import LibP2PMPLEX
import LibP2PRelay
import LibP2PAutoNAT
import LibP2PDCUtR
import LibP2PMDNS
import LibP2PKadDHT
#if os(iOS)
import UIKit
#endif

/// Any class that conforms to the ChatDelegate can register themselves on the LibP2PService to get notified of Chat events
protocol ChatDelegate {
    func on(message:String, from:PeerID)
    func on(nickname:String, from:PeerID)
}

/// We extend the Request struct with a computed var that provides access to a shared instance of our LibP2PService.
/// This allows us to interact with the LibP2PService without our Route handlers.
extension Request {
    var myService: LibP2PService { LibP2PService.shared }
}

/// We create a simple LibP2PService Singleton that is responsible for...
/// - starting and stoping libp2p
/// - configuring our libp2p networking stack
/// - registering our Route handlers
/// - listening for peer discovery events
/// - sending messages to connected peers
class LibP2PService: ObservableObject {
    static let shared = LibP2PService()

    private enum RuntimeProfile: String {
        case macOS
        case macCatalyst
        case iPad
        case iPhone
        case madeForiPad
    }

    private enum LifecycleState {
        case stopped
        case starting
        case running
        case stopping
    }

    public enum PeerConnectionState: Equatable {
        case disconnected
        case dialing
        case connected
    }

    private var app:Application
    private let peerID: PeerID
    private var lna: LocalNetworkAuthorization?
    
    internal var delegate:ChatDelegate? = nil {
        didSet { installRuntimeHandlersIfNeeded() }
    }
    
    private var pingTask:RepeatedTask? = nil
    private var runtimeHandlersInstalled = false
    private var lifecycleState: LifecycleState = .stopped
    private var topologyRegistrations: [TopologyRegistration] = []
    private var discoveredPeerAddresses: [String: Multiaddr] = [:]
    private let discoveredPeerAddressesQueue = DispatchQueue(label: "LibP2PService.discoveredPeerAddresses")
    @Published private var peerConnectionStates: [String: PeerConnectionState] = [:]
    
    public var savedPeerID:PeerID? {
        Self.loadStoredPeerID(for: Self.runtimeProfile)
    }
    
    private init() {
        self.peerID = Self.loadOrCreatePeerID(for: Self.runtimeProfile)
        self.app = Self.makeApplication(peerID: self.peerID)
        self.lna = LocalNetworkAuthorization()
        self.app.logger.notice("Resolved runtime profile: \(Self.runtimeProfile.rawValue) on port \(Self.listenPort)")
    }

    private static var runtimeProfile: RuntimeProfile {
        #if targetEnvironment(macCatalyst)
        return .macCatalyst
        #elseif os(macOS)
        return .macOS
        #elseif os(iOS)
        if ProcessInfo.processInfo.isiOSAppOnMac {
            return .madeForiPad
        }
        switch UIDevice.current.userInterfaceIdiom {
        case .pad:
            return .iPad
        case .mac:
            return .madeForiPad
        default:
            return .iPhone
        }
        #else
        return .iPhone
        #endif
    }

    private static var listenPort: Int {
        if let value = ProcessInfo.processInfo.environment["P2P_LISTEN_PORT"],
           let port = Int(value),
           port > 0 {
            return port
        }
        switch Self.runtimeProfile {
        case .macOS:
            return 10000
        case .iPhone:
            return 10001
        case .iPad:
            return 10002
        case .macCatalyst:
            return 10003
        case .madeForiPad:
            return 10004
        }
    }

    private static var peerIDStorageKey: String {
        "MyPeerID.\(Self.runtimeProfile.rawValue)"
    }

    private static func loadStoredPeerID(for profile: RuntimeProfile) -> PeerID? {
        let key = "MyPeerID.\(profile.rawValue)"
        guard let data = UserDefaults.standard.data(forKey: key) else { return nil }
        return try? PeerID(marshaledPeerID: data)
    }

    private static func loadOrCreatePeerID(for profile: RuntimeProfile) -> PeerID {
        let key = "MyPeerID.\(profile.rawValue)"
        if let saved = Self.loadStoredPeerID(for: profile) {
            return saved
        }

        let seed = Data(SHA256.hash(data: Data("gnostr-chat.peerid.\(profile.rawValue)".utf8)))
        let privateKey = try! Curve25519.Signing.PrivateKey(rawRepresentation: seed)
        let peerID = try! PeerID(marshaledPrivateKey: privateKey.marshal())
        let marshaled = try! peerID.marshal(includingPrivateKey: true)
        UserDefaults.standard.set(Data(marshaled), forKey: key)
        return peerID
    }

    private static func makeApplication(peerID: PeerID) -> Application {
        let app = Application(.testing, peerID: peerID)
        app.logger.logLevel = .notice
        // Keep connections open long enough for pubsub chatter and simulator pauses.
        app.connectionManager.setIdleTimeout(.seconds(300))
        app.security.use(.noise)
        app.muxers.use(.mplex)
        app.relay.use(.relay)
        app.autonat.use(.autonat)
        app.dcutr.use(.dcutr)
        app.discovery.use(.mdns)
        app.discovery.use(.kadDHT)
        app.servers.use(.tcp(host: "0.0.0.0", port: Self.listenPort))
        try! routes(app)
        return app
    }

    public func connectionState(for peerID: PeerID) -> PeerConnectionState {
        self.peerConnectionStates[peerID.b58String] ?? .disconnected
    }

    public func markPeerConnected(_ peerID: PeerID) {
        DispatchQueue.main.async {
            self.peerConnectionStates[peerID.b58String] = .connected
        }
    }

    public func markPeerDialing(_ peerID: PeerID) {
        DispatchQueue.main.async {
            self.peerConnectionStates[peerID.b58String] = .dialing
        }
    }

    public func markPeerDisconnected(_ peerID: PeerID) {
        DispatchQueue.main.async {
            self.peerConnectionStates[peerID.b58String] = .disconnected
        }
    }

    private func recordDiscoveredAddress(_ address: Multiaddr, for peerID: PeerID) {
        self.discoveredPeerAddressesQueue.sync {
            self.discoveredPeerAddresses[peerID.b58String] = address
        }
    }

    private func discoveredAddress(for peerID: PeerID) -> Multiaddr? {
        self.discoveredPeerAddressesQueue.sync {
            self.discoveredPeerAddresses[peerID.b58String]
        }
    }

    private func isDialableTCPAddress(_ address: Multiaddr) -> Bool {
        guard let tcp = address.tcpAddress else { return false }
        return tcp.ip4
    }

    private func dial(peerID: PeerID, address: Multiaddr) {
        self.app.logger.notice("Dialing peer \(peerID) at \(address)")
        do {
            try self.app.newStream(to: address, forProtocol: "/ipfs/id/1.0.0")
        } catch {
            self.app.logger.error("Failed to dial peer \(peerID): \(error)")
            self.markPeerDisconnected(peerID)
        }
    }

    private func redial(peerID: PeerID) {
        guard let address = self.discoveredAddress(for: peerID),
              self.isDialableTCPAddress(address) else {
            self.app.logger.warning("No stored address available for peer \(peerID.b58String); cannot redial")
            self.markPeerDisconnected(peerID)
            return
        }
        self.markPeerDialing(peerID)
        self.dial(peerID: peerID, address: address)
    }

    private func installRuntimeHandlersIfNeeded() {
        guard !self.runtimeHandlersInstalled, let delegate = self.delegate else { return }

        self.app.discovery.onPeerDiscovered(self.app) { peer in
            self.app.logger.notice("We discovered a peer: \(peer)")
            self.app.connections.getConnectionsToPeer(peer: peer.peer, on: nil).whenSuccess { conns in
                if conns.isEmpty {
                    guard let address = peer.addresses.first(where: { self.isDialableTCPAddress($0) }) else {
                        self.app.logger.warning("No dialable IPv4 TCP address found for peer \(peer.peer)")
                        return
                    }
                    self.recordDiscoveredAddress(address, for: peer.peer)
                    self.markPeerDialing(peer.peer)
                    self.dial(peerID: peer.peer, address: address)
                } else {
                    self.markPeerConnected(peer.peer)
                }
            }
        }

        self.app.events.on(self, event: .disconnected({ _, peerID in
            guard let peerID = peerID else { return }
            self.app.logger.notice("Disconnected from peer \(peerID.b58String); scheduling redial")
            self.markPeerDialing(peerID)
            self.app.eventLoopGroup.any().scheduleTask(in: .seconds(1)) {
                self.redial(peerID: peerID)
            }
        }))

        self.pingTask?.cancel()
        self.pingTask = self.app.eventLoopGroup.any().scheduleRepeatedTask(initialDelay: .seconds(15), delay: .seconds(15), { _ in
            self.pingDiscoveredUsers()
        })

        self.runtimeHandlersInstalled = true
        self.app.logger.notice("Installed runtime handlers for delegate \(String(describing: delegate))")
    }
    
    public func deletePeerID() {
        UserDefaults.standard.removeObject(forKey: Self.peerIDStorageKey)
        UserDefaults.standard.removeObject(forKey: "MyPeerID")
    }
    
    public func register(register:AnyObject, event: EventBus.EventHandler) {
        self.app.events.on(register, event: event)
    }
    
    public func topology(_ reg:TopologyRegistration) {
        self.topologyRegistrations.append(reg)
        self.app.topology.register(reg)
    }

    private func reinstallTopologyRegistrations() {
        for registration in self.topologyRegistrations {
            self.app.topology.register(registration)
        }
    }
    
    public func start() async throws {
        guard self.lifecycleState == .stopped else { return }
        self.lifecycleState = .starting
        guard await self.lna?.requestAuthorization() ?? true else {
            self.lifecycleState = .stopped
            throw CocoaError(.userCancelled)
        }
        if self.app.didShutdown {
            self.app = Self.makeApplication(peerID: self.peerID)
            self.lna = LocalNetworkAuthorization()
            self.runtimeHandlersInstalled = false
            self.reinstallTopologyRegistrations()
        }
        self.installRuntimeHandlersIfNeeded()
        do {
            try app.start()
            self.app.logger.notice("LibP2P Started!")
            self.lifecycleState = .running
        } catch {
            self.lifecycleState = .stopped
            throw error
        }
    }
    
    public func stop() {
        guard self.lifecycleState == .running else { return }
        self.lifecycleState = .stopping
        self.pingTask?.cancel()
        app.shutdown()
        self.runtimeHandlersInstalled = false
        self.lifecycleState = .stopped
    }
    
    public func send(message:String, to peer:PeerID) {
        guard self.app.isRunning else { print("LibP2P needs to be running in order to send messages!"); return }
        // There's a lot happening in this `newRequest` call, let's break it down
        // We have some data (our `message`) that we would like to send to our `peer`
        // Libp2p offers a `Request` type that makes sending a single chunk of data easier than opening up and managing a streaming channel (similar to an HTTP Request)
        // So we create a `newRequest` to our `peer`, destined for the `/chat/1.0.0` protocol, with the `message` we'd like to send them
        //
        // The next couple params are a little more in depth...
        //  `style` provides the Request with a hint at what kind of behavior to expect.
        //      `.noResponseExpected` means the stream will imediately request to be closed after sending the data, not waiting for a response / reply. (like a PUT request)
        //      `.responseExpected` means that we expect data back from the peer (like a GET request)
        //     Because our `/chat/1.0.0` doesn't support read reciepts we set this to `.noResponseExpected`
        //     If, let's say, `/chat/2.0.0` supported delivery confirmations (read receipts), we could change this to `.responseExpected` and parse the returned message for confirmation of delivery.
        //  `withHandlers` let's us configure the `/chat/1.0.0` stream with custom Channel Handlers (similar to middleware if you're familiar with other server side frameworks).
        //     When we registered our `/chat/1.0.0` route earlier (in our initiailizer) we told Libp2p that the `/chat/1.0.0` should be `.newLineDelimited` (Routes.swift).
        //     Therefor, when set to `.inherit`, libp2p can automagically use this info to configure the `/chat/1.0.0` stream with the same channel handlers.
        //     If, for some reason, you wanted to have a unique channel handler configuration for this particular requets, you can add any ChannelHandlers you'd like to here.
        //       ex: perhaps adding additional Logging handlers if you were trying to debug a request
        self.app.newRequest(to: peer, forProtocol: "/chat/1.0.0", withRequest: Data(message.utf8), style: .noResponseExpected, withHandlers: .inherit).whenComplete { result in
            switch result {
            case .failure(let error):
                self.app.logger.error("Error: \(error)")
            case .success:
                self.app.logger.trace("Sent message to peer: \(peer)")
            }
        }
    }
    
    public func isConnectedTo(peer:PeerID) async -> Bool {
        await withCheckedContinuation { continuation in
            self.app.connections.connectedness(peer: peer, on: nil).whenComplete { result in
                switch result {
                case .failure(_):
                    return continuation.resume(returning: false)
                case .success(let connectedness):
                    return continuation.resume(returning: connectedness == .Connected)
                }
            }
        }
    }
    
    /// This recurring task acts as a Keep-Alive service for Peers that support the `/chat/1.0.0` protocol.
    /// It keeps direct chat connections from idling out while pubsub traffic is sparse.
    public func pingDiscoveredUsers() {
        let _ = self.app.peers.getPeers(supportingProtocol: .init("chat/1.0.0")! ).map { peers in
            return peers.compactMap { peerID in
                self.app.connections.connectedness(peer: try! PeerID(cid: peerID), on: nil).map { connectedness -> (String, EventLoopFuture<TimeAmount>)? in
                    switch connectedness {
                    case .Connected:
                        return (peerID, self.app.identify.ping(peer: try! PeerID(cid: peerID)).always { result in
                            self.app.logger.debug("Ping Result: \(result)")
                        })
                    default:
                        return nil
                    }
                }
            }.flatten(on: self.app.eventLoopGroup.any())
        }
    }
}

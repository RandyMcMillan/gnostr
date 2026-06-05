import Foundation
import LibP2P

extension String {
    static func key(_ key: String) -> String { return NSLocalizedString(key, comment: "") }
}

enum State {
    case loading
    case ready
    case packed
    case create
    case first
}

public struct GitPeerEnvelope: Codable {
    public enum Kind: String, Codable {
        case hello
        case status
        case refs
        case fetch
        case pack
        case push
        case error
    }

    public var kind: Kind
    public var repository: String?
    public var payload: String?
    public var peer: String?
    public var timestamp: Date

    public init(
        kind: Kind,
        repository: String? = nil,
        payload: String? = nil,
        peer: String? = nil,
        timestamp: Date = .init()
    ) {
        self.kind = kind
        self.repository = repository
        self.payload = payload
        self.peer = peer
        self.timestamp = timestamp
    }
}

public protocol GitPeerServiceDelegate: AnyObject {
    func gitPeerService(_ service: GitPeerService, didDiscover peer: PeerID)
    func gitPeerService(_ service: GitPeerService, didLose peer: PeerID)
    func gitPeerService(_ service: GitPeerService, didReceive envelope: GitPeerEnvelope, from peer: PeerID)
}

public extension GitPeerServiceDelegate {
    func gitPeerService(_ service: GitPeerService, didDiscover peer: PeerID) {}
    func gitPeerService(_ service: GitPeerService, didLose peer: PeerID) {}
    func gitPeerService(_ service: GitPeerService, didReceive envelope: GitPeerEnvelope, from peer: PeerID) {}
}

public final class GitPeerService {
    public static let shared = GitPeerService()

    private enum LifecycleState {
        case stopped
        case starting
        case running
        case stopping
    }

    public enum ServiceError: Error {
        case notRunning
        case peerIdentityUnavailable
        case unableToEncodeEnvelope
    }

    private static let peerStorageKey = "GitPeerService.peerID"
    private static let protocolName = "/git/1.0.0"

    public weak var delegate: GitPeerServiceDelegate?
    private var app: Application?
    private var state: LifecycleState = .stopped

    public var isRunning: Bool { self.state == .running }
    public var peerID: PeerID? { self.app?.peerID }

    private init() {}

    public func start() async throws {
        guard self.state == .stopped else { return }
        self.state = .starting
        do {
            let peerID = try self.loadPeerID()
            let app = Application(.detect(), peerID: peerID)
            app.logger.logLevel = .info
            app.security.use(.noise)
            app.muxers.use(.yamux)
            app.discovery.use(.mdns)
            app.servers.use(.tcp(host: "0.0.0.0", port: 0))
            self.installRoutes(on: app)
            self.installDiscovery(on: app)
            try await app.startup()
            self.app = app
            self.state = .running
            app.logger.notice("Git peer service started as \(peerID.shortDescription)")
        } catch {
            self.state = .stopped
            throw error
        }
    }

    public func stop() async {
        guard self.state == .running, let app = self.app else { return }
        self.state = .stopping
        await app.asyncShutdown()
        self.app = nil
        self.state = .stopped
    }

    public func announce(repository url: URL) throws {
        guard let app = self.app, self.state == .running else { throw ServiceError.notRunning }
        let envelope = GitPeerEnvelope(kind: .status, repository: url.lastPathComponent, payload: "available")
        guard let data = try? JSONEncoder().encode(envelope) else { throw ServiceError.unableToEncodeEnvelope }
        self.broadcast(data, using: app)
    }

    public func send(_ envelope: GitPeerEnvelope, to peer: PeerID) throws {
        guard let app = self.app, self.state == .running else { throw ServiceError.notRunning }
        guard let data = try? JSONEncoder().encode(envelope) else { throw ServiceError.unableToEncodeEnvelope }
        app.newRequest(
            to: peer,
            forProtocol: Self.protocolName,
            withRequest: data,
            style: .noResponseExpected,
            withHandlers: .inherit
        ).whenComplete { result in
            switch result {
            case .failure(let error):
                app.logger.error("Failed to send git peer envelope to \(peer): \(error)")
            case .success:
                app.logger.trace("Sent git peer envelope to \(peer)")
            }
        }
    }

    private func broadcast(_ data: Data, using app: Application) {
        guard self.state == .running else { return }
        app.logger.notice("Announcing repository presence to connected peers")
        app.peers.getPeers(supportingProtocol: .init(Self.protocolName)!).map { peers in
            peers.compactMap { peerID in
                try? PeerID(cid: peerID)
            }.forEach {
                self.sendData(data, to: $0, using: app)
            }
        }
    }

    private func sendData(_ data: Data, to peer: PeerID, using app: Application) {
        app.newRequest(
            to: peer,
            forProtocol: Self.protocolName,
            withRequest: data,
            style: .noResponseExpected,
            withHandlers: .inherit
        ).whenComplete { result in
            switch result {
            case .failure(let error):
                app.logger.error("Failed to announce to \(peer): \(error)")
            case .success:
                app.logger.trace("Announced to \(peer)")
            }
        }
    }

    private func loadPeerID() throws -> PeerID {
        if let data = UserDefaults.standard.data(forKey: Self.peerStorageKey) {
            return try PeerID(marshaledPrivateKey: data)
        }
        let peerID = try PeerID(.Ed25519)
        UserDefaults.standard.set(try peerID.marshalPrivateKey(), forKey: Self.peerStorageKey)
        return peerID
    }

    private func installRoutes(on app: Application) {
        app.group("git") { routes in
            routes.on("1.0.0", handlers: [.newLineDelimited]) { [weak self] req -> Response<String> in
                guard let peer = req.remotePeer else {
                    req.logger.warning("Unidentified git peer request")
                    return .close
                }
                switch req.event {
                case .ready:
                    return .stayOpen
                case .data(let payload):
                    if let str = String(data: Data(payload.readableBytesView), encoding: .utf8),
                       let data = str.data(using: .utf8),
                       let envelope = try? JSONDecoder().decode(GitPeerEnvelope.self, from: data) {
                        self?.delegate?.gitPeerService(self ?? GitPeerService.shared, didReceive: envelope, from: peer)
                    } else if let str = String(data: Data(payload.readableBytesView), encoding: .utf8) {
                        let envelope = GitPeerEnvelope(kind: .error, payload: str, peer: peer.shortDescription)
                        self?.delegate?.gitPeerService(self ?? GitPeerService.shared, didReceive: envelope, from: peer)
                    }
                    return .stayOpen
                case .closed:
                    return .close
                case .error(let error):
                    req.logger.error("Git route error: \(error)")
                    return .close
                }
            }
        }
    }

    private func installDiscovery(on app: Application) {
        app.discovery.onPeerDiscovered(app) { [weak self] peer in
            guard let self else { return }
            app.connections.getConnectionsToPeer(peer: peer.peer, on: nil).whenSuccess { conns in
                guard conns.isEmpty else { return }
                guard let address = peer.addresses.first(where: { $0.description.contains("/tcp/") }) else {
                    app.logger.warning("No dialable TCP address for peer \(peer.peer)")
                    return
                }
                do {
                    try app.newStream(to: address, forProtocol: Self.protocolName)
                } catch {
                    app.logger.error("Failed to dial peer \(peer.peer): \(error)")
                }
            }
        }

        app.topology.register(
            TopologyRegistration(
                protocol: Self.protocolName,
                handler: TopologyHandler(
                    onConnect: { [weak self] peer, _ in
                        self?.delegate?.gitPeerService(self ?? GitPeerService.shared, didDiscover: peer)
                    },
                    onDisconnect: { [weak self] peer in
                        self?.delegate?.gitPeerService(self ?? GitPeerService.shared, didLose: peer)
                    }
                )
            )
        )
    }
}

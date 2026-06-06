import Foundation
import Git
import LibP2P
import LibP2PNoise
import LibP2PYAMUX
import LibP2PMDNS

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

public struct GitPeerRepositoryAdvertisement: Codable {
    public let name: String
    public let path: String
    public let peer: String
    public let timestamp: Date

    public init(url: URL, peer: String, timestamp: Date = .init()) {
        self.name = url.lastPathComponent
        self.path = url.path
        self.peer = peer
        self.timestamp = timestamp
    }
}

public struct GitPeerRepositoryStatus: Codable {
    public let repository: String
    public let branch: String
    public let remote: String
    public let peer: String
    public let peerCount: Int
    public let timestamp: Date

    public init(
        repository: String,
        branch: String,
        remote: String,
        peer: String,
        peerCount: Int,
        timestamp: Date = .init()
    ) {
        self.repository = repository
        self.branch = branch
        self.remote = remote
        self.peer = peer
        self.peerCount = peerCount
        self.timestamp = timestamp
    }
}

public struct GitPeerRepositoryRefs: Codable {
    public let repository: String
    public let branch: String
    public let reference: String
    public let commit: String
    public let remote: String
    public let peer: String
    public let timestamp: Date

    public init(
        repository: String,
        branch: String,
        reference: String,
        commit: String,
        remote: String,
        peer: String,
        timestamp: Date = .init()
    ) {
        self.repository = repository
        self.branch = branch
        self.reference = reference
        self.commit = commit
        self.remote = remote
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

    public struct Status {
        public let isRunning: Bool
        public let peerCount: Int
        public let hasAdvertisement: Bool
        public let peerID: String?
    }

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
    private var peers: [String: PeerID] = [:]
    private var advertisement: GitPeerRepositoryAdvertisement?
    private var repositoryURL: URL?
    private var peerStatuses: [String: GitPeerRepositoryStatus] = [:]
    private var peerRefs: [String: GitPeerRepositoryRefs] = [:]

    public var isRunning: Bool { self.state == .running }
    public var peerID: PeerID? { self.app?.peerID }
    public var status: Status {
        Status(
            isRunning: self.isRunning,
            peerCount: self.peers.count,
            hasAdvertisement: self.advertisement != nil,
            peerID: self.peerID?.b58String
        )
    }
    public var connectedPeers: [PeerID] { Array(self.peers.values) }
    public var knownPeerStatuses: [GitPeerRepositoryStatus] { Array(self.peerStatuses.values) }
    public var knownPeerRefs: [GitPeerRepositoryRefs] { Array(self.peerRefs.values) }

    private init() {}

    public func start() async throws {
        guard self.state == .stopped else { return }
        self.state = .starting
        do {
            let peerID = try self.loadPeerID()
            let app = try await Application.make(.detect(), peerID: peerID)
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
        try? await app.asyncShutdown()
        self.app = nil
        self.state = .stopped
    }

    public func announce(repository url: URL) throws {
        guard let app = self.app, self.state == .running else { throw ServiceError.notRunning }
        let peerID = app.peerID

        self.repositoryURL = url
        let advertisement = GitPeerRepositoryAdvertisement(url: url, peer: peerID.shortDescription)
        self.advertisement = advertisement

        guard let payload = try? JSONEncoder().encode(advertisement) else { throw ServiceError.unableToEncodeEnvelope }
        let envelope = GitPeerEnvelope(
            kind: .hello,
            repository: advertisement.name,
            payload: String(decoding: payload, as: UTF8.self),
            peer: advertisement.peer
        )

        app.logger.notice("Advertising repository \(advertisement.name) to \(self.peers.count) peer(s)")
        for peer in self.peers.values {
            try? self.send(envelope, to: peer)
            try? self.sendStatus(to: peer)
            try? self.requestRefs(from: peer)
        }
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

    public func sendStatus(to peer: PeerID) throws {
        guard let envelope = self.statusEnvelope else { return }
        try self.send(envelope, to: peer)
    }

    public func requestRefs(from peer: PeerID) throws {
        guard let app = self.app, self.state == .running else { throw ServiceError.notRunning }
        let request = GitPeerEnvelope(kind: .refs, repository: self.advertisement?.name, peer: self.peerID?.shortDescription)
        guard let data = try? JSONEncoder().encode(request) else { throw ServiceError.unableToEncodeEnvelope }
        app.newRequest(
            to: peer,
            forProtocol: Self.protocolName,
            withRequest: data,
            style: .responseExpected,
            withHandlers: .inherit
        ).whenComplete { result in
            switch result {
            case .failure(let error):
                app.logger.error("Failed to request git peer refs from \(peer): \(error)")
            case .success(let response):
                guard let str = String(data: response, encoding: .utf8),
                      let data = str.data(using: .utf8),
                      let refs = try? JSONDecoder().decode(GitPeerRepositoryRefs.self, from: data) else {
                    app.logger.error("Failed to decode git peer refs from \(peer)")
                    return
                }
                self.peerRefs[peer.b58String] = refs
                app.logger.trace("Received git peer refs from \(peer)")
            }
        }
    }

    private var statusEnvelope: GitPeerEnvelope? {
        guard let peerID = self.peerID, let repositoryURL = self.repositoryURL else { return nil }
        let branch = Hub.branch(repositoryURL) ?? "unknown"
        let remote = Hub.remote(repositoryURL)
        let payload = GitPeerRepositoryStatus(
            repository: repositoryURL.lastPathComponent,
            branch: branch,
            remote: remote,
            peer: peerID.shortDescription,
            peerCount: self.peers.count
        )
        guard let data = try? JSONEncoder().encode(payload) else { return nil }
        return GitPeerEnvelope(
            kind: .status,
            repository: payload.repository,
            payload: String(decoding: data, as: UTF8.self),
            peer: payload.peer
        )
    }

    private var refsEnvelope: GitPeerEnvelope? {
        guard let peerID = self.peerID, let repositoryURL = self.repositoryURL else { return nil }
        let branch = Hub.branch(repositoryURL) ?? "unknown"
        let reference = Hub.reference(repositoryURL) ?? branch
        let commit = Hub.id(repositoryURL) ?? ""
        let remote = Hub.remote(repositoryURL)
        let payload = GitPeerRepositoryRefs(
            repository: repositoryURL.lastPathComponent,
            branch: branch,
            reference: reference,
            commit: commit,
            remote: remote,
            peer: peerID.shortDescription
        )
        guard let data = try? JSONEncoder().encode(payload) else { return nil }
        return GitPeerEnvelope(
            kind: .refs,
            repository: payload.repository,
            payload: String(decoding: data, as: UTF8.self),
            peer: payload.peer
        )
    }
    private func loadPeerID() throws -> PeerID {
        if let data = UserDefaults.standard.data(forKey: Self.peerStorageKey) {
            return try PeerID(marshaledPrivateKey: data)
        }
        let peerID = try PeerID(.Ed25519)
        UserDefaults.standard.set(Data(try peerID.marshalPrivateKey()), forKey: Self.peerStorageKey)
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
                        if envelope.kind == .hello {
                            try? self?.sendStatus(to: peer)
                            try? self?.requestRefs(from: peer)
                        } else if envelope.kind == .refs {
                            if let refs = self?.refsEnvelope?.payload {
                                return .respondThenClose(refs)
                            }
                        }
                        if envelope.kind == .refs,
                           let refs = envelope.payload?.data(using: .utf8).flatMap({ try? JSONDecoder().decode(GitPeerRepositoryRefs.self, from: $0) }) {
                            self?.peerRefs[peer.b58String] = refs
                        } else if envelope.kind == .status,
                            let status = envelope.payload?.data(using: .utf8).flatMap({ try? JSONDecoder().decode(GitPeerRepositoryStatus.self, from: $0) }) {
                            self?.peerStatuses[peer.b58String] = status
                        }
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
        app.discovery.onPeerDiscovered(app) { peer in
            app.connections.getConnectionsToPeer(peer: peer.peer, on: nil).whenSuccess { conns in
                guard conns.isEmpty else { return }
                self.peers[peer.peer.b58String] = peer.peer
                if let advertisement = self.advertisement, let payload = try? JSONEncoder().encode(advertisement) {
                    let envelope = GitPeerEnvelope(
                        kind: .hello,
                        repository: advertisement.name,
                        payload: String(decoding: payload, as: UTF8.self),
                        peer: advertisement.peer
                    )
                    try? self.send(envelope, to: peer.peer)
                    try? self.sendStatus(to: peer.peer)
                }
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
                        self?.peers[peer.b58String] = peer
                        self?.delegate?.gitPeerService(self ?? GitPeerService.shared, didDiscover: peer)
                        if let advertisement = self?.advertisement, let payload = try? JSONEncoder().encode(advertisement) {
                            let envelope = GitPeerEnvelope(
                                kind: .hello,
                                repository: advertisement.name,
                                payload: String(decoding: payload, as: UTF8.self),
                                peer: advertisement.peer
                            )
                            try? self?.send(envelope, to: peer)
                            try? self?.sendStatus(to: peer)
                            try? self?.requestRefs(from: peer)
                        }
                    },
                    onDisconnect: { [weak self] peer in
                        self?.peers.removeValue(forKey: peer.b58String)
                        self?.peerStatuses.removeValue(forKey: peer.b58String)
                        self?.delegate?.gitPeerService(self ?? GitPeerService.shared, didLose: peer)
                    }
                )
            )
        )
    }
}

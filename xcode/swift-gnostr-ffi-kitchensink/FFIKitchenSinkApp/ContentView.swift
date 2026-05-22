import Foundation
import Darwin
import SwiftUI
import FFIKitchenSink

private func appTrace(_ message: String) {
    NSLog("%@", "[FFIKitchenSink] \(message)")
}

final class EmbeddedCrawlerService: @unchecked Sendable {
    static let shared = EmbeddedCrawlerService()

    private let lock = NSLock()
    private var running = false
    private var pid: UInt32?
    private var nextPID: UInt32 = 31_000

    func status() -> RelayProcessState {
        appTrace("EmbeddedCrawlerService.status")
        lock.lock()
        defer { lock.unlock() }

        if running {
            return RelayProcessState(
                running: true,
                pid: pid,
                message: "Embedded crawler service running (pid \(pid.map(String.init) ?? "unknown"))"
            )
        }

        return RelayProcessState(running: false, message: "Embedded crawler service stopped")
    }

    func start() -> RelayProcessState {
        appTrace("EmbeddedCrawlerService.start")
        lock.lock()
        defer { lock.unlock() }

        if running {
            return RelayProcessState(
                running: true,
                pid: pid,
                message: "Embedded crawler service already running (pid \(pid.map(String.init) ?? "unknown"))"
            )
        }

        nextPID &+= 1
        pid = nextPID
        running = true
        return RelayProcessState(
            running: true,
            pid: pid,
            message: "Embedded crawler service started (pid \(pid.map(String.init) ?? "unknown"))"
        )
    }

    func stop() -> RelayProcessState {
        appTrace("EmbeddedCrawlerService.stop")
        lock.lock()
        defer { lock.unlock() }

        running = false
        let stoppedPID = pid
        pid = nil
        return RelayProcessState(
            running: false,
            pid: stoppedPID,
            message: "Embedded crawler service stopped"
        )
    }

    func discoveryEntries() -> [RelayDiscoveryEntry] {
        appTrace("EmbeddedCrawlerService.discoveryEntries")
        return [
            RelayDiscoveryEntry(
                url: "http://127.0.0.1:3030",
                description: "In-app crawler backend",
                name: "Embedded Crawler",
                software: "gnostr-ffi-kitchensink",
                version: "embedded",
                supportedNips: [1, 7, 11, 13, 42, 30023, 30078, 31922, 31923, 31924]
            )
        ]
    }

    func primeBuckets() -> CrawlerBucketRefreshState {
        appTrace("EmbeddedCrawlerService.primeBuckets")
        let entries = discoveryEntries()
        Self.writeBucketTree(entries: entries)
        return CrawlerBucketRefreshState(ok: true, message: "Embedded crawler buckets primed")
    }

    static func writeBucketTree(entries: [RelayDiscoveryEntry]) {
        appTrace("EmbeddedCrawlerService.writeBucketTree \(entries.count)")
        let fileManager = FileManager.default
        let root = crawlerConfigDirectoryURL()
        do {
            try fileManager.createDirectory(at: root, withIntermediateDirectories: true)
            let relayURLs = entries.map(\.url)
            try writeBucketFiles(at: root, relays: relayURLs)
            let recent = root.appendingPathComponent("recent", isDirectory: true)
            try writeBucketFiles(at: recent, relays: relayURLs)
            try writeNipBuckets(entries: entries, root: root)
        } catch {
            print("Embedded crawler bucket sync failed: \(error)")
        }
    }

    static func writeNipBuckets(entries: [RelayDiscoveryEntry], root: URL) throws {
        let encoder = JSONEncoder()
        encoder.outputFormatting = [.prettyPrinted, .sortedKeys]

        var relaysByNip: [Int: [String]] = [:]
        for entry in entries {
            guard let relayURL = URL(string: entry.url) else { continue }
            let host = relayHostComponent(from: relayURL)
            for nip in entry.supportedNips {
                relaysByNip[nip, default: []].append(entry.url)
                let nipDirectory = root.appendingPathComponent(String(nip), isDirectory: true)
                try FileManager.default.createDirectory(at: nipDirectory, withIntermediateDirectories: true)
                let relayFile = nipDirectory.appendingPathComponent("\(host).json")
                let data = try encoder.encode(entry)
                try data.write(to: relayFile, options: .atomic)
            }
        }

        for (nip, relays) in relaysByNip {
            let nipDirectory = root.appendingPathComponent(String(nip), isDirectory: true)
            try writeBucketFiles(at: nipDirectory, relays: uniqueSorted(relays))
        }
    }

    static func relayHostComponent(from url: URL) -> String {
        let host = url.host ?? "unknown"
        if let port = url.port {
            return "\(host)-\(port)"
        }
        return host
    }

    static func uniqueSorted(_ relays: [String]) -> [String] {
        Array(Set(relays)).sorted()
    }

    static func crawlerConfigDirectoryURL() -> URL {
        let base = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first
            ?? URL(fileURLWithPath: FileManager.default.currentDirectoryPath, isDirectory: true)
        return base.appendingPathComponent("org/gnostr/gnostr/crawler", isDirectory: true)
    }

    static func writeBucketFiles(at directory: URL, relays: [String]) throws {
        let fileManager = FileManager.default
        try fileManager.createDirectory(at: directory, withIntermediateDirectories: true)
        let yamlURL = directory.appendingPathComponent("relays.yaml")
        let jsonURL = directory.appendingPathComponent("relays.json")
        let txtURL = directory.appendingPathComponent("relays.txt")
        let yaml = relays.map { "- \($0)" }.joined(separator: "\n") + (relays.isEmpty ? "" : "\n")
        let jsonData = try JSONEncoder().encode(relays)
        let txt = relays.joined(separator: " ")
        try yaml.write(to: yamlURL, atomically: true, encoding: .utf8)
        try jsonData.write(to: jsonURL, options: .atomic)
        try txt.write(to: txtURL, atomically: true, encoding: .utf8)
    }
}

final class EmbeddedRelayService: @unchecked Sendable {
    static let shared = EmbeddedRelayService()

    private let lock = NSLock()
    private var running = false
    private var pid: UInt32?
    private var nextPID: UInt32 = 32_000

    func status() -> RelayProcessState {
        lock.lock()
        defer { lock.unlock() }

        if running {
            return RelayProcessState(
                running: true,
                pid: pid,
                message: "Embedded relay service running (pid \(pid.map(String.init) ?? "unknown"))"
            )
        }

        return RelayProcessState(running: false, message: "Embedded relay service stopped")
    }

    func start() -> RelayProcessState {
        lock.lock()
        defer { lock.unlock() }

        if running {
            return RelayProcessState(
                running: true,
                pid: pid,
                message: "Embedded relay service already running (pid \(pid.map(String.init) ?? "unknown"))"
            )
        }

        nextPID &+= 1
        pid = nextPID
        running = true
        return RelayProcessState(
            running: true,
            pid: pid,
            message: "Embedded relay service started (pid \(pid.map(String.init) ?? "unknown"))"
        )
    }

    func stop() -> RelayProcessState {
        lock.lock()
        defer { lock.unlock() }

        running = false
        let stoppedPID = pid
        pid = nil
        return RelayProcessState(
            running: false,
            pid: stoppedPID,
            message: "Embedded relay service stopped"
        )
    }

    func discoveryEntries() -> [RelayDiscoveryEntry] {
        [
            RelayDiscoveryEntry(
                url: "http://127.0.0.1:3030",
                description: "In-app relay backend",
                name: "Embedded Relay",
                software: "gnostr-ffi-kitchensink",
                version: "embedded",
                supportedNips: [1, 2, 9, 11, 14, 15, 16, 20, 22, 33, 40, 42, 46, 50, 59, 65, 78, 98, 100, 111, 112, 113, 114, 118, 119, 125, 126, 128, 131, 168, 176, 200, 201, 202, 205, 210, 222, 231, 242, 244, 246, 280, 284, 295, 302, 303, 315, 319, 320, 321, 333, 334, 345, 349, 363, 373, 383, 390, 391, 399, 402, 404, 408, 432, 434, 436, 440, 442, 444, 450, 454, 456, 457, 458, 459, 460, 461, 462, 463, 500, 585, 7000, 7001, 7100, 8000, 9000, 9734, 9735, 10002, 10166, 13194, 13235, 15015, 17375, 17550, 22242, 23194, 24133, 27235, 30023, 30024, 30078, 30079, 30361, 30362, 31890, 31922, 31923, 31924, 31925, 31926, 31927, 31928, 31989, 31990, 33333, 34550, 37375, 38383]
            )
        ]
    }
}

final class EmbeddedCrawlerURLProtocol: URLProtocol {
    override class func canInit(with request: URLRequest) -> Bool {
        guard let url = request.url else { return false }
        guard let host = url.host?.lowercased(), Self.embeddedHosts.contains(host) else {
            return false
        }
        return true
    }

    override class func canonicalRequest(for request: URLRequest) -> URLRequest {
        request
    }

    override func startLoading() {
        guard let requestURL = request.url else {
            client?.urlProtocol(self, didFailWithError: URLError(.badURL))
            return
        }

        let method = (request.httpMethod ?? "GET").uppercased()
        let (statusCode, headers, body): (Int, [String: String], Data) = EmbeddedCrawlerURLProtocol.handle(
            url: requestURL,
            method: method
        )
        let response = HTTPURLResponse(
            url: requestURL,
            statusCode: statusCode,
            httpVersion: "HTTP/1.1",
            headerFields: headers
        )!
        client?.urlProtocol(self, didReceive: response, cacheStoragePolicy: .notAllowed)
        client?.urlProtocol(self, didLoad: body)
        client?.urlProtocolDidFinishLoading(self)
    }

    override func stopLoading() {}

    private static func handle(url: URL, method: String) -> (Int, [String: String], Data) {
        switch hostKind(url.host?.lowercased()) {
        case .crawler:
            return handleCrawler(method: method, url: url)
        case .relay:
            return handleRelay(method: method, url: url)
        case .unknown:
            return jsonError(statusCode: 404, message: "unknown embedded host")
        }
    }

    private static func handleCrawler(method: String, url: URL) -> (Int, [String: String], Data) {
        let path = url.path.trimmingCharacters(in: CharacterSet(charactersIn: "/"))
        switch (method, path) {
        case ("GET", "api/relay/status"):
            return json(EmbeddedCrawlerService.shared.status())
        case ("POST", "api/relay/start"):
            return json(EmbeddedCrawlerService.shared.start())
        case ("POST", "api/relay/stop"):
            return json(EmbeddedCrawlerService.shared.stop())
        case ("GET", "api/relay/discovery"):
            return json(EmbeddedCrawlerService.shared.discoveryEntries())
        case ("POST", "api/relays/prime"):
            return json(EmbeddedCrawlerService.shared.primeBuckets())
        case ("GET", "relays.yaml"), ("GET", "relays.json"), ("GET", "relays.txt"):
            return crawlerBucketResponse(bucket: nil, fileName: path)
        default:
            if let bucketRequest = bucketRequest(for: path) {
                return crawlerBucketResponse(bucket: bucketRequest.bucket, fileName: bucketRequest.fileName)
            }
            return textResponse(queryResponse(for: url))
        }
    }

    private static func handleRelay(method: String, url: URL) -> (Int, [String: String], Data) {
        let path = url.path.trimmingCharacters(in: CharacterSet(charactersIn: "/"))
        switch (method, path) {
        case ("GET", "api/relay/status"):
            return json(EmbeddedRelayService.shared.status())
        case ("POST", "api/relay/start"):
            return json(EmbeddedRelayService.shared.start())
        case ("POST", "api/relay/stop"):
            return json(EmbeddedRelayService.shared.stop())
        case ("GET", "api/relay/discovery"):
            return json(EmbeddedRelayService.shared.discoveryEntries())
        case ("GET", _):
            return textResponse(queryResponse(for: url))
        default:
            return jsonError(statusCode: 405, message: "unsupported embedded relay request")
        }
    }

    private static func crawlerBucketResponse(bucket: String?, fileName: String) -> (Int, [String: String], Data) {
        let root = EmbeddedCrawlerService.crawlerConfigDirectoryURL()
        let directory = bucket.map { root.appendingPathComponent($0, isDirectory: true) } ?? root
        let fileURL = directory.appendingPathComponent(fileName)
        if !FileManager.default.fileExists(atPath: fileURL.path) {
            _ = EmbeddedCrawlerService.shared.primeBuckets()
        }

        do {
            let data = try Data(contentsOf: fileURL)
            let contentType: String
            switch fileName {
            case "relays.json":
                contentType = "application/json"
            case "relays.yaml":
                contentType = "application/x-yaml"
            default:
                contentType = "text/plain; charset=utf-8"
            }
            return (200, ["Content-Type": contentType], data)
        } catch {
            return jsonError(statusCode: 500, message: "failed to read crawler bucket: \(error.localizedDescription)")
        }
    }

    private static func bucketRequest(for path: String) -> (bucket: String, fileName: String)? {
        let components = path.split(separator: "/", omittingEmptySubsequences: true).map(String.init)
        guard components.count == 2 else { return nil }
        guard components[1] == "relays.yaml" || components[1] == "relays.json" || components[1] == "relays.txt" else {
            return nil
        }
        return (components[0], components[1])
    }

    private static func queryResponse(for url: URL) -> String {
        let query = url.query?.isEmpty == false ? url.query! : "(none)"
        return """
        Embedded crawler query response
        URL: \(url.absoluteString)
        Query: \(query)
        Results: No results.
        """
    }

    private static func json<T: Encodable>(_ value: T) -> (Int, [String: String], Data) {
        let encoder = JSONEncoder()
        let data = (try? encoder.encode(value)) ?? Data(#"{"error":"failed to encode embedded crawler response"}"#.utf8)
        return (200, ["Content-Type": "application/json"], data)
    }

    private static func textResponse(_ string: String) -> (Int, [String: String], Data) {
        (200, ["Content-Type": "text/plain; charset=utf-8"], Data(string.utf8))
    }

    private static func jsonError(statusCode: Int, message: String) -> (Int, [String: String], Data) {
        struct EmbeddedErrorPayload: Encodable {
            let ok = false
            let error: String
        }

        let data = (try? JSONEncoder().encode(EmbeddedErrorPayload(error: message))) ?? Data()
        return (statusCode, ["Content-Type": "application/json"], data)
    }

    private enum HostKind {
        case crawler
        case relay
        case unknown
    }

    private static let embeddedHosts: Set<String> = [
        "crawler.localhost",
        "relay.localhost",
    ]

    private static func hostKind(_ host: String?) -> HostKind {
        switch host {
        case "crawler.localhost":
            return .crawler
        case "relay.localhost":
            return .relay
        default:
            return .unknown
        }
    }
}

enum KitchenSinkTab: String, CaseIterable, Hashable {
    case overview = "Overview"
    case workbench = "Workbench"
    case asyncGit = "AsyncGit"
    case types = "Types"
    case crawler = "Crawler"
    case buckets = "Buckets"
    case relay = "Relay"
}

struct CrawlerBucketEntry: Identifiable, Hashable {
    let url: URL
    let isDirectory: Bool
    let size: UInt64?
    let modifiedAt: Date?

    var id: String { url.path }

    var name: String {
        url.lastPathComponent.isEmpty ? url.path : url.lastPathComponent
    }

    var displayKind: String {
        isDirectory ? "bucket" : "file"
    }

    var sizeLabel: String {
        guard let size else { return "—" }
        return ByteCountFormatter.string(fromByteCount: Int64(size), countStyle: .file)
    }
}

enum KitchenSinkMode: String, CaseIterable, Hashable {
    case balanced = "Balanced"
    case performance = "Performance"
    case debug = "Debug"
}

@MainActor
final class KitchenSinkViewModel: ObservableObject {
    @Published var selectedTab: KitchenSinkTab = .overview
    @Published var isEnabled = true
    @Published var selectedMode: KitchenSinkMode = .balanced
    @Published var counter = 0
    @Published var sliderValue = 42.0
    @Published var stepperValue = 3
    @Published var notes = "A kitchen sink app for iOS, iPadOS, macOS, and Mac Catalyst."
    @Published var newItemText = ""
    @Published var items = ["Alpha", "Beta", "Gamma"]
    @Published var activityLog: [String] = []
    @Published var crawlerRelay = ""
    @Published var crawlerAuthors = ""
    @Published var crawlerIds = ""
    @Published var crawlerLimit = "25"
    @Published var crawlerGenericTag = "d"
    @Published var crawlerGenericValue = ""
    @Published var crawlerHashtag = ""
    @Published var crawlerMentions = ""
    @Published var crawlerReferences = ""
    @Published var crawlerKinds = "1,7"
    @Published var crawlerSearch = ""
    @Published var crawlerSubscriptionID = "ffi-kitchen-sink"
    @Published var relayLogging = "info"
    @Published var relayConfigFilePath = ".gnostr/relay.toml"
    @Published var crawlerStatusMessage = "Idle"
    @Published var crawlerStatus: RelayProcessState?
    @Published var crawlerDiscovery: [RelayDiscoveryEntry] = []
    @Published var crawlerBucketStatusMessage = "No bucket directory loaded."
    @Published var crawlerBucketCurrentPath = "/"
    @Published var crawlerBucketEntries: [CrawlerBucketEntry] = []
    @Published var crawlerBucketPreviewPath = "No file selected."
    @Published var crawlerBucketPreview = "Select a bucket file to inspect it."
    @Published var crawlerServerMessage = "Idle"
    @Published var crawlerServerState: RelayProcessState?
    @Published var crawlerQueryStatusMessage = "Ready to submit."
    @Published var crawlerQueryResult = "No query submitted yet."
    @Published var relayHost = "127.0.0.1"
    @Published var relayPort = "3030"
    @Published var relayStatusMessage = "Idle"
    @Published var relayStatus: RelayProcessState?
    @Published var relayDiscovery: [RelayDiscoveryEntry] = []

    let platformLabel: String
    let asyncGitKinds: [AsyncGitEventKind]
    let sampleNote: GitNote
    let crawlerBaseURL = URL(string: "http://127.0.0.1:3030")!
    let relayBaseURL = URL(string: "http://127.0.0.1:3030")!
    let embeddedCrawlerBaseURL = URL(string: "http://crawler.localhost:3030")!
    let embeddedRelayBaseURL = URL(string: "http://relay.localhost:3030")!
    let crawlerBucketsRootURL: URL
    let crawlerClient: CrawlerClient
    let embeddedCrawlerClient: CrawlerClient
    let relayClient: RelayClient
    let embeddedRelayClient: RelayClient
    let crawlerServerController: CrawlerServerController
    let relayServerController: RelayServerController
    let supportsLocalCrawlerControl: Bool

    init() {
        appTrace("KitchenSinkViewModel.init")
        #if targetEnvironment(macCatalyst)
        self.platformLabel = "Mac Catalyst"
        #elseif os(macOS)
        self.platformLabel = "macOS"
        #elseif os(iOS)
        self.platformLabel = "iOS / iPadOS"
        #else
        self.platformLabel = "Other"
        #endif

        self.asyncGitKinds = FFIKitchenSink.asyncGitEventKinds()
        self.sampleNote = GitNote(
            noteID: "deadbeef",
            annotatedID: "cafebabe",
            notesRef: "refs/notes/commits",
            message: "FFI kitchen sink sample note",
            author: "alice",
            committer: "bob",
            committerTime: 1_234
        )
        self.crawlerClient = FFIKitchenSink.crawlerClient()
        self.embeddedCrawlerClient = FFIKitchenSink.crawlerClient(
            baseURL: embeddedCrawlerBaseURL,
            session: Self.embeddedCrawlerSession()
        )
        self.relayClient = FFIKitchenSink.relayClient()
        self.embeddedRelayClient = FFIKitchenSink.relayClient(
            baseURL: embeddedRelayBaseURL,
            session: Self.embeddedCrawlerSession()
        )
        self.crawlerServerController = CrawlerServerController()
        self.relayServerController = RelayServerController()
        self.crawlerBucketsRootURL = Self.crawlerBucketsRootDirectoryURL()
        self.supportsLocalCrawlerControl = self.crawlerServerController.isAvailable
        self.loadRelayDefaults()
        self.crawlerRelay = Self.defaultCrawlerRelayTargets().joined(separator: ",")
        self.refreshCrawlerStatus()
        self.refreshRelayStatus()
        self.rebuildCrawlerPreview()
        self.refreshCrawlerBuckets()
        self.bootstrapServices()
        self.log("GUI ready")
    }

    private static func embeddedCrawlerSession() -> URLSession {
        appTrace("KitchenSinkViewModel.embeddedCrawlerSession")
        let configuration = URLSessionConfiguration.ephemeral
        configuration.protocolClasses = [EmbeddedCrawlerURLProtocol.self]
        configuration.waitsForConnectivity = false
        return URLSession(configuration: configuration)
    }

    private var crawlerServiceClient: CrawlerClient {
        appTrace("KitchenSinkViewModel.crawlerServiceClient")
        return supportsLocalCrawlerControl ? crawlerClient : embeddedCrawlerClient
    }

    private var relayServiceClient: RelayClient {
        appTrace("KitchenSinkViewModel.relayServiceClient")
        #if os(macOS) || targetEnvironment(macCatalyst)
        return relayClient
        #else
        return embeddedRelayClient
        #endif
    }

    private static func defaultCrawlerRelayTargets() -> [String] {
        ["ws://127.0.0.1:8080", "wss://relay.damus.io", "wss://nos.lol"]
    }

    var crawlerQueryParameters: CrawlerQueryParameters {
        CrawlerQueryParameters(
            relay: trimmedOrNil(crawlerRelay),
            authors: trimmedOrNil(crawlerAuthors),
            ids: trimmedOrNil(crawlerIds),
            limit: Int(crawlerLimit.trimmingCharacters(in: .whitespacesAndNewlines)),
            genericTag: trimmedOrNil(crawlerGenericTag),
            genericValue: trimmedOrNil(crawlerGenericValue),
            hashtag: trimmedOrNil(crawlerHashtag),
            mentions: trimmedOrNil(crawlerMentions),
            references: trimmedOrNil(crawlerReferences),
            kinds: trimmedOrNil(crawlerKinds),
            search: trimmedOrNil(crawlerSearch)
        )
    }

    var crawlerWirePreview: String {
        (try? crawlerQueryParameters.buildWireQuery(subscriptionID: crawlerSubscriptionID)) ?? "unavailable"
    }

    var crawlerBucketCurrentDirectoryURL: URL {
        let trimmed = crawlerBucketCurrentPath.trimmingCharacters(in: CharacterSet(charactersIn: "/"))
        if trimmed.isEmpty {
            return crawlerBucketsRootURL
        }
        return crawlerBucketsRootURL.appendingPathComponent(trimmed, isDirectory: true)
    }

    var crawlerURLPreview: String {
        crawlerQueryParameters.queryURL(baseURL: crawlerBaseURL).absoluteString
    }

    var relayListenPreview: String {
        let port = UInt16(relayPort) ?? 3030
        return RelayEndpoints.listenEndpoint(host: relayHost, port: port)
    }

    var relayConfiguration: RelayConfiguration {
        RelayConfiguration(logging: relayLogging, configFilePath: relayConfigFilePath)
    }

    func incrementCounter() {
        appTrace("KitchenSinkViewModel.incrementCounter")
        counter += 1
        log("Counter -> \(counter)")
    }

    func resetWorkbench() {
        appTrace("KitchenSinkViewModel.resetWorkbench")
        counter = 0
        sliderValue = 42
        stepperValue = 3
        selectedMode = .balanced
        isEnabled = true
        log("Workbench reset")
    }

    func randomizeWorkbench() {
        appTrace("KitchenSinkViewModel.randomizeWorkbench")
        sliderValue = Double.random(in: 0...100)
        stepperValue = Int.random(in: 0...10)
        selectedMode = KitchenSinkMode.allCases.randomElement() ?? .balanced
        isEnabled.toggle()
        log("Workbench randomized")
    }

    func addItem() {
        appTrace("KitchenSinkViewModel.addItem")
        let trimmed = newItemText.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else { return }
        items.insert(trimmed, at: 0)
        newItemText = ""
        log("Added item \(trimmed)")
    }

    func removeItem(at index: Int) {
        appTrace("KitchenSinkViewModel.removeItem index=\(index)")
        guard items.indices.contains(index) else { return }
        let value = items.remove(at: index)
        log("Removed item \(value)")
    }

    func resetCrawlerFields() {
        appTrace("KitchenSinkViewModel.resetCrawlerFields")
        crawlerRelay = Self.defaultCrawlerRelayTargets().joined(separator: ",")
        crawlerAuthors = ""
        crawlerIds = ""
        crawlerLimit = "25"
        crawlerGenericTag = "d"
        crawlerGenericValue = ""
        crawlerHashtag = ""
        crawlerMentions = ""
        crawlerReferences = ""
        crawlerKinds = "1,7"
        crawlerSearch = ""
        crawlerSubscriptionID = "ffi-kitchen-sink"
        crawlerQueryStatusMessage = "Ready to submit."
        crawlerQueryResult = "No query submitted yet."
        rebuildCrawlerPreview()
        log("Crawler fields reset")
    }

    func openCrawlerBuckets() {
        appTrace("KitchenSinkViewModel.openCrawlerBuckets")
        selectedTab = .buckets
        refreshCrawlerBuckets()
        log("Opened bucket browser")
    }

    func applyCrawlerPreset(_ preset: CrawlerPreset) {
        appTrace("KitchenSinkViewModel.applyCrawlerPreset \(preset.rawValue)")
        switch preset {
        case .nip34:
            crawlerRelay = "wss://relay.damus.io"
            crawlerAuthors = "npub1example"
            crawlerIds = "deadbeef"
            crawlerLimit = "10"
            crawlerGenericTag = "d"
            crawlerGenericValue = "refs/notes/commits"
            crawlerKinds = "1617"
            crawlerSearch = "git note"
        case .hashtags:
            crawlerRelay = "wss://nos.lol"
            crawlerHashtag = "gnostr"
            crawlerMentions = "npub1example"
            crawlerReferences = "note1example"
            crawlerKinds = "1,7"
            crawlerSearch = "discover"
        case .profiles:
            crawlerRelay = "wss://relay.snort.social"
            crawlerAuthors = "npub1example,npub1another"
            crawlerKinds = "0,3"
            crawlerSearch = "profile"
        }
        rebuildCrawlerPreview()
        log("Applied crawler preset \(preset.rawValue)")
    }

    func rebuildCrawlerPreview() {
        appTrace("KitchenSinkViewModel.rebuildCrawlerPreview")
        log("Crawler query updated")
    }

    func submitCrawlerQuery() {
        appTrace("KitchenSinkViewModel.submitCrawlerQuery")
        let client = crawlerServiceClient
        let parameters = crawlerQueryParameters

        crawlerQueryStatusMessage = "Discovering relay targets..."
        crawlerQueryResult = "Waiting for response..."

        Task {
            do {
                let discoveredRelays = (try? await client.relaysTXT())
                    .map(Self.relayTargets(from:)) ?? []
                let configuredRelays = Self.relayTargets(from: crawlerRelay)
                let targets = Self.uniqueRelayTargets(
                    [Self.localCrawlerRelay] + configuredRelays + discoveredRelays
                )

                guard !targets.isEmpty else {
                    await MainActor.run {
                        crawlerQueryStatusMessage = "No relay targets available"
                        crawlerQueryResult = "No relay targets available."
                        log(crawlerQueryStatusMessage)
                    }
                    return
                }

                await MainActor.run {
                    crawlerQueryStatusMessage = "Submitting query to \(targets.count) relays..."
                    crawlerQueryResult = "Waiting for \(targets.count) relay responses..."
                }

                var sections: [String] = []
                for target in targets {
                    var targetParameters = parameters
                    targetParameters.relay = target
                    do {
                        let response = try await client.queryPage(targetParameters)
                        let trimmed = response.trimmingCharacters(in: .whitespacesAndNewlines)
                        let rendered = trimmed.isEmpty ? "No results." : trimmed
                        sections.append("""
                        === \(target) ===
                        \(rendered)
                        """)
                    } catch {
                        sections.append("""
                        === \(target) ===
                        Query failed: \(error.localizedDescription)
                        """)
                    }
                }

                let combinedResult = sections.joined(separator: "\n\n")
                await MainActor.run {
                    crawlerQueryStatusMessage = "Submitted query to \(targets.count) relays"
                    crawlerQueryResult = combinedResult.isEmpty ? "No results." : combinedResult
                    log("Crawler query submitted")
                }
            } catch {
                let message = "Crawler query failed: \(error.localizedDescription)"
                await MainActor.run {
                    crawlerQueryStatusMessage = message
                    crawlerQueryResult = error.localizedDescription
                    log(message)
                }
            }
        }
    }

    private static let localCrawlerRelay = "ws://127.0.0.1:8080"

    nonisolated private static func relayTargets(from text: String) -> [String] {
        text
            .split { $0 == "," || $0.isNewline || $0.isWhitespace }
            .map { String($0).trimmingCharacters(in: .whitespacesAndNewlines) }
            .filter { !$0.isEmpty }
    }

    nonisolated private static func uniqueRelayTargets(_ relays: [String]) -> [String] {
        var seen = Set<String>()
        var unique: [String] = []
        for relay in relays {
            if seen.insert(relay).inserted {
                unique.append(relay)
            }
        }
        return unique
    }

    func loadRelayDefaults() {
        appTrace("KitchenSinkViewModel.loadRelayDefaults")
        let defaults = RelayConfiguration.rustDefault() ?? RelayConfiguration()
        relayLogging = defaults.logging
        relayConfigFilePath = defaults.configFilePath
        log("Loaded relay defaults")
    }

    func bootstrapServices() {
        appTrace("KitchenSinkViewModel.bootstrapServices supportsLocalCrawlerControl=\(supportsLocalCrawlerControl)")
        if supportsLocalCrawlerControl {
            Task {
                do {
                    let crawlerState = try await crawlerServerController.start()
                    let relayState = try await relayServerController.start()
                    await MainActor.run {
                        crawlerServerState = crawlerState
                        crawlerStatus = crawlerState
                        crawlerServerMessage = crawlerState.message
                        crawlerStatusMessage = crawlerState.message
                        relayStatus = relayState
                        relayStatusMessage = relayState.message
                        log("Embedded services bootstrapped")
                    }
                    await MainActor.run {
                        refreshCrawlerDiscovery()
                        refreshRelayDiscovery()
                    }
                } catch {
                    await MainActor.run {
                        crawlerServerMessage = "Service bootstrap failed: \(error.localizedDescription)"
                        crawlerStatusMessage = crawlerServerMessage
                        relayStatusMessage = crawlerServerMessage
                        log(crawlerServerMessage)
                    }
                }
            }
            return
        }

        _ = EmbeddedCrawlerService.shared.start()
        _ = EmbeddedRelayService.shared.start()
        crawlerServerState = EmbeddedCrawlerService.shared.status()
        crawlerStatus = crawlerServerState
        crawlerServerMessage = crawlerServerState?.message ?? "Idle"
        crawlerStatusMessage = crawlerServerMessage
        relayStatus = EmbeddedRelayService.shared.status()
        relayStatusMessage = relayStatus?.message ?? "Idle"
        refreshCrawlerDiscovery()
        refreshRelayDiscovery()
        log("Embedded services bootstrapped")
    }

    func refreshCrawlerStatus() {
        appTrace("KitchenSinkViewModel.refreshCrawlerStatus")
        if supportsLocalCrawlerControl {
            let controller = crawlerServerController
            Task {
                await MainActor.run {
                    self.log("Crawler status request -> FFI runtime")
                }
                let state = controller.status()
                await MainActor.run {
                    crawlerServerState = state
                    crawlerServerMessage = state.message
                    crawlerStatus = state
                    crawlerStatusMessage = state.message
                    log("Crawler status refreshed: \(state.message)")
                }
            }
            return
        }

        Task {
            do {
                let state = try await crawlerServiceClient.relayStatus()
                await MainActor.run {
                    crawlerServerState = state
                    crawlerServerMessage = state.message
                    crawlerStatus = state
                    crawlerStatusMessage = state.message
                    log("Crawler backend status refreshed")
                }
            } catch {
                await MainActor.run {
                    crawlerServerMessage = "Crawler status failed: \(error.localizedDescription)"
                    crawlerStatusMessage = crawlerServerMessage
                    log(crawlerStatusMessage)
                }
            }
        }
    }

    func startCrawler() {
        appTrace("KitchenSinkViewModel.startCrawler")
        if supportsLocalCrawlerControl {
            let controller = crawlerServerController
            Task {
                do {
                    await MainActor.run {
                        self.log("Crawler start requested -> FFI runtime")
                    }
                    let state = try await controller.start()
                    await MainActor.run {
                        crawlerServerState = state
                        crawlerServerMessage = state.message
                        crawlerStatus = state
                        crawlerStatusMessage = state.message
                        log("Crawler started: \(state.message)")
                    }
                } catch {
                    await MainActor.run {
                        crawlerServerMessage = "Crawler start failed: \(error.localizedDescription)"
                        crawlerStatusMessage = crawlerServerMessage
                        log(crawlerStatusMessage)
                    }
                }
            }
            return
        }

        Task {
            do {
                let state = try await crawlerServiceClient.startRelay()
                await MainActor.run {
                    crawlerServerState = state
                    crawlerServerMessage = state.message
                    crawlerStatus = state
                    crawlerStatusMessage = state.message
                    log("Crawler backend started")
                }
            } catch {
                await MainActor.run {
                    crawlerServerMessage = "Crawler start failed: \(error.localizedDescription)"
                    crawlerStatusMessage = crawlerServerMessage
                    log(crawlerStatusMessage)
                }
            }
        }
    }

    func stopCrawler() {
        appTrace("KitchenSinkViewModel.stopCrawler")
        if supportsLocalCrawlerControl {
            let controller = crawlerServerController
            Task {
                do {
                    await MainActor.run {
                        self.log("Crawler stop requested -> FFI runtime")
                    }
                    let state = try controller.stop()
                    await MainActor.run {
                        crawlerServerState = state
                        crawlerServerMessage = state.message
                        crawlerStatus = state
                        crawlerStatusMessage = state.message
                        log("Crawler stopped: \(state.message)")
                    }
                } catch {
                    await MainActor.run {
                        crawlerServerMessage = "Crawler stop failed: \(error.localizedDescription)"
                        crawlerStatusMessage = crawlerServerMessage
                        log(crawlerStatusMessage)
                    }
                }
            }
            return
        }

        Task {
            do {
                let state = try await crawlerServiceClient.stopRelay()
                await MainActor.run {
                    crawlerServerState = state
                    crawlerServerMessage = state.message
                    crawlerStatus = state
                    crawlerStatusMessage = state.message
                    log("Crawler backend stopped")
                }
            } catch {
                await MainActor.run {
                    crawlerServerMessage = "Crawler stop failed: \(error.localizedDescription)"
                    crawlerStatusMessage = crawlerServerMessage
                    log(crawlerStatusMessage)
                }
            }
        }
    }

    func refreshCrawlerDiscovery() {
        appTrace("KitchenSinkViewModel.refreshCrawlerDiscovery")
        Task {
            do {
                let discovery = try await crawlerServiceClient.relayDiscovery()
                await MainActor.run {
                    crawlerDiscovery = discovery
                    crawlerStatusMessage = "Loaded \(discovery.count) crawler discovery entries"
                    refreshCrawlerBuckets()
                    log(crawlerStatusMessage)
                }
            } catch {
                await MainActor.run {
                    crawlerStatusMessage = "Crawler discovery failed: \(error.localizedDescription)"
                    log(crawlerStatusMessage)
                }
            }
        }
    }

    func refreshCrawlerBuckets() {
        appTrace("KitchenSinkViewModel.refreshCrawlerBuckets")
        let fileManager = FileManager.default
        do {
            try fileManager.createDirectory(at: crawlerBucketsRootURL, withIntermediateDirectories: true)
            let directory = crawlerBucketCurrentDirectoryURL
            let values: Set<URLResourceKey> = [.isDirectoryKey, .fileSizeKey, .contentModificationDateKey]
            let entries = try fileManager.contentsOfDirectory(
                at: directory,
                includingPropertiesForKeys: Array(values),
                options: [.skipsHiddenFiles]
            )
            let mapped: [CrawlerBucketEntry] = entries.compactMap { url in
                let resourceValues = try? url.resourceValues(forKeys: values)
                return CrawlerBucketEntry(
                    url: url,
                    isDirectory: resourceValues?.isDirectory ?? false,
                    size: resourceValues?.fileSize.map(UInt64.init),
                    modifiedAt: resourceValues?.contentModificationDate
                )
            }
            .sorted {
                if $0.isDirectory != $1.isDirectory { return $0.isDirectory && !$1.isDirectory }
                return $0.name.localizedStandardCompare($1.name) == .orderedAscending
            }
            crawlerBucketEntries = mapped
            crawlerBucketStatusMessage = mapped.isEmpty
                ? "Bucket directory is empty."
                : "Loaded \(mapped.count) bucket items."
            crawlerBucketPreviewPath = crawlerBucketCurrentDirectoryURL.path
            if crawlerBucketPreview.isEmpty {
                crawlerBucketPreview = "Select a bucket file to inspect it."
            }
        } catch {
            crawlerBucketEntries = []
            crawlerBucketStatusMessage = "Bucket browser failed: \(error.localizedDescription)"
            crawlerBucketPreview = error.localizedDescription
        }
    }

    func goToCrawlerBucketParent() {
        appTrace("KitchenSinkViewModel.goToCrawlerBucketParent")
        let directory = crawlerBucketCurrentDirectoryURL
        let parent = directory.deletingLastPathComponent()
        guard parent.path.hasPrefix(crawlerBucketsRootURL.path) else { return }
        crawlerBucketCurrentPath = relativeCrawlerBucketPath(for: parent)
        refreshCrawlerBuckets()
    }

    func openCrawlerBucket(_ entry: CrawlerBucketEntry) {
        appTrace("KitchenSinkViewModel.openCrawlerBucket \(entry.name)")
        if entry.isDirectory {
            crawlerBucketCurrentPath = relativeCrawlerBucketPath(for: entry.url)
            crawlerBucketPreviewPath = entry.url.path
            crawlerBucketPreview = "Directory selected."
            refreshCrawlerBuckets()
            return
        }

        crawlerBucketPreviewPath = entry.url.path
        crawlerBucketPreview = readCrawlerBucketFile(at: entry.url)
        crawlerBucketStatusMessage = "Previewing \(entry.name)"
    }

    private func readCrawlerBucketFile(at url: URL) -> String {
        appTrace("KitchenSinkViewModel.readCrawlerBucketFile \(url.path)")
        do {
            let data = try Data(contentsOf: url)
            if let string = String(data: data, encoding: .utf8) {
                return string.isEmpty ? "Empty file." : string
            }
            return "Binary file (\(data.count) bytes)"
        } catch {
            return "Failed to read file: \(error.localizedDescription)"
        }
    }

    private func relativeCrawlerBucketPath(for directory: URL) -> String {
        appTrace("KitchenSinkViewModel.relativeCrawlerBucketPath \(directory.path)")
        let root = crawlerBucketsRootURL.standardizedFileURL.path
        let path = directory.standardizedFileURL.path
        guard path.hasPrefix(root) else { return "/" }
        let remainder = path.dropFirst(root.count).trimmingCharacters(in: CharacterSet(charactersIn: "/"))
        return remainder.isEmpty ? "/" : "/" + remainder
    }

    private static func crawlerBucketsRootDirectoryURL() -> URL {
        let base = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first
            ?? URL(fileURLWithPath: FileManager.default.currentDirectoryPath, isDirectory: true)
        return base.appendingPathComponent("org/gnostr/gnostr/crawler", isDirectory: true)
    }

    func refreshRelayStatus() {
        appTrace("KitchenSinkViewModel.refreshRelayStatus")
        if supportsLocalCrawlerControl {
            Task {
                do {
                    let state = try await relayServiceClient.status()
                    await MainActor.run {
                        relayStatus = state
                        relayStatusMessage = state.message
                        log("Relay status refreshed")
                    }
                } catch {
                    await MainActor.run {
                        relayStatusMessage = "Relay status failed: \(error.localizedDescription)"
                        log(relayStatusMessage)
                    }
                }
            }
            return
        }

        let state = EmbeddedRelayService.shared.status()
        relayStatus = state
        relayStatusMessage = state.message
        log("Relay status refreshed")
    }

    func startRelay() {
        appTrace("KitchenSinkViewModel.startRelay")
        if supportsLocalCrawlerControl {
            Task {
                do {
                    let state = try await relayServerController.start()
                    await MainActor.run {
                        relayStatus = state
                        relayStatusMessage = state.message
                        log("Relay started")
                    }
                } catch {
                    await MainActor.run {
                        relayStatusMessage = "Relay start failed: \(error.localizedDescription)"
                        log(relayStatusMessage)
                    }
                }
            }
            return
        }

        Task {
            do {
                let state = try await relayServiceClient.start()
                await MainActor.run {
                    relayStatus = state
                    relayStatusMessage = state.message
                    log("Relay started")
                }
            } catch {
                await MainActor.run {
                    relayStatusMessage = "Relay start failed: \(error.localizedDescription)"
                    log(relayStatusMessage)
                }
            }
        }
    }

    func stopRelay() {
        appTrace("KitchenSinkViewModel.stopRelay")
        if supportsLocalCrawlerControl {
            Task {
                do {
                    let state = try await relayServerController.stop()
                    await MainActor.run {
                        relayStatus = state
                        relayStatusMessage = state.message
                        log("Relay stopped")
                    }
                } catch {
                    await MainActor.run {
                        relayStatusMessage = "Relay stop failed: \(error.localizedDescription)"
                        log(relayStatusMessage)
                    }
                }
            }
            return
        }

        Task {
            do {
                let state = try await relayServiceClient.stop()
                await MainActor.run {
                    relayStatus = state
                    relayStatusMessage = state.message
                    log("Relay stopped")
                }
            } catch {
                await MainActor.run {
                    relayStatusMessage = "Relay stop failed: \(error.localizedDescription)"
                    log(relayStatusMessage)
                }
            }
        }
    }

    func refreshRelayDiscovery() {
        appTrace("KitchenSinkViewModel.refreshRelayDiscovery")
        Task {
            do {
                let discovery = try await relayServiceClient.discovery()
                await MainActor.run {
                    relayDiscovery = discovery
                    relayStatusMessage = "Loaded \(discovery.count) discovery entries"
                    log(relayStatusMessage)
                }
            } catch {
                await MainActor.run {
                    relayStatusMessage = "Relay discovery failed: \(error.localizedDescription)"
                    log(relayStatusMessage)
                }
            }
        }
    }

    func log(_ message: String) {
        appTrace("KitchenSinkViewModel.log \(message)")
        let formatter = Self.timestampFormatter
        let line = "[\(formatter.string(from: Date()))] \(message)"
        activityLog.insert(line, at: 0)
        NSLog("%@", line)
    }

    private func trimmedOrNil(_ value: String) -> String? {
        appTrace("KitchenSinkViewModel.trimmedOrNil")
        let trimmed = value.trimmingCharacters(in: .whitespacesAndNewlines)
        return trimmed.isEmpty ? nil : trimmed
    }

    private static let timestampFormatter: DateFormatter = {
        let formatter = DateFormatter()
        formatter.dateFormat = "HH:mm:ss"
        return formatter
    }()
}

#if os(macOS) || targetEnvironment(macCatalyst) || os(iOS)
private struct CrawlerServerCommand {
    let executableURL: URL
    let arguments: [String]
    let currentDirectoryURL: URL
}

@MainActor
final class CrawlerServerController {
    private let bridge = RustCrawlerBridge.shared
    private let serviceName = "gnostr-crawler"
    private let port: UInt16 = 3030
    private let fileManager = FileManager.default
    private var logTailTask: Task<Void, Never>?

    var isAvailable: Bool {
        bridge.isAvailable
    }

    func status() -> RelayProcessState {
        appTrace("CrawlerServerController.status")
        do {
            return try bridge.crawlerRuntimeStatus() ?? RelayProcessState(
                running: false,
                message: "Crawler runtime status unavailable"
            )
        } catch {
            return RelayProcessState(
                running: false,
                message: "Crawler runtime status failed: \(error.localizedDescription)"
            )
        }
    }

    func start() async throws -> RelayProcessState {
        appTrace("CrawlerServerController.start")
        guard let state = try bridge.startCrawlerRuntime(port: port) else {
            throw CocoaError(.coderInvalidValue)
        }
        return state
    }

    func stop() throws -> RelayProcessState {
        appTrace("CrawlerServerController.stop")
        guard let state = try bridge.stopCrawlerRuntime() else {
            throw CocoaError(.coderInvalidValue)
        }
        return state
    }

    #if false
    private func resolveCommand() throws -> CrawlerServerCommand {
        let workdir = repositoryRootURL()
        let arguments = ["crawler", "serve", "--port", String(port), "--detach"]

        if let binary = resolvedBinaryURL() {
            return CrawlerServerCommand(
                executableURL: binary,
                arguments: arguments,
                currentDirectoryURL: workdir
            )
        }

        if fileManager.isExecutableFile(atPath: "/usr/bin/env") {
            return CrawlerServerCommand(
                executableURL: URL(fileURLWithPath: "/usr/bin/env"),
                arguments: ["gnostr"] + arguments,
                currentDirectoryURL: workdir
            )
        }

        throw CocoaError(.fileNoSuchFile)
    }

    private func launch(_ command: CrawlerServerCommand) throws -> String {
        let process = Process()
        let stdout = Pipe()
        let stderr = Pipe()
        process.executableURL = command.executableURL
        process.arguments = command.arguments
        process.currentDirectoryURL = command.currentDirectoryURL
        process.standardOutput = stdout
        process.standardError = stderr
        var environment = ProcessInfo.processInfo.environment
        if environment["RUST_LOG"] == nil || environment["RUST_LOG"]?.isEmpty == true {
            environment["RUST_LOG"] = "debug"
        }
        process.environment = environment

        try process.run()
        process.waitUntilExit()

        let output = [stdout, stderr]
            .map { $0.fileHandleForReading.readDataToEndOfFile() }
            .compactMap { String(data: $0, encoding: .utf8) }
            .joined(separator: "\n")
            .trimmingCharacters(in: .whitespacesAndNewlines)

        guard process.terminationStatus == 0 else {
            let message = output.isEmpty ? "gnostr crawler serve exited with status \(process.terminationStatus)" : output
            throw NSError(domain: "CrawlerServerController", code: Int(process.terminationStatus), userInfo: [NSLocalizedDescriptionKey: message])
        }

        return output
    }

    private func startLogTail() {
        guard logTailTask == nil else { return }
        let logURL = crawlerLogFileURL()
        logTailTask = Task.detached(priority: .background) {
            var lastLength: Int = 0
            while !Task.isCancelled {
                autoreleasepool {
                    if let data = try? Data(contentsOf: logURL) {
                        if data.count < lastLength {
                            lastLength = 0
                            print("[gnostr-crawler] log rotated: \(logURL.path)")
                        }

                        if data.count > lastLength {
                            let chunk = data[lastLength...]
                            if let text = String(data: chunk, encoding: .utf8) {
                                for line in text.split(whereSeparator: \.isNewline, omittingEmptySubsequences: false) {
                                    let trimmed = line.trimmingCharacters(in: .whitespacesAndNewlines)
                                    if !trimmed.isEmpty {
                                        print("[gnostr-crawler] \(trimmed)")
                                    }
                                }
                            }
                            lastLength = data.count
                        }
                    }
                }

                try? await Task.sleep(nanoseconds: 500_000_000)
            }
        }
    }

    private func stopLogTail() {
        logTailTask?.cancel()
        logTailTask = nil
    }

    private func crawlerLogFileURL() -> URL {
        FileManager.default.homeDirectoryForCurrentUser
            .appendingPathComponent(".config/gnostr/gnostr.log", isDirectory: false)
    }

    private func existingDetachedPID() -> UInt32? {
        guard let pid = readDetachedPID() else {
            return nil
        }

        if pidIsRunning(pid) {
            return pid
        }

        removeStalePIDFile()
        return nil
    }

    private func readDetachedPID() -> UInt32? {
        let url = detachedPIDFileURL()
        guard let raw = try? String(contentsOf: url, encoding: .utf8) else {
            return nil
        }
        return UInt32(raw.trimmingCharacters(in: .whitespacesAndNewlines))
    }

    private func removeStalePIDFile() {
        let url = detachedPIDFileURL()
        if fileManager.fileExists(atPath: url.path) {
            try? fileManager.removeItem(at: url)
        }
    }

    private func detachedPIDFileURL() -> URL {
        repositoryRootURL().appendingPathComponent(".gnostr/\(serviceName).pid")
    }

    private func repositoryRootURL() -> URL {
        var directory = URL(fileURLWithPath: fileManager.currentDirectoryPath, isDirectory: true)
        while true {
            let cargoToml = directory.appendingPathComponent("Cargo.toml")
            if fileManager.fileExists(atPath: cargoToml.path) {
                return directory
            }

            let parent = directory.deletingLastPathComponent()
            if parent.path == directory.path {
                return URL(fileURLWithPath: fileManager.currentDirectoryPath, isDirectory: true)
            }
            directory = parent
        }
    }

    private func resolvedBinaryURL() -> URL? {
        if let envBinary = ProcessInfo.processInfo.environment["GNOSTR_BIN"]?.trimmingCharacters(in: .whitespacesAndNewlines),
           !envBinary.isEmpty,
           fileManager.isExecutableFile(atPath: envBinary) {
            return URL(fileURLWithPath: envBinary)
        }

        for root in ancestorDirectories(from: repositoryRootURL()) {
            let debug = root.appendingPathComponent("target/debug/gnostr")
            if fileManager.isExecutableFile(atPath: debug.path) {
                return debug
            }

            let release = root.appendingPathComponent("target/release/gnostr")
            if fileManager.isExecutableFile(atPath: release.path) {
                return release
            }
        }

        return nil
    }

    private func ancestorDirectories(from url: URL) -> [URL] {
        var directories: [URL] = []
        var current = url
        while true {
            directories.append(current)
            let parent = current.deletingLastPathComponent()
            if parent.path == current.path {
                break
            }
            current = parent
        }
        return directories
    }

    private func pidIsRunning(_ pid: UInt32) -> Bool {
        if kill(pid_t(pid), 0) == 0 {
            return true
        }

        return errno == EPERM
    }
    #endif
}

 #if os(macOS) || targetEnvironment(macCatalyst)
@MainActor
final class RelayServerController {
    private let serviceName = "gnostr-relay"
    private let port: UInt16 = 3030
    private let fileManager = FileManager.default

    func status() -> RelayProcessState {
        appTrace("RelayServerController.status")
        guard let pid = existingDetachedPID() else {
            return RelayProcessState(running: false, message: "Relay server not running")
        }

        return RelayProcessState(
            running: true,
            pid: pid,
            message: "Relay server running (pid \(pid))"
        )
    }

    func start() async throws -> RelayProcessState {
        appTrace("RelayServerController.start")
        if let pid = existingDetachedPID() {
            return RelayProcessState(
                running: true,
                pid: pid,
                message: "Relay server already running (pid \(pid))"
            )
        }

        removeStalePIDFile()
        let command = try resolveCommand()
        let launchOutput = try launch(command)

        for _ in 0..<20 {
            if let pid = existingDetachedPID() {
                return RelayProcessState(
                    running: true,
                    pid: pid,
                    message: "Relay server started (pid \(pid))"
                )
            }
            try await Task.sleep(nanoseconds: 100_000_000)
        }

        return RelayProcessState(
            running: true,
            message: launchOutput.isEmpty ? "Relay server start requested" : launchOutput
        )
    }

    func stop() throws -> RelayProcessState {
        appTrace("RelayServerController.stop")
        guard let pid = existingDetachedPID() else {
            return RelayProcessState(running: false, message: "Relay server not running")
        }

        if kill(pid_t(pid), SIGTERM) != 0, errno != ESRCH {
            let message = String(cString: strerror(errno))
            throw NSError(domain: "RelayServerController", code: Int(errno), userInfo: [NSLocalizedDescriptionKey: message])
        }

        removeStalePIDFile()
        return RelayProcessState(running: false, pid: pid, message: "Relay server stopped")
    }

    private func resolveCommand() throws -> CrawlerServerCommand {
        appTrace("RelayServerController.resolveCommand")
        let workdir = repositoryRootURL()
        let arguments = ["relay", "--detach"]

        if let binary = resolvedBinaryURL() {
            return CrawlerServerCommand(
                executableURL: binary,
                arguments: arguments,
                currentDirectoryURL: workdir
            )
        }

        if fileManager.isExecutableFile(atPath: "/usr/bin/env") {
            return CrawlerServerCommand(
                executableURL: URL(fileURLWithPath: "/usr/bin/env"),
                arguments: ["gnostr"] + arguments,
                currentDirectoryURL: workdir
            )
        }

        throw CocoaError(.fileNoSuchFile)
    }

    private func launch(_ command: CrawlerServerCommand) throws -> String {
        appTrace("RelayServerController.launch \(command.executableURL.path)")
        let process = Process()
        let stdout = Pipe()
        let stderr = Pipe()
        process.executableURL = command.executableURL
        process.arguments = command.arguments
        process.currentDirectoryURL = command.currentDirectoryURL
        process.standardOutput = stdout
        process.standardError = stderr

        try process.run()
        process.waitUntilExit()

        let output = [stdout, stderr]
            .map { $0.fileHandleForReading.readDataToEndOfFile() }
            .compactMap { String(data: $0, encoding: .utf8) }
            .joined(separator: "\n")
            .trimmingCharacters(in: .whitespacesAndNewlines)

        guard process.terminationStatus == 0 else {
            let message = output.isEmpty ? "gnostr relay exited with status \(process.terminationStatus)" : output
            throw NSError(domain: "RelayServerController", code: Int(process.terminationStatus), userInfo: [NSLocalizedDescriptionKey: message])
        }

        return output
    }

    private func existingDetachedPID() -> UInt32? {
        appTrace("RelayServerController.existingDetachedPID")
        guard let pid = readDetachedPID() else {
            return nil
        }

        if pidIsRunning(pid) {
            return pid
        }

        removeStalePIDFile()
        return nil
    }

    private func readDetachedPID() -> UInt32? {
        appTrace("RelayServerController.readDetachedPID")
        let url = detachedPIDFileURL()
        guard let raw = try? String(contentsOf: url, encoding: .utf8) else {
            return nil
        }
        return UInt32(raw.trimmingCharacters(in: .whitespacesAndNewlines))
    }

    private func removeStalePIDFile() {
        appTrace("RelayServerController.removeStalePIDFile")
        let url = detachedPIDFileURL()
        if fileManager.fileExists(atPath: url.path) {
            try? fileManager.removeItem(at: url)
        }
    }

    private func detachedPIDFileURL() -> URL {
        appTrace("RelayServerController.detachedPIDFileURL")
        repositoryRootURL().appendingPathComponent(".gnostr/\(serviceName).pid")
    }

    private func repositoryRootURL() -> URL {
        appTrace("RelayServerController.repositoryRootURL")
        var directory = URL(fileURLWithPath: fileManager.currentDirectoryPath, isDirectory: true)
        while true {
            let cargoToml = directory.appendingPathComponent("Cargo.toml")
            if fileManager.fileExists(atPath: cargoToml.path) {
                return directory
            }

            let parent = directory.deletingLastPathComponent()
            if parent.path == directory.path {
                return URL(fileURLWithPath: fileManager.currentDirectoryPath, isDirectory: true)
            }
            directory = parent
        }
    }

    private func resolvedBinaryURL() -> URL? {
        appTrace("RelayServerController.resolvedBinaryURL")
        if let envBinary = ProcessInfo.processInfo.environment["GNOSTR_BIN"]?.trimmingCharacters(in: .whitespacesAndNewlines),
           !envBinary.isEmpty,
           fileManager.isExecutableFile(atPath: envBinary) {
            return URL(fileURLWithPath: envBinary)
        }

        for root in ancestorDirectories(from: repositoryRootURL()) {
            let debug = root.appendingPathComponent("target/debug/gnostr")
            if fileManager.isExecutableFile(atPath: debug.path) {
                return debug
            }

            let release = root.appendingPathComponent("target/release/gnostr")
            if fileManager.isExecutableFile(atPath: release.path) {
                return release
            }
        }

        return nil
    }

    private func ancestorDirectories(from url: URL) -> [URL] {
        appTrace("RelayServerController.ancestorDirectories \(url.path)")
        var directories: [URL] = []
        var current = url
        while true {
            directories.append(current)
            let parent = current.deletingLastPathComponent()
            if parent.path == current.path {
                break
            }
            current = parent
        }
        return directories
    }

    private func pidIsRunning(_ pid: UInt32) -> Bool {
        appTrace("RelayServerController.pidIsRunning \(pid)")
        if kill(pid_t(pid), 0) == 0 {
            return true
        }

        return errno == EPERM
    }
}
#else
@MainActor
final class RelayServerController {
    private let unavailableMessage = "Local relay process control is unavailable on this platform"

    func status() -> RelayProcessState {
        RelayProcessState(running: false, message: unavailableMessage)
    }

    func start() async throws -> RelayProcessState {
        status()
    }

    func stop() throws -> RelayProcessState {
        status()
    }
}
#endif
#else
@MainActor
final class CrawlerServerController {
    private let unavailableMessage = "Local crawler process control is unavailable on this platform"

    func status() -> RelayProcessState {
        RelayProcessState(running: false, message: unavailableMessage)
    }

    func start() async throws -> RelayProcessState {
        status()
    }

    func stop() throws -> RelayProcessState {
        status()
    }
}

@MainActor
final class RelayServerController {
    private let unavailableMessage = "Local relay process control is unavailable on this platform"

    func status() -> RelayProcessState {
        RelayProcessState(running: false, message: unavailableMessage)
    }

    func start() async throws -> RelayProcessState {
        status()
    }

    func stop() throws -> RelayProcessState {
        status()
    }
}
#endif

enum CrawlerPreset: String, CaseIterable, Identifiable {
    case nip34 = "NIP-34"
    case hashtags = "Hashtags"
    case profiles = "Profiles"

    var id: String { rawValue }
}

struct ContentView: View {
    @StateObject private var model = KitchenSinkViewModel()

    var body: some View {
        let _ = appTrace("ContentView.body")
        TabView(selection: $model.selectedTab) {
            overviewTab
                .tabItem { Label("Overview", systemImage: "square.grid.2x2") }
                .tag(KitchenSinkTab.overview)

            workbenchTab
                .tabItem { Label("Workbench", systemImage: "slider.horizontal.3") }
                .tag(KitchenSinkTab.workbench)

            asyncGitTab
                .tabItem { Label("AsyncGit", systemImage: "arrow.triangle.2.circlepath") }
                .tag(KitchenSinkTab.asyncGit)

            typesTab
                .tabItem { Label("Types", systemImage: "cube.transparent") }
                .tag(KitchenSinkTab.types)

            crawlerTab
                .tabItem { Label("Crawler", systemImage: "network") }
                .tag(KitchenSinkTab.crawler)

            bucketsTab
                .tabItem { Label("Buckets", systemImage: "folder") }
                .tag(KitchenSinkTab.buckets)

            relayTab
                .tabItem { Label("Relay", systemImage: "antenna.radiowaves.left.and.right") }
                .tag(KitchenSinkTab.relay)
        }
        .padding()
    }

    private var overviewTab: some View {
        scroll {
            title("FFI Kitchen Sink", subtitle: "Interactive controls for the FFI-backed gnostr stack.")

            groupBox("Platform") {
                infoRow("Platform", model.platformLabel)
                infoRow("AsyncGit kinds", "\(model.asyncGitKinds.count)")
                infoRow("Crawler base URL", model.crawlerBaseURL.absoluteString)
                infoRow("Relay base URL", model.relayBaseURL.absoluteString)
                infoRow("Crawler query", model.crawlerURLPreview)
            }

            groupBox("Bridge availability") {
                infoRow("Crawler", FFIKitchenSink.crawlerBridge.isAvailable ? "available" : "unavailable")
                infoRow("Relay", FFIKitchenSink.relayBridge.isAvailable ? "available" : "unavailable")
            }

            groupBox("Live previews") {
                infoRow("Relay endpoint", model.relayListenPreview)
                infoRow("Relay status", model.relayStatus?.message ?? model.relayStatusMessage)
                infoRow("Crawler status", model.crawlerStatus?.message ?? model.crawlerStatusMessage)
            }
        }
    }

    private var workbenchTab: some View {
        scroll {
            title("Workbench", subtitle: "Buttons, toggles, sliders, and lists that change state immediately.")

            groupBox("Controls") {
                Toggle("Enabled", isOn: $model.isEnabled)
                Picker("Mode", selection: $model.selectedMode) {
                    ForEach(KitchenSinkMode.allCases, id: \.self) { mode in
                        Text(mode.rawValue).tag(mode)
                    }
                }
                .pickerStyle(.segmented)

                HStack {
                    Button("Increment") { model.incrementCounter() }
                    Button("Randomize") { model.randomizeWorkbench() }
                    Button("Reset") { model.resetWorkbench() }
                }

                Slider(value: $model.sliderValue, in: 0...100, step: 1)
                Stepper("Stepper value: \(model.stepperValue)", value: $model.stepperValue, in: 0...10)
            }

            groupBox("Editable notes") {
                TextEditor(text: $model.notes)
                    .frame(minHeight: 120)
                    .overlay(RoundedRectangle(cornerRadius: 8).stroke(.quaternary))
            }

            groupBox("Counter and items") {
                infoRow("Counter", "\(model.counter)")
                HStack {
                    TextField("Add item", text: $model.newItemText)
                    Button("Add") { model.addItem() }
                }
                ForEach(Array(model.items.enumerated()), id: \.offset) { index, item in
                    HStack {
                        Text(item)
                        Spacer()
                        Button("Remove") { model.removeItem(at: index) }
                    }
                }
            }
        }
    }

    private var asyncGitTab: some View {
        scroll {
            title("AsyncGit", subtitle: "Shared event kinds and NIP-34 helpers exposed by the umbrella package.")

            groupBox("Event kinds") {
                ForEach(model.asyncGitKinds, id: \.self) { kind in
                    infoRow(kindLabel(kind), "\(kind.rawValue)")
                }
            }

            groupBox("Sample Git note") {
                infoRow("noteID", model.sampleNote.noteID)
                infoRow("annotatedID", model.sampleNote.annotatedID)
                infoRow("notesRef", model.sampleNote.notesRef ?? "nil")
                infoRow("message", model.sampleNote.message)
                infoRow("author", model.sampleNote.author)
                infoRow("committer", model.sampleNote.committer)
            }
        }
    }

    private var typesTab: some View {
        scroll {
            title("Types", subtitle: "Core Nostr and Git note values shared across the FFI layers.")

            groupBox("Live relay configuration") {
                infoRow("logging", model.relayLogging)
                infoRow("configFilePath", model.relayConfigFilePath)
                Button("Load Rust defaults") { model.loadRelayDefaults() }
            }

            groupBox("Activity log") {
                ForEach(Array(model.activityLog.prefix(8).enumerated()), id: \.offset) { _, entry in
                    Text(entry)
                        .font(.callout.monospaced())
                }
            }
        }
    }

    private var crawlerTab: some View {
        scroll {
            title("Crawler", subtitle: "Edit query inputs, submit them, and inspect the response.")

            groupBox("Crawler service") {
                infoRow("status", model.crawlerServerState?.message ?? model.crawlerServerMessage)
                infoRow("running", (model.crawlerServerState?.running ?? false) ? "yes" : "no")
                if !model.supportsLocalCrawlerControl {
                    Text("Using embedded in-app crawler service on this platform.")
                        .foregroundStyle(.secondary)
                }
                HStack {
                    Button("Start server") { model.startCrawler() }
                    Button("Stop server") { model.stopCrawler() }
                    Button("Refresh") { model.refreshCrawlerStatus() }
                    Button("Discovery") { model.refreshCrawlerDiscovery() }
                }
            }

            groupBox("Quick presets") {
                Picker("Preset", selection: .constant(CrawlerPreset.nip34)) {
                    ForEach(CrawlerPreset.allCases) { preset in
                        Text(preset.rawValue).tag(preset)
                    }
                }
                .hidden()

                HStack {
                    ForEach(CrawlerPreset.allCases) { preset in
                        Button(preset.rawValue) { model.applyCrawlerPreset(preset) }
                    }
                    Button("Reset") { model.resetCrawlerFields() }
                }
            }

            groupBox("Query editor") {
                labeledField("relay", text: $model.crawlerRelay)
                labeledField("authors", text: $model.crawlerAuthors)
                labeledField("ids", text: $model.crawlerIds)
                labeledField("limit", text: $model.crawlerLimit)
                labeledField("generic_tag", text: $model.crawlerGenericTag)
                labeledField("generic_value", text: $model.crawlerGenericValue)
                labeledField("hashtag", text: $model.crawlerHashtag)
                labeledField("mentions", text: $model.crawlerMentions)
                labeledField("references", text: $model.crawlerReferences)
                labeledField("kinds", text: $model.crawlerKinds)
                labeledField("search", text: $model.crawlerSearch)
                labeledField("subscription_id", text: $model.crawlerSubscriptionID)
                HStack {
                    Button("Refresh preview") { model.rebuildCrawlerPreview() }
                    Button("Submit query") { model.submitCrawlerQuery() }
                    Button("Open buckets") { model.openCrawlerBuckets() }
                }
            }

            groupBox("Preview") {
                infoRow("URL", model.crawlerURLPreview)
                Text(model.crawlerWirePreview)
                    .font(.footnote.monospaced())
                    .textSelection(.enabled)
            }

            groupBox("Query result") {
                infoRow("status", model.crawlerQueryStatusMessage)
                Text(model.crawlerQueryResult)
                    .font(.footnote.monospaced())
                    .frame(maxWidth: .infinity, minHeight: 140, alignment: .topLeading)
                    .padding(8)
                    .background(.quaternary.opacity(0.12))
                    .overlay(RoundedRectangle(cornerRadius: 8).stroke(.quaternary))
                    .textSelection(.enabled)
            }

            groupBox("Crawler discovery") {
                if model.crawlerDiscovery.isEmpty {
                    Text("No crawler discovery entries loaded.")
                        .foregroundStyle(.secondary)
                }
                ForEach(model.crawlerDiscovery, id: \.self) { entry in
                    VStack(alignment: .leading, spacing: 4) {
                        Text(entry.url)
                            .font(.headline)
                        Text(entry.name ?? entry.description ?? "No description")
                            .foregroundStyle(.secondary)
                        Text("NIPs: \(entry.supportedNips.map(String.init).joined(separator: ", "))")
                            .font(.footnote.monospaced())
                    }
                    .padding(.vertical, 4)
                }
            }
        }
    }

    private var bucketsTab: some View {
        scroll {
            title("Buckets", subtitle: "Inspect the crawler's on-disk relay buckets and cached files.")

            groupBox("Bucket browser") {
                infoRow("root", model.crawlerBucketsRootURL.path)
                infoRow("current", model.crawlerBucketCurrentPath)
                infoRow("status", model.crawlerBucketStatusMessage)
                HStack {
                    Button("Refresh") { model.refreshCrawlerBuckets() }
                    Button("Up") { model.goToCrawlerBucketParent() }
                    Button("Root") {
                        model.crawlerBucketCurrentPath = "/"
                        model.refreshCrawlerBuckets()
                    }
                }

                if model.crawlerBucketEntries.isEmpty {
                    Text("No bucket files found in this directory.")
                        .foregroundStyle(.secondary)
                } else {
                    VStack(alignment: .leading, spacing: 6) {
                        ForEach(model.crawlerBucketEntries) { entry in
                            Button {
                                model.openCrawlerBucket(entry)
                            } label: {
                                HStack(spacing: 10) {
                                    Image(systemName: entry.isDirectory ? "folder.fill" : "doc.text")
                                    Text(entry.name)
                                        .font(.body.monospaced())
                                    Spacer()
                                    Text(entry.displayKind)
                                        .foregroundStyle(.secondary)
                                    Text(entry.sizeLabel)
                                        .foregroundStyle(.secondary)
                                }
                                .frame(maxWidth: .infinity, alignment: .leading)
                            }
                            .buttonStyle(.plain)
                        }
                    }
                }
            }

            groupBox("Selected file") {
                infoRow("path", model.crawlerBucketPreviewPath)
                Text(model.crawlerBucketPreview)
                    .font(.footnote.monospaced())
                    .frame(maxWidth: .infinity, minHeight: 220, alignment: .topLeading)
                    .padding(8)
                    .background(.quaternary.opacity(0.12))
                    .overlay(RoundedRectangle(cornerRadius: 8).stroke(.quaternary))
                    .textSelection(.enabled)
            }
        }
    }

    private var relayTab: some View {
        scroll {
            title("Relay", subtitle: "Edit relay config and run lifecycle actions.")

            groupBox("Configuration") {
                labeledField("logging", text: $model.relayLogging)
                labeledField("config_file_path", text: $model.relayConfigFilePath)
                labeledField("host", text: $model.relayHost)
                labeledField("port", text: $model.relayPort)

                HStack {
                    Button("Load Rust defaults") { model.loadRelayDefaults() }
                    Button("Refresh status") { model.refreshRelayStatus() }
                }
                HStack {
                    Button("Start") { model.startRelay() }
                    Button("Stop") { model.stopRelay() }
                    Button("Discover") { model.refreshRelayDiscovery() }
                }
            }

            groupBox("Status") {
                infoRow("endpoint", model.relayListenPreview)
                infoRow("message", model.relayStatus?.message ?? model.relayStatusMessage)
                if let state = model.relayStatus {
                    infoRow("running", state.running ? "yes" : "no")
                    infoRow("pid", state.pid.map(String.init) ?? "nil")
                    infoRow("disk usage", state.diskUsageBytes.map(String.init) ?? "nil")
                }
            }

            groupBox("Discovery") {
                if model.relayDiscovery.isEmpty {
                    Text("No relay discovery entries loaded.")
                        .foregroundStyle(.secondary)
                }
                ForEach(model.relayDiscovery, id: \.self) { entry in
                    VStack(alignment: .leading, spacing: 4) {
                        Text(entry.url)
                            .font(.headline)
                        Text(entry.name ?? entry.description ?? "No description")
                            .foregroundStyle(.secondary)
                        Text("NIPs: \(entry.supportedNips.map(String.init).joined(separator: ", "))")
                            .font(.footnote.monospaced())
                    }
                    .padding(.vertical, 4)
                }
            }
        }
    }

    private func scroll<Content: View>(@ViewBuilder _ content: () -> Content) -> some View {
        appTrace("ContentView.scroll")
        return ScrollView {
            VStack(alignment: .leading, spacing: 16, content: content)
                .frame(maxWidth: .infinity, alignment: .leading)
        }
    }

    private func title(_ title: String, subtitle: String) -> some View {
        appTrace("ContentView.title \(title)")
        return VStack(alignment: .leading, spacing: 8) {
            Text(title)
                .font(.largeTitle.weight(.bold))
            Text(subtitle)
                .foregroundStyle(.secondary)
        }
    }

    private func groupBox<Content: View>(_ title: String, @ViewBuilder content: () -> Content) -> some View {
        appTrace("ContentView.groupBox \(title)")
        return GroupBox(title) {
            VStack(alignment: .leading, spacing: 8, content: content)
                .frame(maxWidth: .infinity, alignment: .leading)
        }
    }

    private func infoRow(_ label: String, _ value: String) -> some View {
        appTrace("ContentView.infoRow \(label)")
        return HStack(alignment: .top, spacing: 12) {
            Text(label)
                .font(.headline)
                .frame(width: 160, alignment: .leading)
            Text(value)
                .font(.body)
                .textSelection(.enabled)
            Spacer(minLength: 0)
        }
    }

    private func labeledField(_ label: String, text: Binding<String>) -> some View {
        appTrace("ContentView.labeledField \(label)")
        return HStack(alignment: .firstTextBaseline, spacing: 12) {
            Text(label)
                .font(.headline)
                .frame(width: 160, alignment: .leading)
            TextField(label, text: text)
                .textFieldStyle(.roundedBorder)
        }
    }

    private func kindLabel(_ kind: AsyncGitEventKind) -> String {
        let _ = appTrace("ContentView.kindLabel \(kind.rawValue)")
        switch kind {
        case .repoAnnouncement: return "Repo announcement"
        case .repoState: return "Repo state"
        case .patches: return "Patches"
        case .gitStatusOpen: return "Git status open"
        case .gitStatusApplied: return "Git status applied"
        case .gitStatusClosed: return "Git status closed"
        case .gitStatusDraft: return "Git status draft"
        }
    }
}

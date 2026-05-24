import Foundation
import Testing
@testable import RelayGUI

final class MockRelayURLProtocol: URLProtocol {
    nonisolated(unsafe) static var requests: [(String, String)] = []
    nonisolated(unsafe) static var responses: [String: (Int, Data)] = [:]

    override class func canInit(with request: URLRequest) -> Bool {
        true
    }

    override class func canonicalRequest(for request: URLRequest) -> URLRequest {
        request
    }

    override func startLoading() {
        guard let url = request.url else {
            client?.urlProtocol(self, didFailWithError: URLError(.badURL))
            return
        }

        let method = request.httpMethod ?? "GET"
        Self.requests.append((method, url.path))
        let key = "\(method) \(url.path)"
        let (statusCode, body) = Self.responses[key] ?? (500, Data(#"{"running":false,"pid":null,"message":"missing mock response","disk_usage_bytes":null}"#.utf8))
        let response = HTTPURLResponse(
            url: url,
            statusCode: statusCode,
            httpVersion: "HTTP/1.1",
            headerFields: ["Content-Type": "application/json"]
        )!
        client?.urlProtocol(self, didReceive: response, cacheStoragePolicy: .notAllowed)
        client?.urlProtocol(self, didLoad: body)
        client?.urlProtocolDidFinishLoading(self)
    }

    override func stopLoading() {}
}

final class MockRelayControl: RelayControlling, @unchecked Sendable {
    var calls: [String] = []

    var statusState = RelayProcessState(running: false, pid: nil, message: "relay stopped", diskUsageBytes: nil)
    var startState = RelayProcessState(running: true, pid: 41_001, message: "spawned detached relay pid 41001", diskUsageBytes: nil)
    var stopState = RelayProcessState(running: false, pid: 41_001, message: "stopped relay pid 41001", diskUsageBytes: nil)
    var restartState = RelayProcessState(running: true, pid: 41_002, message: "spawned detached relay pid 41002", diskUsageBytes: nil)

    func status() async throws -> RelayProcessState {
        calls.append("status")
        return statusState
    }

    func start() async throws -> RelayProcessState {
        calls.append("start")
        return startState
    }

    func stop() async throws -> RelayProcessState {
        calls.append("stop")
        return stopState
    }

    func restart() async throws -> RelayProcessState {
        calls.append("restart")
        return restartState
    }
}

@MainActor
@Test func relayDashboardViewModelBuildsEndpoint() {
    let model = RelayDashboardViewModel(host: "127.0.0.1", port: "8080")
    #expect(model.listenEndpoint == "ws://127.0.0.1:8080")
}

@MainActor
@Test func relayDashboardViewModelUpdatesConfigFields() {
    let model = RelayDashboardViewModel(autoStart: false)
    model.updateLogging("debug")
    model.updateConfigFilePath(".gnostr/custom-relay.toml")
    #expect(model.defaultConfiguration.logging == "debug")
    #expect(model.defaultConfiguration.configFilePath == ".gnostr/custom-relay.toml")
}

@MainActor
@Test func relayDashboardViewModelTogglesConfigEditor() {
    let model = RelayDashboardViewModel(autoStart: false)
    #expect(model.isConfigEditorVisible == false)
    model.toggleConfigEditorVisibility()
    #expect(model.isConfigEditorVisible == true)
}

@MainActor
@Test func relayDashboardViewModelTogglesConsoleVisibility() {
    let model = RelayDashboardViewModel(autoStart: false)
    #expect(model.isConsoleExpanded == false)
    model.toggleConsoleVisibility()
    #expect(model.isConsoleExpanded == true)
}

@MainActor
@Test func relayDashboardViewModelMapsStatusIndicatorState() {
    #expect(RelayDashboardViewModel.statusIndicatorState(for: "Relay FFI unavailable", isRunning: false) == .red)
    #expect(RelayDashboardViewModel.statusIndicatorState(for: "Relay FFI available", isRunning: false) == .yellow)
    #expect(RelayDashboardViewModel.statusIndicatorState(for: "Relay FFI available", isRunning: true) == .green)
    #expect(RelayDashboardViewModel.statusIndicatorState(for: "Relay stopping...", isRunning: false) == .yellow)
    #expect(RelayDashboardViewModel.statusIndicatorState(for: "stopped relay pid 41001", isRunning: false) == .red)
    #expect(RelayDashboardViewModel.statusIndicatorState(for: "Relay restarting...", isRunning: false) == .yellow)
    #expect(RelayDashboardViewModel.statusIndicatorState(for: "spawned detached relay pid 41002", isRunning: true) == .green)
}

@MainActor
@Test func relayDashboardViewModelRestartsRelay() async {
    let control = MockRelayControl()
    let model = RelayDashboardViewModel(autoStart: false, relayControl: control)

    await model.restartRelay()

    #expect(control.calls == ["restart"])
    #expect(model.isRunning == true)
    #expect(model.statusMessage == "spawned detached relay pid 41002")
}

@MainActor
@Test func relayControlClientRestartsRelayWithStopThenStart() async throws {
    MockRelayURLProtocol.requests = []
    MockRelayURLProtocol.responses = [
        "GET /api/relay/status": (200, Data(#"{"running":true,"pid":41001,"message":"relay already running with pid 41001","disk_usage_bytes":0}"#.utf8)),
        "POST /api/relay/stop": (200, Data(#"{"running":false,"pid":41001,"message":"stopped relay pid 41001","disk_usage_bytes":0}"#.utf8)),
        "POST /api/relay/start": (200, Data(#"{"running":true,"pid":41002,"message":"spawned detached relay pid 41002","disk_usage_bytes":0}"#.utf8))
    ]

    let configuration = URLSessionConfiguration.ephemeral
    configuration.protocolClasses = [MockRelayURLProtocol.self]
    let session = URLSession(configuration: configuration)
    let client = RelayControlClient(baseURL: URL(string: "http://127.0.0.1:3030")!, session: session)

    let state = try await client.restart()

    #expect(MockRelayURLProtocol.requests.map { "\($0.0) \($0.1)" } == [
        "GET /api/relay/status",
        "POST /api/relay/stop",
        "POST /api/relay/start"
    ])
    #expect(state.running == true)
    #expect(state.message == "spawned detached relay pid 41002")
}

@MainActor
@Test func relayDashboardViewModelLoadsConfigFileContents() throws {
    let fileManager = FileManager.default
    let root = fileManager.temporaryDirectory.appendingPathComponent(UUID().uuidString, isDirectory: true)
    let configURL = root.appendingPathComponent(".gnostr/relay.toml")
    try fileManager.createDirectory(at: configURL.deletingLastPathComponent(), withIntermediateDirectories: true)
    try "logging = \"trace\"\n".write(to: configURL, atomically: true, encoding: .utf8)

    let model = RelayDashboardViewModel(autoStart: false, configBaseDirectoryPath: root.path)
    #expect(model.configFileContents.contains("logging = \"trace\""))
    #expect(model.configFileStatus.hasPrefix("Loaded"))
}

@MainActor
@Test func relayDashboardViewModelKeepsEditorLockedUntilLoadPressed() {
    let model = RelayDashboardViewModel(autoStart: false)
    #expect(model.isConfigFileEditable == false)
    model.loadConfigFileContents()
    #expect(model.isConfigFileEditable == true)
}

@MainActor
@Test func relayDashboardViewModelLoadsTemplateWhenConfigFileIsMissing() {
    let fileManager = FileManager.default
    let root = fileManager.temporaryDirectory.appendingPathComponent(UUID().uuidString, isDirectory: true)

    let model = RelayDashboardViewModel(autoStart: false, configBaseDirectoryPath: root.path)
    #expect(model.configFileContents.contains("# [network]"))
    #expect(model.configFileContents.contains("# max_message_length = 524288"))
    #expect(model.configFileStatus.contains("template"))
}

@MainActor
@Test func relayDashboardViewModelSavesConfigFileContents() throws {
    let fileManager = FileManager.default
    let root = fileManager.temporaryDirectory.appendingPathComponent(UUID().uuidString, isDirectory: true)
    let model = RelayDashboardViewModel(autoStart: false, configBaseDirectoryPath: root.path)

    model.loadConfigFileContents()
    model.configFileContents = "logging = \"debug\"\n"
    model.saveConfigFileContents()

    let saved = try String(contentsOf: root.appendingPathComponent(".gnostr/relay.toml"), encoding: .utf8)
    #expect(saved.contains("logging = \"debug\""))
    #expect(model.configFileStatus.hasPrefix("Saved"))
    #expect(model.isConfigFileEditable == false)
}

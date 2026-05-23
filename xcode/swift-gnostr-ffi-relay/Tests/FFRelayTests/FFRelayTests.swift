import Foundation
import Testing
@testable import FFRelay

@Test func relayConfigurationUsesExpectedDefaults() {
    let configuration = RelayConfiguration.rustDefault() ?? RelayConfiguration()
    #expect(configuration.logging == "info")
    #expect(configuration.configFilePath.contains("relay.toml"))
}

@Test func relayEndpointFallsBackToWebSocketUrl() {
    let endpoint = RelayEndpoints.listenEndpoint(host: "127.0.0.1", port: 8080)
    #expect(endpoint == "ws://127.0.0.1:8080")
}

@Test func relayModelsAreCodable() throws {
    let state = RelayProcessState(running: true, pid: 42, message: "ok", diskUsageBytes: 99)
    let data = try JSONEncoder().encode(state)
    let decoded = try JSONDecoder().decode(RelayProcessState.self, from: data)
    #expect(decoded.running)
    #expect(decoded.pid == 42)
    #expect(decoded.diskUsageBytes == 99)
}

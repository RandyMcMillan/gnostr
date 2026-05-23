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

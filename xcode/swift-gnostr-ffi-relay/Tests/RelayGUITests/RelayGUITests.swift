import Testing
@testable import RelayGUI

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

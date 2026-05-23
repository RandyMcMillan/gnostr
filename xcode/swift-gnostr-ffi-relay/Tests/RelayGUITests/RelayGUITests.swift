import Testing
@testable import RelayGUI

@MainActor
@Test func relayDashboardViewModelBuildsEndpoint() {
    let model = RelayDashboardViewModel(host: "127.0.0.1", port: "8080")
    #expect(model.listenEndpoint == "ws://127.0.0.1:8080")
}

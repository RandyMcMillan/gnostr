import Foundation
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

@MainActor
@Test func relayDashboardViewModelLoadsConfigFileContents() throws {
    let fileManager = FileManager.default
    let root = fileManager.temporaryDirectory.appendingPathComponent(UUID().uuidString, isDirectory: true)
    let configURL = root.appendingPathComponent(".gnostr/relay.toml")
    try fileManager.createDirectory(at: configURL.deletingLastPathComponent(), withIntermediateDirectories: true)
    try "logging = \"trace\"\n".write(to: configURL, atomically: true, encoding: .utf8)

    let model = RelayDashboardViewModel(autoStart: false, currentDirectoryPath: root.path)
    #expect(model.configFileContents.contains("logging = \"trace\""))
    #expect(model.configFileStatus.hasPrefix("Loaded"))
}

@MainActor
@Test func relayDashboardViewModelSavesConfigFileContents() throws {
    let fileManager = FileManager.default
    let root = fileManager.temporaryDirectory.appendingPathComponent(UUID().uuidString, isDirectory: true)
    let model = RelayDashboardViewModel(autoStart: false, currentDirectoryPath: root.path)

    model.configFileContents = "logging = \"debug\"\n"
    model.saveConfigFileContents()

    let saved = try String(contentsOf: root.appendingPathComponent(".gnostr/relay.toml"), encoding: .utf8)
    #expect(saved.contains("logging = \"debug\""))
    #expect(model.configFileStatus.hasPrefix("Saved"))
}

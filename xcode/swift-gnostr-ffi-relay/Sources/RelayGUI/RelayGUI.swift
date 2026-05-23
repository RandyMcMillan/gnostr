import Foundation
import SwiftUI
import FFRelay
#if canImport(UIKit)
import UIKit
#elseif canImport(AppKit)
import AppKit
#endif

@MainActor
public final class RelayDashboardViewModel: ObservableObject {
    @Published public var host: String
    @Published public var port: String
    @Published public var defaultConfiguration: RelayConfiguration
    @Published public private(set) var listenEndpoint: String
    @Published public private(set) var statusMessage: String
    @Published public private(set) var isRunning = false
    @Published public private(set) var logLines: [String] = []

    public init(host: String = "127.0.0.1", port: String = "8080", autoStart: Bool = true) {
        self.host = host
        self.port = port
        self.defaultConfiguration = RelayConfiguration.rustDefault() ?? RelayConfiguration()
        self.listenEndpoint = RelayEndpoints.listenEndpoint(host: host, port: UInt16(port) ?? 8080)
        self.statusMessage = RustRelayBridge.shared.isAvailable ? "Relay FFI available" : "Relay FFI unavailable"
        appendLog("Relay dashboard ready")
        appendLog(statusMessage)
        if autoStart {
            startRelay()
        }
    }

    public func refresh() {
        defaultConfiguration = RelayConfiguration.rustDefault() ?? RelayConfiguration()
        listenEndpoint = RelayEndpoints.listenEndpoint(host: host, port: UInt16(port) ?? 8080)
        statusMessage = RustRelayBridge.shared.isAvailable ? "Relay FFI available" : "Relay FFI unavailable"
        appendLog("Refreshed relay defaults")
        appendLog("Listen endpoint: \(listenEndpoint)")
    }

    public func updateLogging(_ value: String) {
        defaultConfiguration.logging = value
        appendLog("Logging updated to \(value)")
    }

    public func updateConfigFilePath(_ value: String) {
        defaultConfiguration.configFilePath = value
        appendLog("Config file updated to \(value)")
        appendLog("System path: \(defaultConfiguration.resolvedConfigFilePath)")
    }

    public func updateEndpoint() {
        listenEndpoint = RelayEndpoints.listenEndpoint(host: host, port: UInt16(port) ?? 8080)
        appendLog("Endpoint updated to \(listenEndpoint)")
    }

    public func startRelay() {
        isRunning = true
        statusMessage = "Relay start requested"
        appendLog("Start pressed")
        appendLog(statusMessage)
        appendLog("FFI bridge does not expose relay process control yet.")
    }

    public func stopRelay() {
        isRunning = false
        statusMessage = "Relay stop requested"
        appendLog("Stop pressed")
        appendLog(statusMessage)
        appendLog("FFI bridge does not expose relay process control yet.")
    }

    public func clearLog() {
        logLines.removeAll()
        appendLog("Console cleared")
    }

    private func appendLog(_ message: String) {
        logLines.insert("[\(Self.timestamp())] \(message)", at: 0)
    }

    private static func timestamp() -> String {
        let formatter = DateFormatter()
        formatter.dateFormat = "HH:mm:ss"
        return formatter.string(from: Date())
    }
}

public struct RelayDashboardView: View {
    @ObservedObject private var model: RelayDashboardViewModel

    public init(model: RelayDashboardViewModel = RelayDashboardViewModel()) {
        _model = ObservedObject(wrappedValue: model)
    }

    public var body: some View {
        VStack(alignment: .leading, spacing: 20) {
            header
            controls
            card(title: "Rust defaults", content: configurationContent)
            card(title: "Listen endpoint", content: endpointContent)
            card(title: "Status", content: statusContent)
            card(title: "Log console", content: consoleContent)
        }
        .padding()
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
    }

    private var header: some View {
        HStack(alignment: .center, spacing: 16) {
            relayHeaderIcon()
                .resizable()
                .aspectRatio(contentMode: .fit)
                .frame(width: 56, height: 56)
            VStack(alignment: .leading, spacing: 8) {
                Text("Gnostr Relay")
                    .font(.largeTitle.bold())
                Text("FFI-only relay tools backed by the Rust relay crate.")
                    .foregroundColor(.secondary)
            }
        }
    }

    private func relayHeaderIcon() -> Image {
        guard let url = Bundle.module.url(forResource: "Icon", withExtension: "png"),
              let data = try? Data(contentsOf: url) else {
#if canImport(UIKit)
            return Image(uiImage: UIImage())
#elseif canImport(AppKit)
            return Image(nsImage: NSImage())
#else
            return Image(decorative: "")
#endif
        }

#if canImport(UIKit)
        return Image(uiImage: UIImage(data: data) ?? UIImage())
#elseif canImport(AppKit)
        return Image(nsImage: NSImage(data: data) ?? NSImage())
#else
        return Image(decorative: "")
#endif
    }

    private var configurationContent: some View {
        VStack(alignment: .leading, spacing: 12) {
            TextField("Logging", text: Binding(
                get: { model.defaultConfiguration.logging },
                set: { model.updateLogging($0) }
            ))
            .textFieldStyle(.roundedBorder)
            TextField("Config file", text: Binding(
                get: { model.defaultConfiguration.configFilePath },
                set: { model.updateConfigFilePath($0) }
            ))
            .textFieldStyle(.roundedBorder)
            row(label: "System path", value: model.defaultConfiguration.resolvedConfigFilePath)
            HStack {
                Button("Refresh defaults") { model.refresh() }
                Button("Clear console") { model.clearLog() }
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
    }

    private var controls: some View {
        HStack(spacing: 12) {
            Button("Start") { model.startRelay() }
                .disabled(model.isRunning)
            Button("Stop") { model.stopRelay() }
                .disabled(!model.isRunning)
            Button("Refresh") { model.refresh() }
        }
    }

    private var endpointContent: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                TextField("Host", text: Binding(
                    get: { model.host },
                    set: { newValue in
                        model.host = newValue
                        model.updateEndpoint()
                    }
                ))
                .textFieldStyle(.roundedBorder)
                TextField("Port", text: Binding(
                    get: { model.port },
                    set: { newValue in
                        model.port = newValue
                        model.updateEndpoint()
                    }
                ))
                .textFieldStyle(.roundedBorder)
                .frame(width: 100)
            }
            row(label: "Endpoint", value: model.listenEndpoint)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
    }

    private var statusContent: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text(model.statusMessage)
            row(label: "Running", value: model.isRunning ? "yes" : "no")
            Text("This package exposes models and defaults from `./relay` via FFI, then renders them in SwiftUI.")
                .font(.footnote)
                .foregroundColor(.secondary)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
    }

    private var consoleContent: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 8) {
                ForEach(Array(model.logLines.enumerated()), id: \.offset) { _, line in
                    Text(line)
                        .font(.system(.caption, design: .monospaced))
                        .frame(maxWidth: .infinity, alignment: .leading)
                }
            }
            .frame(maxWidth: .infinity, alignment: .leading)
        }
        .frame(maxHeight: 220, alignment: .topLeading)
    }

    private func row(label: String, value: String) -> some View {
        HStack(alignment: .top, spacing: 12) {
            Text(label)
                .frame(width: 92, alignment: .leading)
                .foregroundColor(.secondary)
            Text(value)
                .font(.system(.body, design: .monospaced))
        }
    }

    private func card(title: String, content: some View) -> some View {
        VStack(alignment: .leading, spacing: 12) {
            Text(title)
                .font(.headline)
            content
        }
        .padding()
        .background(
            RoundedRectangle(cornerRadius: 12, style: .continuous)
                .fill(Color.secondary.opacity(0.12))
        )
    }
}

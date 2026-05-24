import Foundation
import SwiftUI
import FFRelay
#if canImport(UIKit)
import UIKit
#elseif canImport(AppKit)
import AppKit
#endif

enum RelayStatusIndicatorState: String, Sendable {
    case green
    case yellow
    case red
}

@MainActor
public final class RelayDashboardViewModel: ObservableObject {
    @Published public var host: String
    @Published public var port: String
    @Published public var defaultConfiguration: RelayConfiguration
    @Published public var configFileContents: String
    @Published public var isConfigEditorVisible = false
    @Published public private(set) var listenEndpoint: String
    @Published public private(set) var statusMessage: String
    @Published public private(set) var configFileStatus: String
    @Published public private(set) var isRunning = false
    @Published public private(set) var logLines: [String] = []

    private let configBaseDirectoryPath: String

    public init(
        host: String = "127.0.0.1",
        port: String = "8080",
        autoStart: Bool = true,
        configBaseDirectoryPath: String? = nil
    ) {
        self.host = host
        self.port = port
        self.configBaseDirectoryPath = configBaseDirectoryPath ?? Self.defaultConfigBaseDirectoryPath()
        self.defaultConfiguration = RelayConfiguration.rustDefault() ?? RelayConfiguration()
        self.listenEndpoint = RelayEndpoints.listenEndpoint(host: host, port: UInt16(port) ?? 8080)
        self.statusMessage = RustRelayBridge.shared.isAvailable ? "Relay FFI available" : "Relay FFI unavailable"
        self.configFileContents = ""
        self.configFileStatus = "Config file not loaded"
        loadConfigFileContents()
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
        loadConfigFileContents()
        appendLog("Config file updated to \(value)")
        appendLog("System path: \(resolvedConfigFilePath)")
    }

    public func loadConfigFileContents() {
        do {
            if let contents = try readConfigFile() {
                configFileContents = contents
                configFileStatus = "Loaded \(resolvedConfigFilePath)"
            } else {
                configFileContents = Self.defaultConfigTemplate()
                configFileStatus = "Loaded template for \(resolvedConfigFilePath)"
            }
            appendLog(configFileStatus)
        } catch {
            configFileStatus = "Load failed: \(error.localizedDescription)"
            appendLog(configFileStatus)
        }
    }

    public func saveConfigFileContents() {
        do {
            try writeConfigFile(configFileContents)
            configFileStatus = "Saved \(resolvedConfigFilePath)"
            appendLog(configFileStatus)
        } catch {
            configFileStatus = "Save failed: \(error.localizedDescription)"
            appendLog(configFileStatus)
        }
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

    public func toggleConfigEditorVisibility() {
        isConfigEditorVisible.toggle()
        appendLog(isConfigEditorVisible ? "Config editor shown" : "Config editor hidden")
    }

    var statusIndicatorState: RelayStatusIndicatorState {
        Self.statusIndicatorState(for: statusMessage, isRunning: isRunning)
    }

    static func statusIndicatorState(for statusMessage: String, isRunning: Bool) -> RelayStatusIndicatorState {
        if statusMessage.localizedCaseInsensitiveContains("unavailable") {
            return .red
        }
        return isRunning ? .green : .yellow
    }

    private func appendLog(_ message: String) {
        logLines.insert("[\(Self.timestamp())] \(message)", at: 0)
    }

    private func readConfigFile() throws -> String? {
        let url = configFileURL
        guard FileManager.default.fileExists(atPath: url.path) else {
            return nil
        }
        return try String(contentsOf: url, encoding: .utf8)
    }

    private func writeConfigFile(_ contents: String) throws {
        let url = configFileURL
        try FileManager.default.createDirectory(
            at: url.deletingLastPathComponent(),
            withIntermediateDirectories: true
        )
        try contents.write(to: url, atomically: true, encoding: .utf8)
    }

    private var configFileURL: URL {
        URL(fileURLWithPath: resolvedConfigFilePath)
    }

    var resolvedConfigFilePath: String {
        RelayConfiguration.resolvedConfigFilePath(
            defaultConfiguration.configFilePath,
            relativeTo: configBaseDirectoryPath
        )
    }

    private static func defaultConfigBaseDirectoryPath() -> String {
        let bundleID = Bundle.main.bundleIdentifier ?? "gnostr"
        let supportURL = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first
            ?? FileManager.default.temporaryDirectory
        return supportURL.appendingPathComponent(bundleID, isDirectory: true).path
    }

    private static func defaultConfigTemplate() -> String {
        guard let url = Bundle.module.url(forResource: "relay", withExtension: "toml"),
              let contents = try? String(contentsOf: url, encoding: .utf8) else {
            return ""
        }
        return contents
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
        VStack(alignment: .leading, spacing: 0) {
#if os(macOS) || targetEnvironment(macCatalyst)
            header
                .padding(.top, 24)
                .padding(.bottom, 8)
                .background(headerBackground)
                .zIndex(1)
#else
            header
                .padding(.top, 24)
                .padding(.bottom, 8)
                .background(headerBackground)
                .zIndex(1)
#endif
            ScrollView(showsIndicators: false) {
                VStack(alignment: .leading, spacing: model.isConfigEditorVisible ? 12 : 20) {
                    controls
                    card(title: nil, compact: false, content: configurationContent)
                    card(title: "Log console", compact: model.isConfigEditorVisible, content: consoleContent)
                        .frame(maxHeight: model.isConfigEditorVisible ? 60 : 220, alignment: .topLeading)
                        .layoutPriority(1)
                }
                .frame(maxWidth: .infinity, alignment: .topLeading)
            }
            .frame(maxHeight: .infinity, alignment: .topLeading)
        }
        .padding(.horizontal)
        .padding(.bottom)
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
                //Text("todo")
                  //  .foregroundColor(.secondary)
            }
            Spacer()
            trafficLightIndicator
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
            HStack {
                //Text("Listen endpoint")
                //  .font(.subheadline.weight(.semibold))
                //Spacer()
                Text(model.listenEndpoint)
                    .font(.system(.body, design: .monospaced))
                Spacer()
                Button(action: { model.toggleConfigEditorVisibility() }) {
                    if #available(macOS 11.0, iOS 14.0, *) {
                        Image(systemName: model.isConfigEditorVisible ? "gearshape.fill" : "gearshape")
                    } else {
                        Text("⚙")
                    }
                }
                .buttonStyle(.plain)
            }
            if model.isConfigEditorVisible {
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
                //row(label: "System path", value: model.resolvedConfigFilePath)
                configEditor
                row(label: "File status", value: model.configFileStatus)
                HStack {
                    Button("Refresh defaults") { model.refresh() }
                    // the file should not be editable until load file pressed
                    // gray out text to indicate not active
                    Button("Load file") { model.loadConfigFileContents() }
                    Button("Save file") { model.saveConfigFileContents() }
                    // this belongs some where else Button("Clear console") { model.clearLog() }
                }
            } else {
                Text("Click the gear to edit the relay config file.")
                    .foregroundColor(.secondary)
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
    }

    @ViewBuilder
    private var configEditor: some View {
        if #available(macOS 11.0, iOS 14.0, *) {
            TextEditor(text: Binding(
                get: { model.configFileContents },
                set: { model.configFileContents = $0 }
            ))
            .font(.system(.body, design: .monospaced))
            .frame(minHeight: 240)
            .overlay(
                RoundedRectangle(cornerRadius: 8, style: .continuous)
                    .stroke(Color.secondary.opacity(0.3))
            )
        } else {
            TextField("Config contents", text: Binding(
                get: { model.configFileContents },
                set: { model.configFileContents = $0 }
            ))
            .textFieldStyle(.roundedBorder)
        }
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
        .frame(maxHeight: model.isConfigEditorVisible ? 60 : 220, alignment: .topLeading)
    }

    private var trafficLightIndicator: some View {
        Circle()
            .fill(statusIndicatorColor)
            .frame(width: 14, height: 14)
        .padding(8)
        .background(
            RoundedRectangle(cornerRadius: 10, style: .continuous)
                .fill(Color.secondary.opacity(0.12))
        )
    }

    @ViewBuilder
    private var headerBackground: some View {
        #if canImport(AppKit)
        if #available(macOS 12.0, *) {
            Color(nsColor: .windowBackgroundColor)
        } else {
            Color.white
        }
        #elseif canImport(UIKit)
        if #available(iOS 15.0, *) {
            Color(uiColor: .systemBackground)
        } else {
            Color(white: 1.0)
        }
        #else
        Color.clear
        #endif
    }

    private var statusIndicatorColor: Color {
        switch model.statusIndicatorState {
        case .green: return .green
        case .yellow: return .yellow
        case .red: return .red
        }
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

    private func card(title: String?, compact: Bool, content: some View) -> some View {
        VStack(alignment: .leading, spacing: 12) {
            if let title {
                Text(title)
                    .font(compact ? .subheadline.weight(.semibold) : .headline)
            }
            content
        }
        .padding(compact ? 10 : 16)
        .background(
            RoundedRectangle(cornerRadius: 12, style: .continuous)
                .fill(Color.secondary.opacity(0.12))
        )
    }
}

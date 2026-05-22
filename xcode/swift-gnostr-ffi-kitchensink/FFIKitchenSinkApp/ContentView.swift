import Foundation
import Darwin
import SwiftUI
import FFIKitchenSink

enum KitchenSinkTab: String, CaseIterable, Hashable {
    case overview = "Overview"
    case workbench = "Workbench"
    case asyncGit = "AsyncGit"
    case types = "Types"
    case crawler = "Crawler"
    case relay = "Relay"
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
    @Published var crawlerServerMessage = "Idle"
    @Published var crawlerServerState: RelayProcessState?
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
    let crawlerClient: CrawlerClient
    let relayClient: RelayClient
    let crawlerServerController: CrawlerServerController
    let supportsLocalCrawlerControl: Bool

    init() {
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
        self.relayClient = FFIKitchenSink.relayClient()
        self.crawlerServerController = CrawlerServerController()
        #if os(macOS) || targetEnvironment(macCatalyst)
        self.supportsLocalCrawlerControl = true
        #else
        self.supportsLocalCrawlerControl = false
        #endif
        self.loadRelayDefaults()
        self.refreshCrawlerStatus()
        self.rebuildCrawlerPreview()
        self.log("GUI ready")
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
        counter += 1
        log("Counter -> \(counter)")
    }

    func resetWorkbench() {
        counter = 0
        sliderValue = 42
        stepperValue = 3
        selectedMode = .balanced
        isEnabled = true
        log("Workbench reset")
    }

    func randomizeWorkbench() {
        sliderValue = Double.random(in: 0...100)
        stepperValue = Int.random(in: 0...10)
        selectedMode = KitchenSinkMode.allCases.randomElement() ?? .balanced
        isEnabled.toggle()
        log("Workbench randomized")
    }

    func addItem() {
        let trimmed = newItemText.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else { return }
        items.insert(trimmed, at: 0)
        newItemText = ""
        log("Added item \(trimmed)")
    }

    func removeItem(at index: Int) {
        guard items.indices.contains(index) else { return }
        let value = items.remove(at: index)
        log("Removed item \(value)")
    }

    func resetCrawlerFields() {
        crawlerRelay = ""
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
        rebuildCrawlerPreview()
        log("Crawler fields reset")
    }

    func applyCrawlerPreset(_ preset: CrawlerPreset) {
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
        log("Crawler query updated")
    }

    func loadRelayDefaults() {
        let defaults = RelayConfiguration.rustDefault() ?? RelayConfiguration()
        relayLogging = defaults.logging
        relayConfigFilePath = defaults.configFilePath
        log("Loaded relay defaults")
    }

    func refreshCrawlerStatus() {
        guard supportsLocalCrawlerControl else {
            crawlerServerState = RelayProcessState(
                running: false,
                message: "Local crawler process control is unavailable on this platform"
            )
            crawlerServerMessage = "Local crawler process control is unavailable on this platform"
            crawlerStatusMessage = crawlerServerMessage
            return
        }

        Task {
            let state = await crawlerServerController.status()
            await MainActor.run {
                crawlerServerState = state
                crawlerServerMessage = state.message
                crawlerStatus = state
                crawlerStatusMessage = state.message
                log("Crawler server status refreshed")
            }
        }
    }

    func startCrawler() {
        guard supportsLocalCrawlerControl else {
            crawlerServerState = RelayProcessState(
                running: false,
                message: "Local crawler process control is unavailable on this platform"
            )
            crawlerServerMessage = "Local crawler process control is unavailable on this platform"
            crawlerStatusMessage = crawlerServerMessage
            return
        }

        Task {
            do {
                let state = try await crawlerServerController.start()
                await MainActor.run {
                    crawlerServerState = state
                    crawlerServerMessage = state.message
                    crawlerStatus = state
                    crawlerStatusMessage = state.message
                    log("Crawler started")
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
        guard supportsLocalCrawlerControl else {
            crawlerServerState = RelayProcessState(
                running: false,
                message: "Local crawler process control is unavailable on this platform"
            )
            crawlerServerMessage = "Local crawler process control is unavailable on this platform"
            crawlerStatusMessage = crawlerServerMessage
            return
        }

        Task {
            do {
                let state = try crawlerServerController.stop()
                await MainActor.run {
                    crawlerServerState = state
                    crawlerServerMessage = state.message
                    crawlerStatus = state
                    crawlerStatusMessage = state.message
                    log("Crawler stopped")
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
        guard supportsLocalCrawlerControl else {
            crawlerServerMessage = "Local crawler process control is unavailable on this platform"
            crawlerStatusMessage = crawlerServerMessage
            return
        }

        Task {
            do {
                let discovery = try await crawlerClient.relayDiscovery()
                await MainActor.run {
                    crawlerDiscovery = discovery
                    crawlerStatusMessage = "Loaded \(discovery.count) crawler discovery entries"
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

    func refreshRelayStatus() {
        Task {
            do {
                let state = try await relayClient.status()
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
    }

    func startRelay() {
        Task {
            do {
                let state = try await relayClient.start()
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
        Task {
            do {
                let state = try await relayClient.stop()
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
        Task {
            do {
                let discovery = try await relayClient.discovery()
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
        let formatter = Self.timestampFormatter
        activityLog.insert("[\(formatter.string(from: Date()))] \(message)", at: 0)
    }

    private func trimmedOrNil(_ value: String) -> String? {
        let trimmed = value.trimmingCharacters(in: .whitespacesAndNewlines)
        return trimmed.isEmpty ? nil : trimmed
    }

    private static let timestampFormatter: DateFormatter = {
        let formatter = DateFormatter()
        formatter.dateFormat = "HH:mm:ss"
        return formatter
    }()
}

private struct CrawlerServerCommand {
    let executableURL: URL
    let arguments: [String]
    let currentDirectoryURL: URL
}

final class CrawlerServerController {
    private let serviceName = "gnostr-crawler"
    private let port: UInt16 = 3030
    private let fileManager = FileManager.default

    func status() -> RelayProcessState {
        guard let pid = existingDetachedPID() else {
            return RelayProcessState(running: false, message: "Crawler server not running")
        }

        return RelayProcessState(
            running: true,
            pid: pid,
            message: "Crawler server running (pid \(pid))"
        )
    }

    func start() async throws -> RelayProcessState {
        if let pid = existingDetachedPID() {
            return RelayProcessState(
                running: true,
                pid: pid,
                message: "Crawler server already running (pid \(pid))"
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
                    message: "Crawler server started (pid \(pid))"
                )
            }
            try await Task.sleep(nanoseconds: 100_000_000)
        }

        return RelayProcessState(
            running: true,
            message: launchOutput.isEmpty ? "Crawler server start requested" : launchOutput
        )
    }

    func stop() throws -> RelayProcessState {
        guard let pid = existingDetachedPID() else {
            return RelayProcessState(running: false, message: "Crawler server not running")
        }

        if kill(pid_t(pid), SIGTERM) != 0, errno != ESRCH {
            let message = String(cString: strerror(errno))
            throw NSError(domain: "CrawlerServerController", code: Int(errno), userInfo: [NSLocalizedDescriptionKey: message])
        }

        removeStalePIDFile()
        return RelayProcessState(running: false, pid: pid, message: "Crawler server stopped")
    }

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
}

enum CrawlerPreset: String, CaseIterable, Identifiable {
    case nip34 = "NIP-34"
    case hashtags = "Hashtags"
    case profiles = "Profiles"

    var id: String { rawValue }
}

struct ContentView: View {
    @StateObject private var model = KitchenSinkViewModel()

    var body: some View {
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
            title("Crawler", subtitle: "Edit query inputs and watch the query wire format update.")

            groupBox("Crawler service") {
                infoRow("status", model.crawlerServerState?.message ?? model.crawlerServerMessage)
                infoRow("running", (model.crawlerServerState?.running ?? false) ? "yes" : "no")
                if !model.supportsLocalCrawlerControl {
                    Text("Local crawler server control is available on macOS and Mac Catalyst only.")
                        .foregroundStyle(.secondary)
                }
                HStack {
                    Button("Start server") { model.startCrawler() }
                        .disabled(!model.supportsLocalCrawlerControl)
                    Button("Stop server") { model.stopCrawler() }
                        .disabled(!model.supportsLocalCrawlerControl)
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
                Button("Refresh preview") { model.rebuildCrawlerPreview() }
            }

            groupBox("Preview") {
                infoRow("URL", model.crawlerURLPreview)
                Text(model.crawlerWirePreview)
                    .font(.footnote.monospaced())
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
        ScrollView {
            VStack(alignment: .leading, spacing: 16, content: content)
                .frame(maxWidth: .infinity, alignment: .leading)
        }
    }

    private func title(_ title: String, subtitle: String) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            Text(title)
                .font(.largeTitle.weight(.bold))
            Text(subtitle)
                .foregroundStyle(.secondary)
        }
    }

    private func groupBox<Content: View>(_ title: String, @ViewBuilder content: () -> Content) -> some View {
        GroupBox(title) {
            VStack(alignment: .leading, spacing: 8, content: content)
                .frame(maxWidth: .infinity, alignment: .leading)
        }
    }

    private func infoRow(_ label: String, _ value: String) -> some View {
        HStack(alignment: .top, spacing: 12) {
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
        HStack(alignment: .firstTextBaseline, spacing: 12) {
            Text(label)
                .font(.headline)
                .frame(width: 160, alignment: .leading)
            TextField(label, text: text)
                .textFieldStyle(.roundedBorder)
        }
    }

    private func kindLabel(_ kind: AsyncGitEventKind) -> String {
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

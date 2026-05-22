import Foundation
import SwiftUI
import FFIKitchenSink

enum KitchenSinkTab: String, CaseIterable, Hashable {
    case overview = "Overview"
    case asyncGit = "AsyncGit"
    case types = "Types"
    case crawler = "Crawler"
    case relay = "Relay"
}

@MainActor
final class KitchenSinkViewModel: ObservableObject {
    @Published var selectedTab: KitchenSinkTab = .overview

    let platformLabel: String
    let asyncGitKinds: [AsyncGitEventKind]
    let sampleNote: GitNote
    let queryWire: String
    let crawlerBaseURL: String
    let relayBaseURL: String
    let relayConfiguration: RelayConfiguration

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
        self.queryWire = (try? CrawlerQueryParameters().buildWireQuery(subscriptionID: "ffi-kitchen-sink")) ?? "unavailable"
        self.crawlerBaseURL = FFIKitchenSink.crawlerClient().baseURL.absoluteString
        self.relayBaseURL = URL(string: "http://127.0.0.1:3030")!.absoluteString
        self.relayConfiguration = RelayConfiguration()
    }
}

struct ContentView: View {
    @StateObject private var model = KitchenSinkViewModel()

    var body: some View {
        TabView(selection: $model.selectedTab) {
            overviewTab
                .tabItem { Label("Overview", systemImage: "square.grid.2x2") }
                .tag(KitchenSinkTab.overview)

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
            title("FFI Kitchen Sink", subtitle: "A cross-platform iOS, macOS, and Mac Catalyst shell for the FFI-backed gnostr stack.")

            groupBox("Platform") {
                infoRow("Platform", model.platformLabel)
                infoRow("AsyncGit kinds", "\(model.asyncGitKinds.count)")
                infoRow("Crawler base URL", model.crawlerBaseURL)
                infoRow("Relay base URL", model.relayBaseURL)
            }

            groupBox("Bridge availability") {
                infoRow("Crawler", FFIKitchenSink.crawlerBridge.isAvailable ? "available" : "unavailable")
                infoRow("Relay", FFIKitchenSink.relayBridge.isAvailable ? "available" : "unavailable")
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
        }
    }

    private var typesTab: some View {
        scroll {
            title("Types", subtitle: "Core Nostr and Git note values shared across the FFI layers.")

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

    private var crawlerTab: some View {
        scroll {
            title("Crawler", subtitle: "Relay discovery and query-building helpers.")

            groupBox("Query wire format") {
                Text(model.queryWire)
                    .font(.footnote.monospaced())
                    .textSelection(.enabled)
            }
        }
    }

    private var relayTab: some View {
        scroll {
            title("Relay", subtitle: "Relay configuration and process helpers.")

            groupBox("Default configuration") {
                infoRow("logging", model.relayConfiguration.logging)
                infoRow("configFilePath", model.relayConfiguration.configFilePath)
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

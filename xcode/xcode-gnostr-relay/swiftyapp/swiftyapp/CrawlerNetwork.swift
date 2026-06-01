import Foundation
import SwiftUI
import RustyLib

@MainActor
public final class CrawlerNetworkStore: ObservableObject {
    @Published public private(set) var status: String = crawlerServiceStatus()
    @Published public private(set) var logs: String = crawlerServiceLogs()

    public let port: UInt16

    public init(port: UInt16 = 3030) {
        self.port = port
    }

    public var isRunning: Bool {
        self.status.localizedCaseInsensitiveContains("running")
    }

    public func refresh() async {
        let status = await Task.detached(priority: .userInitiated) {
            crawlerServiceStatus()
        }.value
        let logs = await Task.detached(priority: .userInitiated) {
            crawlerServiceLogs()
        }.value
        self.status = status
        self.logs = logs
    }

    public func startCrawlerServe() async {
        let port = self.port
        let status = await Task.detached(priority: .userInitiated) {
            crawlerServiceStart(port: port)
        }.value
        let logs = await Task.detached(priority: .userInitiated) {
            crawlerServiceLogs()
        }.value
        self.status = status
        self.logs = logs
    }

    public func stopCrawlerServe() async {
        let status = await Task.detached(priority: .userInitiated) {
            crawlerServiceStop()
        }.value
        let logs = await Task.detached(priority: .userInitiated) {
            crawlerServiceLogs()
        }.value
        self.status = status
        self.logs = logs
    }
}

public struct CrawlerNetworkDashboard: View {
    @ObservedObject var store: CrawlerNetworkStore
    @State private var logFontSize: CGFloat = 12

    public init(store: CrawlerNetworkStore) {
        self.store = store
    }

    public var body: some View {
        VStack(spacing: 0) {
            header
            Divider()
            logBody
        }
        .background(.background)
        .task {
            await self.store.refresh()
        }
    }

    private var header: some View {
        HStack(spacing: 12) {
            VStack(alignment: .leading, spacing: 2) {
                Text("Crawler Service")
                    .font(.headline)
                Text(store.status)
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .lineLimit(2)
            }

            Spacer()

            Button("Start") {
                Task { await store.startCrawlerServe() }
            }
            Button("Stop") {
                Task { await store.stopCrawlerServe() }
            }
            Button("Refresh") {
                Task { await store.refresh() }
            }
        }
        .padding()
        .background(.regularMaterial)
    }

    private var logBody: some View {
        ScrollViewReader { proxy in
            ScrollView {
                Text(store.logs.isEmpty ? "No crawler logs yet." : store.logs)
                    .font(.system(size: logFontSize, weight: .regular, design: .monospaced))
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .textSelection(.enabled)
                    .padding()
                    .id("crawler-log-tail")
            }
            .onChange(of: store.logs) { _ in
                withAnimation {
                    proxy.scrollTo("crawler-log-tail", anchor: .bottom)
                }
            }
        }
    }
}

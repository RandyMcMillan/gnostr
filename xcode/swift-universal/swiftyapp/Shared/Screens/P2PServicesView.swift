import Combine
import RustyLib
import SwiftUI

struct P2PServicesView: View {
    @State private var networkStatus = p2pNetworkStatus()
    @State private var networkLogs = p2pNetworkLogs()
    @State private var logFontSize: CGFloat = 12

    var body: some View {
        VStack(spacing: 0) {
            headerBar

            ScrollViewReader { proxy in
                ScrollView {
                    LazyVStack(alignment: .leading, spacing: 16) {
                        topicSummary
                        logPanel
                    }
                    .padding()
                    .frame(maxWidth: .infinity, alignment: .leading)
                }
                .onAppear {
                    scrollToLatestLog(using: proxy)
                }
                .onChange(of: networkLogs) { _ in
                    scrollToLatestLog(using: proxy)
                }
            }
        }
        .navigationTitle("P2P")
        .onAppear {
            refreshNetworkSnapshot()
        }
        .onReceive(Timer.publish(every: 1.0, on: .main, in: .common).autoconnect()) { _ in
            refreshNetworkSnapshot()
        }
    }

    private var headerBar: some View {
        HStack(alignment: .top, spacing: 12) {
            VStack(alignment: .leading, spacing: 4) {
                Text("P2P Services")
                    .font(.headline)
                Text(networkStatus)
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .lineLimit(2)
            }

            Spacer()

            VStack(alignment: .trailing, spacing: 8) {
                HStack(spacing: 8) {
                    Button("Refresh", action: refreshNetworkSnapshot)
                }
                .buttonStyle(.borderedProminent)

                HStack(spacing: 8) {
                    Button {
                        adjustLogFontSize(by: -1)
                    } label: {
                        Image(systemName: "minus")
                    }
                    .accessibilityLabel("Smaller log text")

                    Text("\(Int(logFontSize))")
                        .font(.caption.monospacedDigit())
                        .foregroundStyle(.secondary)
                        .frame(minWidth: 28)

                    Button {
                        adjustLogFontSize(by: 1)
                    } label: {
                        Image(systemName: "plus")
                    }
                    .accessibilityLabel("Larger log text")
                }
            }
        }
        .padding()
        .background(.regularMaterial)
    }

    private var topicSummary: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Topic alignment")
                .font(.subheadline.weight(.semibold))

            VStack(alignment: .leading, spacing: 8) {
                topicRow(label: "Discovery", value: "gnostr/p2p/presence")
                topicRow(label: "Chat", value: "gnostr-dev")
            }
            .padding()
            .background(.thinMaterial, in: RoundedRectangle(cornerRadius: 12, style: .continuous))
        }
    }

    private var logPanel: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Network logs")
                .font(.subheadline.weight(.semibold))

            ScrollView {
                LazyVStack(alignment: .leading, spacing: 4) {
                    if networkLogLines.isEmpty {
                        Text("No P2P logs yet.")
                            .font(logFont)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    } else {
                        ForEach(Array(networkLogLines.enumerated()), id: \.offset) { index, line in
                            Text(line)
                                .font(logFont)
                                .frame(maxWidth: .infinity, alignment: .leading)
                                .id(index)
                        }
                    }
                }
                .textSelection(.enabled)
                .padding()
            }
            .frame(minHeight: 260)
            .background(.thinMaterial, in: RoundedRectangle(cornerRadius: 12, style: .continuous))
        }
    }

    private func topicRow(label: String, value: String) -> some View {
        HStack(alignment: .firstTextBaseline, spacing: 12) {
            Text(label)
                .font(.caption.weight(.semibold))
                .foregroundStyle(.secondary)
                .frame(width: 86, alignment: .leading)
            Text(value)
                .font(.system(.caption, design: .monospaced))
                .textSelection(.enabled)
            Spacer()
        }
    }

    private var networkLogLines: [String] {
        networkLogs.split(separator: "\n", omittingEmptySubsequences: false).map(String.init)
    }

    private var logFont: Font {
        .system(size: logFontSize, weight: .regular, design: .monospaced)
    }

    private func refreshNetworkSnapshot() {
        networkStatus = p2pNetworkStatus()
        networkLogs = p2pNetworkLogs()
    }

    private func scrollToLatestLog(using proxy: ScrollViewProxy) {
        guard let lastIndex = networkLogLines.indices.last else { return }
        DispatchQueue.main.async {
            proxy.scrollTo(lastIndex, anchor: .bottom)
        }
    }

    private func adjustLogFontSize(by delta: CGFloat) {
        logFontSize = min(24, max(10, logFontSize + delta))
    }
}

struct P2PServicesView_Previews: PreviewProvider {
    static var previews: some View {
        P2PServicesView()
    }
}

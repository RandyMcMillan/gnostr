//
//  P2PChatView.swift
//  Universal App
//

import Combine
import Foundation
import RustyLib
import SwiftUI

struct P2PChatView: View {
    @State private var networkStatus = p2pNetworkStatus()
    @State private var networkLogs = p2pNetworkLogs()
    @State private var chatEntries: [P2PChatMessage] = []
    @State private var seenChatLogLines: Set<String> = []
    @State private var logFontSize: CGFloat = 12

    var body: some View {
        VStack(spacing: 0) {
            headerBar

            ScrollViewReader { proxy in
                ScrollView {
                    LazyVStack(alignment: .leading, spacing: 16) {
                        chatSummary
                        messagePanel
                    }
                    .padding()
                    .frame(maxWidth: .infinity, alignment: .leading)
                }
                .onAppear {
                    scrollToLatestLog(using: proxy)
                }
                .onChange(of: chatEntries.count) { _ in
                    scrollToLatestLog(using: proxy)
                }
            }
        }
        .navigationTitle("Chat")
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
                Text("P2P Chat")
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
                    .accessibilityLabel("Smaller chat log text")

                    Text("\(Int(logFontSize))")
                        .font(.caption.monospacedDigit())
                        .foregroundStyle(.secondary)
                        .frame(minWidth: 28)

                    Button {
                        adjustLogFontSize(by: 1)
                    } label: {
                        Image(systemName: "plus")
                    }
                    .accessibilityLabel("Larger chat log text")
                }
            }
        }
        .padding()
        .background(.regularMaterial)
    }

    private var chatSummary: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Chat messages")
                .font(.subheadline.weight(.semibold))

            VStack(alignment: .leading, spacing: 8) {
                topicRow(label: "Topic", value: "gnostr/p2p/presence")
                topicRow(label: "Filter", value: "kind == Chat")
            }
            .padding()
            .background(.thinMaterial, in: RoundedRectangle(cornerRadius: 12, style: .continuous))
        }
    }

    private var messagePanel: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Chat stream")
                .font(.subheadline.weight(.semibold))

            ScrollView {
                LazyVStack(alignment: .leading, spacing: 4) {
                    if chatEntries.isEmpty {
                        Text("No chat messages yet.")
                            .font(logFont)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    } else {
                        ForEach(Array(chatEntries.enumerated()), id: \.element.id) { index, message in
                            chatBubble(message)
                                .id(index)
                        }
                    }
                }
                .textSelection(.enabled)
                .padding()
            }
            .frame(minHeight: 340)
            .background(.thinMaterial, in: RoundedRectangle(cornerRadius: 12, style: .continuous))
        }
    }

    private var logFont: Font {
        .system(size: logFontSize, weight: .regular, design: .monospaced)
    }

    private func refreshNetworkSnapshot() {
        networkStatus = p2pNetworkStatus()
        networkLogs = p2pNetworkLogs()
        ingestChatLogs(networkLogs)
    }

    private func scrollToLatestLog(using proxy: ScrollViewProxy) {
        guard let lastIndex = chatEntries.indices.last else { return }
        DispatchQueue.main.async {
            proxy.scrollTo(lastIndex, anchor: .bottom)
        }
    }

    private func adjustLogFontSize(by delta: CGFloat) {
        logFontSize = min(24, max(10, logFontSize + delta))
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

    private func chatBubble(_ message: P2PChatMessage) -> some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack(alignment: .firstTextBaseline) {
                Text(message.from)
                    .font(.caption.weight(.semibold))
                Spacer()
                Text(message.topic)
                    .font(.caption2.monospaced())
                    .foregroundStyle(.secondary)
            }

            Text(message.text)
                .font(.system(size: logFontSize, weight: .regular, design: .monospaced))
                .frame(maxWidth: .infinity, alignment: .leading)
        }
        .padding(12)
        .background(message.isLocal ? Color.blue.opacity(0.14) : Color.primary.opacity(0.08), in: RoundedRectangle(cornerRadius: 12, style: .continuous))
    }

    private func ingestChatLogs(_ logs: String) {
        for line in logs.split(separator: "\n", omittingEmptySubsequences: false).map(String.init) {
            guard !seenChatLogLines.contains(line),
                  let message = P2PChatMessage.parse(from: line) else {
                continue
            }
            seenChatLogLines.insert(line)
            chatEntries.append(message)
        }
    }
}

private struct P2PChatMessage: Identifiable {
    let id = UUID()
    let from: String
    let topic: String
    let text: String
    let raw: String
    let isLocal: Bool

    private struct Payload: Decodable {
        let from: String?
        let content: [String]?
        let kind: String?
    }

    static func parse(from line: String) -> P2PChatMessage? {
        guard line.contains("kind\":\"Chat") || line.contains("kind: \"Chat\"") || line.contains("kind':'Chat'") || line.contains("\"kind\":\"Chat\"") else {
            return nil
        }

        let receivedMarkers = [
            "Received message: '",
            "received message '"
        ]

        guard let payloadStart = receivedMarkers.compactMap({ line.range(of: $0) }).first,
              let topicRange = line.range(of: "' on topic '", range: payloadStart.upperBound..<line.endIndex),
              let peerRange = line.range(of: "' from peer", range: topicRange.upperBound..<line.endIndex)
        else {
            return nil
        }

        let payloadString = String(line[payloadStart.upperBound..<topicRange.lowerBound])
        let topicString = String(line[topicRange.upperBound..<peerRange.lowerBound])
        let peerString = String(line[peerRange.upperBound...]).trimmingCharacters(in: .whitespaces)
        let rawText = payloadString.trimmingCharacters(in: CharacterSet(charactersIn: "'"))

        if let data = rawText.data(using: .utf8),
           let decoded = try? JSONDecoder().decode(Payload.self, from: data) {
            let text = decoded.content?.first ?? rawText
            let from = decoded.from ?? peerString
            return P2PChatMessage(
                from: from,
                topic: topicString,
                text: text,
                raw: line,
                isLocal: from.contains("79be667e")
            )
        }

        return P2PChatMessage(
            from: peerString,
            topic: topicString,
            text: rawText,
            raw: line,
            isLocal: false
        )
    }
}

struct P2PChatView_Previews: PreviewProvider {
    static var previews: some View {
        P2PChatView()
    }
}

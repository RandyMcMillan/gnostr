//
//  P2PListView.swift
//  Universal App
//

import RustyLib
import SwiftUI

struct P2PListView: View {
    @AppStorage("P2PChatTopics") private var chatTopicsRaw: String = "gnostr-dev"
    @State private var topicDraft: String = ""

    var body: some View {
        List {
            Section("CHAT") {
                HStack(spacing: 8) {
                    TextField("Topic", text: $topicDraft)
                        .textInputAutocapitalization(.never)
                        .autocorrectionDisabled()

                    Button(action: addTopic) {
                        Image(systemName: "plus.circle.fill")
                            .foregroundStyle(.green)
                    }
                    .accessibilityLabel("Add topic")
                    .disabled(normalizedDraft.isEmpty || chatTopics.contains(normalizedDraft))
                }

                ForEach(chatTopics, id: \.self) { topic in
                    NavigationLink(destination: P2PChatView(topic: topic)) {
                        VStack(alignment: .leading, spacing: 2) {
                            Text(topic)
                            if topic == "gnostr-dev" {
                                Text("default")
                                    .font(.caption)
                                    .foregroundStyle(.secondary)
                            }
                        }
                    }
                }
            }

            Section("P2P") {
                NavigationLink(destination: P2PPeersView()) {
                    Label("Peers", systemImage: "person.3.fill")
                }

                NavigationLink(destination: P2PServicesView()) {
                    Label("Services", systemImage: "antenna.radiowaves.left.and.right")
                }
            }
        }
        .navigationTitle("P2P")
        .onAppear {
            syncChatTopics()
        }
        #if os(macOS)
        .listStyle(SidebarListStyle())
        #endif
    }

    private var chatTopics: [String] {
        let topics = chatTopicsRaw
            .split(whereSeparator: { $0 == "\n" || $0 == "," })
            .map { $0.trimmingCharacters(in: .whitespacesAndNewlines) }
            .filter { !$0.isEmpty }

        return topics.isEmpty ? ["gnostr-dev"] : topics
    }

    private var normalizedDraft: String {
        topicDraft.trimmingCharacters(in: .whitespacesAndNewlines)
    }

    private func addTopic() {
        let topic = normalizedDraft
        guard !topic.isEmpty, !chatTopics.contains(topic) else { return }

        let updated = chatTopics + [topic]
        chatTopicsRaw = updated.joined(separator: "\n")
        _ = p2pChatRegisterTopic(topic)
        topicDraft = ""
    }

    private func syncChatTopics() {
        for topic in chatTopics {
            _ = p2pChatRegisterTopic(topic)
        }
    }
}

struct P2PListView_Previews: PreviewProvider {
    static var previews: some View {
        P2PListView()
    }
}

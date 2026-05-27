//
//  P2PListView.swift
//  Universal App
//

import Foundation
import CryptoKit
import SwiftUI

struct P2PListView: View {
    @EnvironmentObject private var appState: AppState
    @State private var topicDraft: String = ""
    @State private var chatTopics: [String] = ["gnostr-dev"]

    var body: some View {
        List {
            Section("IDENTITY") {
                SecureField("Private key", text: $appState.privateKey)
                    .textInputAutocapitalization(.never)
                    .autocorrectionDisabled()
                    .font(.system(.caption, design: .monospaced))
            }

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
            loadChatTopics()
        }
        .onChange(of: appState.privateKey) { _ in
            loadChatTopics()
            persistChatTopics()
        }
        #if os(macOS)
        .listStyle(SidebarListStyle())
        #endif
    }

    private var normalizedDraft: String {
        topicDraft.trimmingCharacters(in: .whitespacesAndNewlines)
    }

    private func addTopic() {
        let topic = normalizedDraft
        guard !topic.isEmpty, !chatTopics.contains(topic) else { return }

        chatTopics.append(topic)
        persistChatTopics()
        topicDraft = ""
    }

    private func loadChatTopics() {
        guard let key = privateKeyString else {
            chatTopics = ["gnostr-dev"]
            return
        }

        guard let url = chatTopicsFileURL(),
              let data = try? Data(contentsOf: url),
              let payload = try? JSONDecoder().decode(EncryptedChatTopicsPayload.self, from: data),
              let decryptedTopics = decryptTopics(payload, using: key),
              !decryptedTopics.isEmpty
        else {
            chatTopics = ["gnostr-dev"]
            return
        }

        chatTopics = decryptedTopics
    }

    private func persistChatTopics() {
        guard let key = privateKeyString else {
            print("Private key required to persist encrypted chat topics")
            return
        }
        guard let url = chatTopicsFileURL() else { return }

        do {
            let payload = try encryptTopics(chatTopics, using: key)
            try FileManager.default.createDirectory(
                at: url.deletingLastPathComponent(),
                withIntermediateDirectories: true
            )
            let data = try JSONEncoder().encode(payload)
            try data.write(to: url, options: .atomic)
        } catch {
            print("Failed to persist chat topics: \(error)")
        }
    }

    private func chatTopicsFileURL() -> URL? {
        FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first?
            .appendingPathComponent("gnostr", isDirectory: true)
            .appendingPathComponent("p2p-chat-topics.txt")
    }

    private var privateKeyString: String? {
        let trimmed = appState.privateKey.trimmingCharacters(in: .whitespacesAndNewlines)
        return trimmed.isEmpty ? nil : trimmed
    }

    private func encryptTopics(_ topics: [String], using privateKey: String) throws -> EncryptedChatTopicsPayload {
        let key = SymmetricKey(data: Data(SHA256.hash(data: Data(privateKey.utf8))))
        let plaintext = try JSONEncoder().encode(topics)
        let sealedBox = try ChaChaPoly.seal(plaintext, using: key)
        return EncryptedChatTopicsPayload(
            nonce: Data(sealedBox.nonce).base64EncodedString(),
            ciphertext: sealedBox.ciphertext.base64EncodedString(),
            tag: sealedBox.tag.base64EncodedString()
        )
    }

    private func decryptTopics(_ payload: EncryptedChatTopicsPayload, using privateKey: String) -> [String]? {
        guard
            let nonceData = Data(base64Encoded: payload.nonce),
            let ciphertext = Data(base64Encoded: payload.ciphertext),
            let tag = Data(base64Encoded: payload.tag)
        else {
            return nil
        }

        do {
            let key = SymmetricKey(data: Data(SHA256.hash(data: Data(privateKey.utf8))))
            let sealedBox = try ChaChaPoly.SealedBox(
                nonce: try ChaChaPoly.Nonce(data: nonceData),
                ciphertext: ciphertext,
                tag: tag
            )
            let plaintext = try ChaChaPoly.open(sealedBox, using: key)
            return try JSONDecoder().decode([String].self, from: plaintext)
        } catch {
            print("Failed to decrypt chat topics: \(error)")
            return nil
        }
    }
}

private struct EncryptedChatTopicsPayload: Codable {
    let nonce: String
    let ciphertext: String
    let tag: String
}

struct P2PListView_Previews: PreviewProvider {
    static var previews: some View {
        P2PListView()
    }
}

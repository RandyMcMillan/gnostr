//
//  P2PListView.swift
//  Universal App
//

import SwiftUI

struct P2PListView: View {
    @AppStorage("P2PChatTopic") private var chatTopic: String = "gnostr-dev"

    var body: some View {
        List {
            Section("CHAT") {
                TextField("Topic", text: $chatTopic)
                    .textInputAutocapitalization(.never)
                    .autocorrectionDisabled()

                NavigationLink(destination: P2PChatView(topic: chatTopic)) {
                    Label("Chat", systemImage: "message.fill")
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
        #if os(macOS)
        .listStyle(SidebarListStyle())
        #endif
    }
}

struct P2PListView_Previews: PreviewProvider {
    static var previews: some View {
        P2PListView()
    }
}

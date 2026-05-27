//
//  P2PListView.swift
//  Universal App
//

import SwiftUI

struct P2PListView: View {
    var body: some View {
        List {
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

//
//  P2PListView.swift
//  Universal App
//

import SwiftUI

struct P2PListView: View {
    #if os(iOS)
    @State private var showingServices = false
    #endif

    var body: some View {
        #if os(iOS)
        List {
            Section("P2P") {
                NavigationLink(
                    destination: P2PServicesView(),
                    isActive: $showingServices
                ) {
                    Label("Services", systemImage: "antenna.radiowaves.left.and.right")
                }
            }
        }
        .navigationTitle("P2P")
        #else
        List {
            Section("P2P") {
                NavigationLink(destination: P2PServicesView()) {
                    Label("Services", systemImage: "antenna.radiowaves.left.and.right")
                }
            }
        }
        .navigationTitle("P2P")
        #endif
    }
}

struct P2PListView_Previews: PreviewProvider {
    static var previews: some View {
        P2PListView()
    }
}

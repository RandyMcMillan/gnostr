//
//  ContentView.swift
//  swiftyapp
//
//  Created by Jonathan McKenzie on 7/9/24.
//

import SwiftUI
import RustyLib

struct ContentView: View {
    @State private var networkStatus = p2pNetworkStatus()

    var body: some View {
        VStack(spacing: 16) {
            Text("P2P Network")
                .font(.headline)
            Text(networkStatus)
                .font(.caption)
                .multilineTextAlignment(.center)
            HStack {
                Button("Start") {
                    networkStatus = p2pNetworkStart()
                }
                Button("Stop") {
                    networkStatus = p2pNetworkStop()
                }
            }
            Text(rustHello())
            Text(String(rustAdd(a: 10, b: 32)))
        }
        .padding()
        .onAppear {
            if networkStatus == "not running" {
                networkStatus = p2pNetworkStart()
            }
        }
    }
}

#Preview {
    ContentView()
}

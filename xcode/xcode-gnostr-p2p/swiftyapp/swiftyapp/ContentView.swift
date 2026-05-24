//
//  ContentView.swift
//  swiftyapp
//
//  Created by Jonathan McKenzie on 7/9/24.
//

import Combine
import SwiftUI
import RustyLib

struct ContentView: View {
    @State private var networkStatus = p2pNetworkStatus()
    @State private var networkLogs = p2pNetworkLogs()

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            Text("P2P Network").font(.headline)
            Text(networkStatus)
                .font(.caption)
                .multilineTextAlignment(.leading)
            HStack {
                Button("Start") { refreshNetworkState(p2pNetworkStart()) }
                Button("Stop") { refreshNetworkState(p2pNetworkStop()) }
                Button("Refresh Logs") { refreshNetworkState(p2pNetworkStatus()) }
            }
            ScrollView {
                Text(networkLogs.isEmpty ? "No P2P logs yet." : networkLogs)
                    .font(.system(.footnote, design: .monospaced))
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .textSelection(.enabled)
            }
            .frame(maxHeight: 240)
            Divider()
            Text(rustHello())
            Text(String(rustAdd(a: 10, b: 32)))
        }
        .padding()
        .onAppear {
            if networkStatus == "not running" {
                refreshNetworkState(p2pNetworkStart())
            }
        }
        .onReceive(Timer.publish(every: 1.0, on: .main, in: .common).autoconnect()) { _ in
            networkStatus = p2pNetworkStatus()
            networkLogs = p2pNetworkLogs()
        }
    }

    private func refreshNetworkState(_ status: String) {
        networkStatus = status
        networkLogs = p2pNetworkLogs()
    }
}

#Preview {
    ContentView()
}

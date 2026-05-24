//
//  ContentView.swift
//  swiftyapp
//
//  Created by Jonathan McKenzie on 7/9/24.
//

import Foundation
import Combine
import SwiftUI
import RustyLib

struct ContentView: View {
    @State private var networkStatus = p2pNetworkStatus()
    @State private var networkLogs = p2pNetworkLogs()
    @State private var showingNetworkPanel = false
    @State private var didAutoStartNetwork = false

    var body: some View {
        VStack(spacing: 0) {
            headerBar

            Color.clear
                .frame(maxWidth: .infinity, maxHeight: .infinity)
                .ignoresSafeArea()
        }
        .onAppear {
            if !didAutoStartNetwork {
                didAutoStartNetwork = true
                startNetwork()
            }
        }
        .onReceive(Timer.publish(every: 1.0, on: .main, in: .common).autoconnect()) { _ in
            refreshNetworkSnapshot()
        }
    }

    private var headerBar: some View {
        HStack(spacing: 12) {
            VStack(alignment: .leading, spacing: 2) {
                Text("P2P Network")
                    .font(.headline)
                Text(networkStatus)
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
            }

            Spacer()

            Button {
                showingNetworkPanel = true
            } label: {
                Image(systemName: "gearshape.fill")
                    .font(.title3.weight(.semibold))
                    .padding(10)
                    .background(.thinMaterial, in: Circle())
                    .shadow(radius: 2)
            }
            .accessibilityLabel("P2P settings")
            .fullScreenCover(isPresented: $showingNetworkPanel) {
                networkPanel
            }
        }
        .padding(.horizontal)
        .padding(.vertical, 12)
        .background(.regularMaterial)
    }

    private var networkPanel: some View {
        VStack(spacing: 0) {
            HStack {
                VStack(alignment: .leading, spacing: 2) {
                    Text("P2P Network")
                        .font(.headline)
                    Text(networkStatus)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .multilineTextAlignment(.leading)
                }

                Spacer()

                Button("Done") {
                    showingNetworkPanel = false
                }
            }
            .padding()
            .background(.regularMaterial)

            VStack(alignment: .leading, spacing: 16) {
                HStack {
                    Button("Start") { startNetwork() }
                    Button("Stop") { stopNetwork() }
                    Button("Refresh Logs") { refreshNetworkSnapshot() }
                }
                ScrollView {
                    Text(networkLogs.isEmpty ? "No P2P logs yet." : networkLogs)
                        .font(.system(.footnote, design: .monospaced))
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .textSelection(.enabled)
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)
                Divider()
                Text(rustHello())
                Text(String(rustAdd(a: 10, b: 32)))
            }
            .padding()
            .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background(.background)
    }

    private func startNetwork() {
        DispatchQueue.global(qos: .userInitiated).async {
            let status = p2pNetworkStart()
            let logs = p2pNetworkLogs()
            DispatchQueue.main.async {
                networkStatus = status
                networkLogs = logs
            }
        }
    }

    private func stopNetwork() {
        DispatchQueue.global(qos: .userInitiated).async {
            let status = p2pNetworkStop()
            let logs = p2pNetworkLogs()
            DispatchQueue.main.async {
                networkStatus = status
                networkLogs = logs
            }
        }
    }

    private func refreshNetworkSnapshot() {
        DispatchQueue.global(qos: .userInitiated).async {
            let status = p2pNetworkStatus()
            let logs = p2pNetworkLogs()
            DispatchQueue.main.async {
                networkStatus = status
                networkLogs = logs
            }
        }
    }
}

#Preview {
    ContentView()
}

//
//  Universal_AppApp.swift
//  Shared
//
//  Created by Can Balkaya on 12/10/20.
//

import RustyLib
import SwiftUI

@main
struct Universal_AppApp: App {
    var body: some Scene {
        WindowGroup {
            AppLifecycleView()
        }
    }
}

private struct AppLifecycleView: View {
    @Environment(\.scenePhase) private var scenePhase
    @State private var didStartNetwork = false

    var body: some View {
        MainView()
            .onAppear {
                startNetworkIfNeeded()
            }
            .onChange(of: scenePhase) { phase in
                switch phase {
                case .active:
                    startNetworkIfNeeded()
                case .background:
                    stopNetwork()
                case .inactive:
                    break
                @unknown default:
                    break
                }
            }
    }

    private func startNetworkIfNeeded() {
        guard !didStartNetwork else { return }
        didStartNetwork = true
        _ = p2pNetworkStart()
    }

    private func stopNetwork() {
        _ = p2pNetworkStop()
        didStartNetwork = false
    }
}

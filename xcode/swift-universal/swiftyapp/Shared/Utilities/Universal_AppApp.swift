//
//  Universal_AppApp.swift
//  Shared
//
//  Created by Can Balkaya on 12/10/20.
//

import Darwin
import RustyLib
import SwiftUI

@main
struct Universal_AppApp: App {
    @StateObject private var appState = AppState()

    var body: some Scene {
        WindowGroup {
            AppLifecycleView()
                .environmentObject(appState)
        }
    }
}

final class AppState: ObservableObject {
    @Published var privateKey: String = ""
}

private struct AppLifecycleView: View {
    @EnvironmentObject private var appState: AppState
    @Environment(\.scenePhase) private var scenePhase
    @State private var didStartNetwork = false

    var body: some View {
        MainView()
            .onAppear {
                applyPrivateKeyEnvironment()
                startNetworkIfNeeded()
            }
            .onChange(of: appState.privateKey) { _ in
                applyPrivateKeyEnvironment()
                restartNetworkIfNeeded()
            }
            .onChange(of: scenePhase) { phase in
                switch phase {
                case .active:
                    applyPrivateKeyEnvironment()
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

    private func restartNetworkIfNeeded() {
        guard didStartNetwork else { return }
        _ = p2pNetworkStop()
        didStartNetwork = false
        startNetworkIfNeeded()
    }

    private func stopNetwork() {
        _ = p2pNetworkStop()
        didStartNetwork = false
    }

    private func applyPrivateKeyEnvironment() {
        if appState.privateKey.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
            unsetenv("GNOSTR_NSEC")
        } else {
            appState.privateKey.withCString { pointer in
                setenv("GNOSTR_NSEC", pointer, 1)
            }
        }
    }
}

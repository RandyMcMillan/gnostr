import Combine
import RustyLib
import SwiftUI

struct P2PServicesView: View {
    @State private var networkStatus = p2pNetworkStatus()
    @State private var networkLogs = p2pNetworkLogs()

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("P2P Services")
                .font(.title2)
                .bold()

            Text(networkStatus)
                .font(.subheadline)
                .foregroundStyle(.secondary)

            HStack(spacing: 10) {
                Button("Start") {
                    _ = p2pNetworkStart()
                    refreshNetworkSnapshot()
                }

                Button("Stop") {
                    _ = p2pNetworkStop()
                    refreshNetworkSnapshot()
                }

                Button("Refresh") {
                    refreshNetworkSnapshot()
                }
            }
            .buttonStyle(.borderedProminent)

            ScrollView {
                Text(networkLogs.isEmpty ? "No P2P logs yet." : networkLogs)
                    .font(.system(.footnote, design: .monospaced))
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .textSelection(.enabled)
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)
        }
        .padding()
        .navigationTitle("P2P")
        .onAppear {
            refreshNetworkSnapshot()
        }
        .onReceive(Timer.publish(every: 1.0, on: .main, in: .common).autoconnect()) { _ in
            refreshNetworkSnapshot()
        }
    }

    private func refreshNetworkSnapshot() {
        networkStatus = p2pNetworkStatus()
        networkLogs = p2pNetworkLogs()
    }
}

struct P2PServicesView_Previews: PreviewProvider {
    static var previews: some View {
        P2PServicesView()
    }
}

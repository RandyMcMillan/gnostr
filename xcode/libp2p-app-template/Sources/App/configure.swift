import LibP2P
import LibP2PNoise
import LibP2PYAMUX
import LibP2PDCUtR
import LibP2PMDNS
import LibP2PKadDHT

// configures your application
public func configure(_ app: Application) async throws {
    
    // We can specify the global log level here
    app.logger.logLevel = .notice

    // Configure your networking stack...
    app.security.use(.noise)
    app.muxers.use(.yamux)
    app.dcutr.use(.dcutr)
    app.discovery.use(.mdns)
    app.discovery.use(.kadDHT)
    
    // Bind to all interfaces so discovery can reach peers on the local network.
    app.listen(.tcp(host: "0.0.0.0", port: listenPort()))
    
    // Add a custom command
    app.asyncCommands.use(Cowsay(), as: "cowsay")
    
    // register routes
    try routes(app)
    
    app.eventLoopGroup.next().scheduleTask(in: .milliseconds(100)) {
        for address in app.listenAddresses {
            let fullAddress = try address.encapsulate(proto: .p2p, address: app.peerID.b58String)
            app.logger.notice("Libp2p listening at \(fullAddress)")
        }
    }
}

fileprivate func listenPort() -> Int {
    if let value = ProcessInfo.processInfo.environment["P2P_LISTEN_PORT"],
       let port = Int(value),
       port > 0 {
        return port
    }
    return 10000
}

/// An example of a custom command you can add to your app
///
/// Execute the `cowsay` cmd by running
/// ```
/// swift run App cowsay "Mmooo" --eyes "👀" --tongue "👅"
/// ```
struct Cowsay: AsyncCommand {
    struct Signature: CommandSignature {
        @Argument(name: "message")
        var message: String

        @Option(name: "eyes", short: "e")
        var eyes: String?

        @Option(name: "tongue", short: "t")
        var tongue: String?
    }

    var help: String {
        "Generates ASCII picture of a cow with a message."
    }

    func run(using context: CommandContext, signature: Signature) async throws {
        let eyes = signature.eyes ?? "oo"
        let tongue = signature.tongue ?? "  "
        let cow = #"""
          < $M >
                  \   ^__^
                   \  ($E)\_______
                      (__)\       )\/\
                       $T ||----w |
                          ||     ||
        """#.replacingOccurrences(of: "$M", with: signature.message)
            .replacingOccurrences(of: "$E", with: eyes)
            .replacingOccurrences(of: "$T", with: tongue)
        context.console.print(cow)
    }
}

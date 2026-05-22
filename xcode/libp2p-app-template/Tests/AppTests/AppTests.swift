import LibP2P
import Testing

@testable import App

@Suite("App Tests", .serialized)
struct AppTests {

    // This is an example of how you can test various aspects of your libp2p app
    @Test func testExample() async throws {
        // Init our app
        let app = try await Application.make(.testing, peerID: .ephemeral())
        // Configure it
        try await configure(app)
        app.environment.arguments = ["libp2p"]
        // Start the app
        try await app.startup()
        
        // Ensure that the application is running in the `testing` environment
        #expect(app.environment == .testing)
        // Ensure that our custom cmd was registered
        #expect(
            app.asyncCommands.commands.contains(where: { key, val in
                key == "cowsay"
            })
        )
        // Ensure that the "echo" route was registered
        #expect(app.routes.all.contains(where: { route in
            route.description == "/echo/1.0.0"
        }))
        // Sleep for a bit
        try await Task.sleep(nanoseconds: 50_000)
        
        // Shutdown the app
        try await app.asyncShutdown()
    }
}

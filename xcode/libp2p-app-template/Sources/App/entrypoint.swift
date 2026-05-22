import Foundation
import LibP2P
import Logging
import NIOCore
import NIOPosix

@main
enum Entrypoint {
    static func main() async throws {
        // Determine the environment based on the executable being ran (testing, development or production)
        var env = try Environment.detect()
        try ensureDevelopmentEnvironmentFileExists(for: env)
        try ensurePeerIDStorageDirectoryExists()

        // Set up our logger
        try LoggingSystem.bootstrap(from: &env)

        // Create a persistent PeerID seeded from the development env password.
        let peerID: KeyPairFile = .persistent(
            type: .Ed25519,
            encryptedWith: .envKey,
            storedAt: .filePath(FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first!)
        )

        // Instantiate our libp2p app
        let app = try await makeApplication(env: env, peerID: peerID)

        // This attempts to install NIO as the Swift Concurrency global executor.
        // You can enable it if you'd like to reduce the amount of context switching between NIO and Swift Concurrency.
        // Note: this has caused issues with some libraries that use `.wait()` and cleanly shutting down.
        // If enabled, you should be careful about calling async functions before this point as it can cause assertion failures.
        // let executorTakeoverSuccess = NIOSingletons.unsafeTryInstallSingletonPosixEventLoopGroupAsConcurrencyGlobalExecutor()
        // app.logger.debug("Tried to install SwiftNIO's EventLoopGroup as Swift's global concurrency executor", metadata: ["success": .stringConvertible(executorTakeoverSuccess)])

        do {
            try await configure(app)
            try await app.execute()
        } catch {
            app.logger.error("\(error)")
            try? await app.asyncShutdown()
            throw error
        }
        try await app.asyncShutdown()
    }

    private static func ensureDevelopmentEnvironmentFileExists(for env: Environment) throws {
        guard env.name == "development" else { return }

        let url = URL(fileURLWithPath: FileManager.default.currentDirectoryPath)
            .appendingPathComponent(".env.development")
        guard !FileManager.default.fileExists(atPath: url.path) else { return }

        try "PEERID_PASSWORD=development\n".write(to: url, atomically: true, encoding: .utf8)
    }

    private static func ensurePeerIDStorageDirectoryExists() throws {
        let url = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first!
        try FileManager.default.createDirectory(at: url, withIntermediateDirectories: true)
    }

    private static func makeApplication(env: Environment, peerID: KeyPairFile) async throws -> Application {
        do {
            return try await Application.make(env, peerID: peerID)
        } catch {
            switch error {
            case KeyPairFile.Error.unableToReadKeyPairFile,
                KeyPairFile.Error.unableToDecryptKeyFile:
                let storageURL = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first!
                let keyFilePath = storageURL.appendingPathComponent(".peer-id-ed25519.\(env.name)").path
                try? FileManager.default.removeItem(atPath: keyFilePath)
                return try await Application.make(env, peerID: peerID)
            default:
                throw error
            }
        }
    }
}

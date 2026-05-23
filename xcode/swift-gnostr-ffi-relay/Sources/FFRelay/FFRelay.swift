import Foundation
import Darwin

private struct RustEnvelope<T: Decodable>: Decodable {
    let ok: Bool
    let data: T?
    let error: String?
}

private typealias RustStringFn = @convention(c) (UnsafePointer<CChar>) -> UnsafeMutablePointer<CChar>?
private typealias RustRelayEndpointFn = @convention(c) (UnsafePointer<CChar>, UInt16) -> UnsafeMutablePointer<CChar>?
private typealias RustFreeFn = @convention(c) (UnsafeMutablePointer<CChar>) -> Void

public struct RelayConfiguration: Codable, Hashable, Sendable {
    public var logging: String
    public var configFilePath: String

    public init(logging: String = "info", configFilePath: String = ".gnostr/relay.toml") {
        self.logging = logging
        self.configFilePath = configFilePath
    }

    private enum CodingKeys: String, CodingKey {
        case logging
        case configFilePath = "config_file_path"
    }

    public static func rustDefault() -> RelayConfiguration? {
        RustRelayBridge.shared.defaultConfiguration()
    }
}

public struct RelayProcessState: Codable, Hashable, Sendable {
    public var running: Bool
    public var pid: UInt32?
    public var message: String
    public var diskUsageBytes: UInt64?

    public init(running: Bool, pid: UInt32? = nil, message: String, diskUsageBytes: UInt64? = nil) {
        self.running = running
        self.pid = pid
        self.message = message
        self.diskUsageBytes = diskUsageBytes
    }

    private enum CodingKeys: String, CodingKey {
        case running
        case pid
        case message
        case diskUsageBytes = "disk_usage_bytes"
    }
}

public struct RelayDiscoveryEntry: Codable, Hashable, Sendable {
    public var url: String
    public var contact: String?
    public var description: String?
    public var name: String?
    public var pingMs: UInt64?
    public var software: String?
    public var version: String?
    public var supportedNips: [Int]
    public var supportedNipExtensions: [String]
    public var sourceNips: [Int]

    public init(
        url: String,
        contact: String? = nil,
        description: String? = nil,
        name: String? = nil,
        pingMs: UInt64? = nil,
        software: String? = nil,
        version: String? = nil,
        supportedNips: [Int] = [],
        supportedNipExtensions: [String] = [],
        sourceNips: [Int] = []
    ) {
        self.url = url
        self.contact = contact
        self.description = description
        self.name = name
        self.pingMs = pingMs
        self.software = software
        self.version = version
        self.supportedNips = supportedNips
        self.supportedNipExtensions = supportedNipExtensions
        self.sourceNips = sourceNips
    }

    private enum CodingKeys: String, CodingKey {
        case url
        case contact
        case description
        case name
        case pingMs = "ping_ms"
        case software
        case version
        case supportedNips = "supported_nips"
        case supportedNipExtensions = "supported_nip_extensions"
        case sourceNips = "source_nips"
    }
}

public final class RelayClient: @unchecked Sendable {
    public init() {}

    public func defaultConfiguration() -> RelayConfiguration? {
        RelayConfiguration.rustDefault()
    }

    public func listenEndpoint(host: String, port: UInt16) -> String {
        RelayEndpoints.listenEndpoint(host: host, port: port)
    }
}

public enum RelayEndpoints {
    public static func listenEndpoint(host: String, port: UInt16) -> String {
        if let bridged = RustRelayBridge.shared.listenEndpoint(host: host, port: port) {
            return bridged
        }
        return "ws://\(host):\(port)"
    }
}

public final class RustRelayBridge: @unchecked Sendable {
    public static let shared = RustRelayBridge()

    private let handle: UnsafeMutableRawPointer?
    private let defaultConfigurationFn: RustStringFn?
    private let listenEndpointFn: RustRelayEndpointFn?
    private let freeFn: RustFreeFn?

    private init() {
        self.handle = Self.openLibrary()
        if let handle {
            self.defaultConfigurationFn = Self.loadSymbol("relay_default_configuration_json", from: handle)
            self.listenEndpointFn = Self.loadSymbol("relay_listen_endpoint_json", from: handle)
            self.freeFn = Self.loadSymbol("relay_string_free", from: handle)
        } else {
            self.defaultConfigurationFn = nil
            self.listenEndpointFn = nil
            self.freeFn = nil
        }
    }

    public var isAvailable: Bool {
        self.handle != nil
            && self.defaultConfigurationFn != nil
            && self.listenEndpointFn != nil
            && self.freeFn != nil
    }

    public func defaultConfiguration() -> RelayConfiguration? {
        guard let json: String = self.callString(self.defaultConfigurationFn, input: "") else {
            return nil
        }
        guard let data = json.data(using: .utf8),
              let defaults = try? JSONDecoder().decode(RelayConfiguration.self, from: data) else {
            return nil
        }
        return defaults
    }

    public func listenEndpoint(host: String, port: UInt16) -> String? {
        guard let fn = self.listenEndpointFn else { return nil }
        let hostCString = host.cString(using: .utf8)!
        return hostCString.withUnsafeBufferPointer { buffer in
            guard let base = buffer.baseAddress else { return nil }
            guard let rawResult = fn(base, port) else { return nil }
            defer { self.freeFn?(rawResult) }
            return Self.decodeEnvelopeString(rawResult)
        }
    }

    private func callString(_ fn: RustStringFn?, input: String) -> String? {
        guard let fn else { return nil }
        let inputCString = input.cString(using: .utf8)!
        return inputCString.withUnsafeBufferPointer { buffer in
            guard let base = buffer.baseAddress else { return nil }
            guard let rawResult = fn(base) else { return nil }
            defer { self.freeFn?(rawResult) }
            return Self.decodeEnvelopeString(rawResult)
        }
    }

    private static func decodeEnvelopeString(_ rawResult: UnsafeMutablePointer<CChar>) -> String? {
        let json = String(cString: rawResult)
        guard let data = json.data(using: .utf8) else { return nil }
        let envelope = try? JSONDecoder().decode(RustEnvelope<String>.self, from: data)
        guard let envelope, envelope.ok, let value = envelope.data else { return nil }
        return value
    }

    private static func openLibrary() -> UnsafeMutableRawPointer? {
        let env = ProcessInfo.processInfo.environment
        let candidates: [String] = {
            if let explicit = env["GNOSTR_RELAY_FFI_LIBRARY"], !explicit.isEmpty {
                return [explicit]
            }
            return [
                "Rust/relay-ffi/target/debug/librelay_ffi.dylib",
                "Rust/relay-ffi/target/release/librelay_ffi.dylib",
            ]
        }()

        for candidate in candidates {
            if let handle = dlopen(candidate, RTLD_NOW | RTLD_LOCAL) {
                return handle
            }
        }

        return nil
    }

    private static func loadSymbol<T>(_ name: String, from handle: UnsafeMutableRawPointer) -> T? {
        guard let symbol = dlsym(handle, name) else { return nil }
        return unsafeBitCast(symbol, to: T.self)
    }
}

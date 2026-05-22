import Foundation
#if canImport(Darwin)
import Darwin
#endif

private struct RustEnvelope<T: Decodable>: Decodable {
    let ok: Bool
    let data: T?
    let error: String?
}

private typealias RustStringFn = @convention(c) (UnsafePointer<CChar>) -> UnsafeMutablePointer<CChar>?
private typealias RustFreeFn = @convention(c) (UnsafeMutablePointer<CChar>) -> Void

public final class RustCrawlerBridge: @unchecked Sendable {
    public static let shared = RustCrawlerBridge()

    private let handle: UnsafeMutableRawPointer?
    private let buildQueryFn: RustStringFn?
    private let websocketHttpURLFn: RustStringFn?
    private let roundtripRelayMetadataFn: RustStringFn?
    private let freeFn: RustFreeFn?

    private init() {
        self.handle = Self.openLibrary()
        if let handle {
            self.buildQueryFn = Self.loadSymbol("crawler_build_gnostr_query_json", from: handle)
            self.websocketHttpURLFn = Self.loadSymbol("crawler_websocket_http_url_json", from: handle)
            self.roundtripRelayMetadataFn = Self.loadSymbol("crawler_roundtrip_relay_metadata_json", from: handle)
            self.freeFn = Self.loadSymbol("crawler_string_free", from: handle)
        } else {
            self.buildQueryFn = nil
            self.websocketHttpURLFn = nil
            self.roundtripRelayMetadataFn = nil
            self.freeFn = nil
        }
    }

    public var isAvailable: Bool {
        self.handle != nil
            && self.buildQueryFn != nil
            && self.websocketHttpURLFn != nil
            && self.roundtripRelayMetadataFn != nil
            && self.freeFn != nil
    }

    public func buildGnostrQuery(_ parameters: CrawlerQueryParameters) throws -> String? {
        guard let buildQueryFn else { return nil }
        let encoder = JSONEncoder()
        encoder.keyEncodingStrategy = .convertToSnakeCase
        guard let data = try? encoder.encode(parameters), let json = String(data: data, encoding: .utf8) else {
            throw CocoaError(.coderInvalidValue)
        }
        return self.callString(buildQueryFn, input: json)
    }

    public func websocketHTTPURL(_ url: String) -> String? {
        self.callString(self.websocketHttpURLFn, input: url)
    }

    public func normalize(_ relay: RelayMetadata) throws -> RelayMetadata {
        guard self.isAvailable, let normalized: RelayMetadata = self.callRoundTrip(self.roundtripRelayMetadataFn, value: relay) else {
            return relay
        }
        return normalized
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

    private func callRoundTrip<T: Codable>(_ fn: RustStringFn?, value: T) -> T? {
        guard let fn else { return nil }
        let encoder = JSONEncoder()
        encoder.keyEncodingStrategy = .convertToSnakeCase
        guard let data = try? encoder.encode(value), let json = String(data: data, encoding: .utf8) else {
            return nil
        }
        let inputCString = json.cString(using: .utf8)!
        return inputCString.withUnsafeBufferPointer { buffer in
            guard let base = buffer.baseAddress else { return nil }
            guard let rawResult = fn(base) else { return nil }
            defer { self.freeFn?(rawResult) }
            let responseJSON = String(cString: rawResult)
            guard let data = responseJSON.data(using: .utf8) else { return nil }
            let envelope = try? JSONDecoder().decode(RustEnvelope<String>.self, from: data)
            guard let envelope, envelope.ok, let payload = envelope.data else { return nil }
            guard let payloadData = payload.data(using: .utf8) else { return nil }
            return try? JSONDecoder().decode(T.self, from: payloadData)
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
            if let explicit = env["GNOSTR_CRAWLER_FFI_LIBRARY"], !explicit.isEmpty {
                return [explicit]
            }
            return [
                "Rust/crawler-ffi/target/debug/libcrawler_ffi.dylib",
                "Rust/crawler-ffi/target/release/libcrawler_ffi.dylib",
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

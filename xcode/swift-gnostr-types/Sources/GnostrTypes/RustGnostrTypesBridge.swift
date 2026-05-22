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
private typealias RustNoteFn = @convention(c) (UnsafePointer<CChar>, UnsafePointer<CChar>, UInt8) -> UnsafeMutablePointer<CChar>?
private typealias RustFreeFn = @convention(c) (UnsafeMutablePointer<CChar>) -> Void

public final class RustGnostrTypesBridge: @unchecked Sendable {
    public static let shared = RustGnostrTypesBridge()

    private let handle: UnsafeMutableRawPointer?
    private let gitNoteEventIDFn: RustStringFn?
    private let gitNoteTagsFn: RustStringFn?
    private let generateGitNoteEventFn: RustNoteFn?
    private let freeFn: RustFreeFn?

    private init() {
        self.handle = Self.openLibrary()
        if let handle {
            self.gitNoteEventIDFn = Self.loadSymbol("gnostr_types_git_note_event_id_json", from: handle)
            self.gitNoteTagsFn = Self.loadSymbol("gnostr_types_git_note_tags_json", from: handle)
            self.generateGitNoteEventFn = Self.loadSymbol("gnostr_types_generate_git_note_event_json", from: handle)
            self.freeFn = Self.loadSymbol("gnostr_types_string_free", from: handle)
        } else {
            self.gitNoteEventIDFn = nil
            self.gitNoteTagsFn = nil
            self.generateGitNoteEventFn = nil
            self.freeFn = nil
        }
    }

    public var isAvailable: Bool {
        self.handle != nil
            && self.gitNoteEventIDFn != nil
            && self.gitNoteTagsFn != nil
            && self.generateGitNoteEventFn != nil
            && self.freeFn != nil
    }

    public func gitNoteEventID(commitID: String) -> String? {
        self.callString(self.gitNoteEventIDFn, input: commitID)
    }

    public func gitNoteTags(note: GitNote) throws -> [Tag] {
        let encoder = JSONEncoder()
        encoder.keyEncodingStrategy = .convertToSnakeCase
        let noteData = try encoder.encode(note)
        guard let noteJSON = String(data: noteData, encoding: .utf8) else {
            throw CocoaError(.coderInvalidValue)
        }
        guard let tagsJSON: String = self.callString(self.gitNoteTagsFn, input: noteJSON) else {
            throw CocoaError(.coderReadCorrupt)
        }
        guard let data = tagsJSON.data(using: .utf8) else {
            throw CocoaError(.coderReadCorrupt)
        }
        return try JSONDecoder().decode([Tag].self, from: data)
    }

    public func generateGitNoteEvent(note: GitNote, privateKeyHex: String, powTargetBits: UInt8 = 0) throws -> Event {
        let encoder = JSONEncoder()
        encoder.keyEncodingStrategy = .convertToSnakeCase
        let noteData = try encoder.encode(note)
        guard let noteJSON = String(data: noteData, encoding: .utf8) else {
            throw CocoaError(.coderInvalidValue)
        }
        guard let eventJSON: String = self.callString(
            self.generateGitNoteEventFn,
            input: noteJSON,
            extraInput: privateKeyHex,
            powTargetBits: powTargetBits
        ) else {
            throw CocoaError(.coderReadCorrupt)
        }
        guard let data = eventJSON.data(using: .utf8) else {
            throw CocoaError(.coderReadCorrupt)
        }
        return try JSONDecoder().decode(Event.self, from: data)
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

    private func callString(_ fn: RustNoteFn?, input: String, extraInput: String, powTargetBits: UInt8) -> String? {
        guard let fn else { return nil }
        let noteCString = input.cString(using: .utf8)!
        let keyCString = extraInput.cString(using: .utf8)!
        return noteCString.withUnsafeBufferPointer { noteBuffer in
            keyCString.withUnsafeBufferPointer { keyBuffer in
                guard let noteBase = noteBuffer.baseAddress, let keyBase = keyBuffer.baseAddress else {
                    return nil
                }
                guard let rawResult = fn(noteBase, keyBase, powTargetBits) else { return nil }
                defer { self.freeFn?(rawResult) }
                return Self.decodeEnvelopeString(rawResult)
            }
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
            if let explicit = env["GNOSTR_TYPES_FFI_LIBRARY"], !explicit.isEmpty {
                return [explicit]
            }
            return [
                "Rust/gnostr-types-ffi/target/debug/libgnostr_types_ffi.dylib",
                "Rust/gnostr-types-ffi/target/release/libgnostr_types_ffi.dylib",
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

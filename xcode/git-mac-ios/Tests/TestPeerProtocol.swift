import XCTest
@testable import Git

final class TestPeerProtocol: XCTestCase {
    func testEnvelopeRoundTrip() throws {
        let refs = Refs(
            repository: "demo",
            branch: "main",
            reference: "refs/heads/main",
            commit: "abc123",
            remote: "example.com/demo.git",
            peer: "peer-1"
        )
        let payload = try JSONEncoder().encode(refs)
        let envelope = Envelope(kind: "refs", repository: refs.repository, payload: String(decoding: payload, as: UTF8.self), peer: refs.peer)
        let data = try JSONEncoder().encode(envelope)
        let decoded = try JSONDecoder().decode(Envelope.self, from: data)
        XCTAssertEqual("refs", decoded.kind)
        XCTAssertEqual("demo", decoded.repository)
        XCTAssertEqual("peer-1", decoded.peer)
        XCTAssertEqual(refs.commit, try JSONDecoder().decode(Refs.self, from: decoded.payload!.data(using: .utf8)!).commit)
    }

    func testPushRequestCarriesPackBytes() throws {
        let request = PushRequest(
            repository: "demo",
            old: "old",
            new: "new",
            pack: Data([0xde, 0xad, 0xbe, 0xef]),
            peer: "peer-1"
        )
        let data = try JSONEncoder().encode(request)
        let decoded = try JSONDecoder().decode(PushRequest.self, from: data)
        XCTAssertEqual("demo", decoded.repository)
        XCTAssertEqual("old", decoded.old)
        XCTAssertEqual("new", decoded.new)
        XCTAssertEqual(Data([0xde, 0xad, 0xbe, 0xef]), Data(base64Encoded: decoded.pack))
    }

    private struct Envelope: Codable {
        let kind: String
        let repository: String?
        let payload: String?
        let peer: String?
    }

    private struct Refs: Codable {
        let repository: String
        let branch: String
        let reference: String
        let commit: String
        let remote: String
        let peer: String
    }

    private struct PushRequest: Codable {
        let repository: String
        let old: String
        let new: String
        let pack: String
        let peer: String

        init(repository: String, old: String, new: String, pack: Data, peer: String) {
            self.repository = repository
            self.old = old
            self.new = new
            self.pack = pack.base64EncodedString()
            self.peer = peer
        }
    }
}

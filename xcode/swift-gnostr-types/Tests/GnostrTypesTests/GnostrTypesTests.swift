import Foundation
import Testing
@testable import GnostrTypes

@Test func eventKindConstantsMatchRust() {
    #expect(NIP34.repoAnnouncementKind == 30617)
    #expect(NIP34.repoStateKind == 30618)
    #expect(NIP34.patchesKind == 1617)
    #expect(statusKinds() == [.gitStatusOpen, .gitStatusApplied, .gitStatusClosed, .gitStatusDraft])
}

@Test func gitNoteCodableRoundTrip() throws {
    let note = GitNote(
        noteID: "deadbeef",
        annotatedID: "cafebabe",
        notesRef: "refs/notes/commits",
        message: "hello",
        author: "alice",
        committer: "bob",
        committerTime: 1234
    )

    let data = try JSONEncoder().encode(note)
    let decoded = try JSONDecoder().decode(GitNote.self, from: data)
    #expect(decoded == note)
}

@Test func tagHelpersBuildExpectedFields() {
    let id = Id(hex: "0123")
    let key = PublicKey(hex: "abcd")

    #expect(Tag.event(id, marker: "root").fields == ["e", "0123", "root"])
    #expect(Tag.pubkey(key, relay: "wss://example.com", marker: "reply").fields == ["p", "abcd", "wss://example.com", "reply"])
}

@Test func leadingZeroBitsCountMatchesExamples() {
    #expect(getLeadingZeroBits([0x00, 0x00, 0x10]) == 19)
    #expect(getLeadingZeroBits([0xff]) == 0)
}

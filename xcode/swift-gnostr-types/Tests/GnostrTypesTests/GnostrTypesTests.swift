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

@Test func rustJsonShapesMatch() throws {
    let event = Event(
        id: Id(hex: "deadbeef"),
        pubkey: PublicKey(hex: "cafebabe"),
        createdAt: Unixtime(1234),
        kind: .patches,
        sig: Signature(hex: "facefeed"),
        content: "hello",
        tags: [Tag.event(Id(hex: "0123"), marker: "root")]
    )

    let data = try JSONEncoder().encode(event)
    let json = String(decoding: data, as: UTF8.self)
    #expect(json.contains(#""created_at":1234"#))
    #expect(json.contains(#""tags":[["e","0123","root"]]"#))
    #expect(json.contains(#""id":"deadbeef""#))
    #expect(json.contains(#""sig":"facefeed""#))
}

@Test func rustBridgeRoundTripsWhenAvailable() throws {
    guard NIP34.rustBridgeAvailable else { return }

    let note = GitNote(
        noteID: "deadbeef",
        annotatedID: "cafebabe",
        notesRef: "refs/notes/commits",
        message: "hello from rust",
        author: "alice",
        committer: "bob",
        committerTime: 1234
    )

    #expect(NIP34.rustGitNoteEventID(commitID: note.annotatedID) != nil)

    let tags = try NIP34.rustGitNoteTags(note: note)
    #expect(tags.contains(where: { $0.tagName() == "commit" }))
    #expect(tags.contains(where: { $0.tagName() == "notes-ref" }))

    let event = try NIP34.rustGenerateGitNoteEvent(
        note: note,
        privateKeyHex: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    )
    #expect(event.kind == .patches)
    #expect(event.content == note.message)
    #expect(event.tags.contains(where: { $0.tagName() == "commit" }))
}

@Test func leadingZeroBitsCountMatchesExamples() {
    #expect(getLeadingZeroBits([0x00, 0x00, 0x10]) == 19)
    #expect(getLeadingZeroBits([0xff]) == 0)
}

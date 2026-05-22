import Foundation
import Testing
@testable import FFIKitchenSink

@Test func packageReexportsCoreTypes() {
    let note = GitNote(
        noteID: "deadbeef",
        annotatedID: "cafebabe",
        notesRef: "refs/notes/commits",
        message: "hello",
        author: "alice",
        committer: "bob",
        committerTime: 1234
    )
    #expect(note.noteID == "deadbeef")
    #expect(FFIKitchenSink.asyncGitEventKinds().contains(.patches))
    _ = FFIKitchenSink.crawlerClient()
    _ = FFIKitchenSink.relayClient()
}

@Test func bridgeNamespaceIsReachable() {
    _ = FFIKitchenSink.crawlerBridge
    _ = FFIKitchenSink.gnostrTypesBridge
    _ = FFIKitchenSink.relayBridge
}

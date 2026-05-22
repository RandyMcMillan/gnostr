import Foundation

public struct GitNote: Codable, Hashable, Sendable {
    public let noteID: String
    public let annotatedID: String
    public let notesRef: String?
    public let message: String
    public let author: String
    public let committer: String
    public let committerTime: Int64

    public init(
        noteID: String,
        annotatedID: String,
        notesRef: String?,
        message: String,
        author: String,
        committer: String,
        committerTime: Int64
    ) {
        self.noteID = noteID
        self.annotatedID = annotatedID
        self.notesRef = notesRef
        self.message = message
        self.author = author
        self.committer = committer
        self.committerTime = committerTime
    }
}

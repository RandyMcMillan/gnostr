import Foundation

public struct AsyncGitRepository: Sendable {
    public let url: URL

    public init(url: URL) {
        self.url = url
    }

    public func repoState() -> RepoState {
        let gitDir = self.url.appendingPathComponent(".git", isDirectory: true)

        if FileManager.default.fileExists(atPath: gitDir.appendingPathComponent("MERGE_HEAD").path) {
            return .merge
        }
        if FileManager.default.fileExists(atPath: gitDir.appendingPathComponent("rebase-merge").path)
            || FileManager.default.fileExists(atPath: gitDir.appendingPathComponent("rebase-apply").path)
        {
            return .rebase
        }
        if FileManager.default.fileExists(atPath: gitDir.appendingPathComponent("REVERT_HEAD").path) {
            return .revert
        }
        if FileManager.default.fileExists(atPath: gitDir.path) {
            return .clean
        }
        return .other
    }

    public func defaultNotesRef() -> String {
        "refs/notes/commits"
    }

    public func gitNote(
        noteID: String,
        annotatedID: String,
        message: String,
        author: String,
        committer: String,
        committerTime: Int64,
        notesRef: String? = nil
    ) -> GitNote {
        GitNote(
            noteID: noteID,
            annotatedID: annotatedID,
            notesRef: notesRef,
            message: message,
            author: author,
            committer: committer,
            committerTime: committerTime
        )
    }
}

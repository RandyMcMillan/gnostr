import Foundation

public enum RepoState: String, Codable, Sendable {
    case clean
    case merge
    case rebase
    case revert
    case other
}

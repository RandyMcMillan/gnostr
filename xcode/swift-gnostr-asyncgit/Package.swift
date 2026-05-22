// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "swift-gnostr-asyncgit",
    platforms: [
        .macOS(.v10_15),
        .iOS(.v13),
    ],
    products: [
        .library(
            name: "AsyncGit",
            targets: ["AsyncGit"]
        ),
    ],
    dependencies: [],
    targets: [
        .target(
            name: "AsyncGit"
        ),
        .testTarget(
            name: "AsyncGitTests",
            dependencies: ["AsyncGit"]
        ),
    ]
)

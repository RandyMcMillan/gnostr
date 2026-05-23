// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "swift-gnostr-ff-relay",
    platforms: [
        .macOS(.v10_15),
        .iOS(.v13),
    ],
    products: [
        .library(
            name: "FFRelay",
            targets: ["FFRelay"]
        ),
    ],
    targets: [
        .target(
            name: "FFRelay"
        ),
        .testTarget(
            name: "FFRelayTests",
            dependencies: ["FFRelay"]
        ),
    ]
)

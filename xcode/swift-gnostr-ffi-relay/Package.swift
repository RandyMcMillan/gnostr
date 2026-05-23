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
        .library(
            name: "RelayGUI",
            targets: ["RelayGUI"]
        ),
    ],
    targets: [
        .target(
            name: "FFRelay"
        ),
        .target(
            name: "RelayGUI",
            dependencies: ["FFRelay"]
        ),
        .testTarget(
            name: "FFRelayTests",
            dependencies: ["FFRelay"]
        ),
        .testTarget(
            name: "RelayGUITests",
            dependencies: ["RelayGUI"]
        ),
    ]
)

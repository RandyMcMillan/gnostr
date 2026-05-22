// swift-tools-version: 6.0
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "libp2p-app-template",
    platforms: [
        .macOS(.v13),
        .iOS(.v13)
    ],
    dependencies: [
        // Dependencies declare other packages that this package depends on.
        .package(path: "../swift-libp2p"),
        // Noise Security Module
        .package(path: "../swift-libp2p-noise"),
        // YAMUX Muxer Module
        .package(path: "../swift-libp2p-yamux"),
        // Direct Connection Upgrade through Relay
        .package(path: "../swift-libp2p-dcutr"),
        // mDNS peer discovery
        .package(path: "../swift-libp2p-mdns"),
        // Kademlia peer discovery
        .package(path: "../swift-libp2p-kad-dht"),
    ],
    targets: [
        // Targets are the basic building blocks of a package. A target can define a module or a test suite.
        // Targets can depend on other targets in this package, and on products in packages this package depends on.
        .executableTarget(
            name: "App",
            dependencies: [
                .product(name: "LibP2P", package: "swift-libp2p"),
                .product(name: "LibP2PNoise", package: "swift-libp2p-noise"),
                .product(name: "LibP2PYAMUX", package: "swift-libp2p-yamux"),
                .product(name: "LibP2PDCUtR", package: "swift-libp2p-dcutr"),
                .product(name: "LibP2PMDNS", package: "swift-libp2p-mdns"),
                .product(name: "LibP2PKadDHT", package: "swift-libp2p-kad-dht"),
            ],
            swiftSettings: swiftSettings),
        .testTarget(
            name: "AppTests",
            dependencies: [
                .target(name: "App")
            ],
            swiftSettings: swiftSettings),
    ]
)

var swiftSettings: [SwiftSetting] { [
    .enableUpcomingFeature("ExistentialAny"),
] }

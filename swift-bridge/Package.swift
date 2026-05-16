// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "AVPlayerBridge",
    platforms: [
        .macOS(.v13)
    ],
    products: [
        .library(
            name: "AVPlayerBridge",
            type: .static,
            targets: ["AVPlayerBridge"])
    ],
    targets: [
        .target(
            name: "AVPlayerBridge",
            path: "Sources/AVPlayerBridge",
            publicHeadersPath: "include")
    ]
)

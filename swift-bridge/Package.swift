// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "AVPlayerBridge",
    platforms: [
        .macOS(.v12)
    ],
    products: [
        .library(
            name: "AVPlayerBridge",
            type: .static,
            targets: ["AVPlayerBridge"])
    ],
    targets: [
        .target(
            name: "AVPlayerABI",
            path: "Sources/AVPlayerABI",
            publicHeadersPath: "include"),
        .target(
            name: "AVPlayerObjCBridge",
            path: "Sources/AVPlayerObjCBridge",
            publicHeadersPath: "include"),
        .target(
            name: "AVPlayerBridge",
            dependencies: ["AVPlayerObjCBridge", "AVPlayerABI"],
            path: "Sources/AVPlayerBridge")
    ]
)

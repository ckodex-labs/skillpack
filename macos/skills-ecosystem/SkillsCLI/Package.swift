// swift-tools-version: 5.10
import PackageDescription

let package = Package(
    name: "SkillsCLI",
    platforms: [
        .macOS(.v13)
    ],
    dependencies: [
        .package(url: "https://github.com/apple/swift-argument-parser", from: "1.3.0"),
        .package(path: "../SkillsCore"),
    ],
    targets: [
        .executableTarget(
            name: "SkillsCLI",
            dependencies: [
                .product(name: "ArgumentParser", package: "swift-argument-parser"),
                .product(name: "SkillsCore", package: "SkillsCore"),
            ],
            path: "Sources/SkillsCLI",
            exclude: [
                "grpc-swift-config.json",
                "swift-protobuf-config.json",
                "Canonical.grpc.swift",
                "Canonical.pb.swift",
                "Skills.grpc.swift",
                "Skills.pb.swift",
                "SkillPackGRPCClient.swift",
                "DaemonClient.swift",
                "skillpack.grpc.swift",
                "skillpack.pb.swift",
            ]
        ),
    ]
)

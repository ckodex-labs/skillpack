// swift-tools-version: 5.10
import PackageDescription

let package = Package(
    name: "SkillsDaemon",
    platforms: [
        .macOS(.v13)
    ],
    dependencies: [
        .package(path: "../SkillsCore"),
        .package(url: "https://github.com/grpc/grpc-swift.git", from: "1.21.0"),
        .package(url: "https://github.com/apple/swift-protobuf.git", from: "1.26.0")
    ],
    targets: [
        .executableTarget(
            name: "SkillsDaemon",
            dependencies: [
                .product(name: "SkillsCore", package: "SkillsCore"),
                .product(name: "GRPC", package: "grpc-swift")
            ],
            exclude: [
                "grpc-swift-config.json",
                "swift-protobuf-config.json"
            ]
        ),
    ]
)

// swift-tools-version: 5.10
import PackageDescription

let package = Package(
    name: "SkillsCore",
    platforms: [
        .macOS(.v13),
        .iOS(.v16)
    ],
    products: [
        .library(
            name: "SkillsCore",
            targets: ["SkillsCore"]),
    ],
    targets: [
        .target(
            name: "SkillsCore"),
        .testTarget(
            name: "SkillsCoreTests",
            dependencies: ["SkillsCore"]),
    ]
)

// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "Hookmarks",
    platforms: [
        .macOS(.v13)
    ],
    dependencies: [],
    targets: [
        .executableTarget(
            name: "Hookmarks",
            dependencies: [],
            swiftSettings: [
                .unsafeFlags(["-suppress-warnings"], .when(configuration: .debug))
            ]
        ),
        .testTarget(
            name: "HookmarksTests",
            dependencies: ["Hookmarks"]
        )
    ]
)

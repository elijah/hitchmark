// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "Hitchmark",
    platforms: [
        .macOS(.v13)
    ],
    dependencies: [],
    targets: [
        .executableTarget(
            name: "Hitchmark",
            dependencies: [],
            swiftSettings: [
                .unsafeFlags(["-suppress-warnings"], .when(configuration: .debug))
            ]
        ),
        .testTarget(
            name: "HitchmarkTests",
            dependencies: ["Hitchmark"]
        )
    ]
)

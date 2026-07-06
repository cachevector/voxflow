// swift-tools-version: 5.9
import PackageDescription

let rustLibRelease = "../../target/release"
let rustLibDebug = "../../target/debug"

let rustLinkerSettings: [LinkerSetting] = [
    .linkedLibrary("voxflow_ffi"),
    .unsafeFlags(
        ["-L", rustLibRelease, "-Xlinker", "-rpath", "-Xlinker", rustLibRelease],
        .when(configuration: .release)
    ),
    .unsafeFlags(
        ["-L", rustLibDebug, "-Xlinker", "-rpath", "-Xlinker", rustLibDebug],
        .when(configuration: .debug)
    ),
]

let package = Package(
    name: "VoxFlow",
    platforms: [.macOS(.v13)],
    products: [
        .executable(name: "VoxFlow", targets: ["VoxFlow"]),
    ],
    targets: [
        .executableTarget(
            name: "VoxFlow",
            dependencies: ["VoxFlowCore"],
            path: "VoxFlow",
            linkerSettings: rustLinkerSettings
        ),
        .target(
            name: "VoxFlowCore",
            dependencies: ["VoxFlowFFI"],
            path: "VoxFlowCore",
            linkerSettings: rustLinkerSettings
        ),
        .systemLibrary(
            name: "VoxFlowFFI",
            path: "VoxFlowFFI"
        ),
    ]
)

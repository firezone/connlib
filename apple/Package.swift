// swift-tools-version:5.7

var package = Package(
  name: "Connlib",
  platforms: [
    .macOS(.v12),
    .iOS(.v15)
  ],
  products: [
    .library(
      name: "Connlib",
      targets: ["Connlib"]),
  ],
  dependencies: [],
  targets: [
    .target(
      name: "Connlib",
      dependencies: []),
    .testTarget(
      name: "ConnlibTests",
      dependencies: ["Connlib"]),
  ]
)

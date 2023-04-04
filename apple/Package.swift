// swift-tools-version:5.7

import PackageDescription

let package = Package(
  name: "Connlib",
  platforms: [
    .macOS(.v12),
    .iOS(.v15)
  ],
  products: [
    .library(
      name: "Connlib",
      targets: ["Connlib"])
  ],
  dependencies: [],
  targets: [
    .target(
      name: "Connlib",
      dependencies: [])
  ]
)

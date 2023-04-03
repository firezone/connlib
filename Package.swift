// swift-tools-version: 5.7
// This top-level Package.swift is needed because the Swift Package Manager does not support
// multiple Package.swift files in a single package. This file is only used to build the
// Swift Package Manager package. The actual package is contained in the apple/ subdirectory.
import PackageDescription

let package = Package(
  name: "ConnlibRoot",
  platforms: [
      .macOS(.v12),
      .iOS(.v15)
  ],
  products: [],
  dependencies: [
      .package(path: "./apple")
  ],
  targets: [
    // Targets are the basic building blocks of a package. A target can define a module or a test suite.
    // Targets can depend on other targets in this package, and on products in packages this package depends on.
    .target(
      name: "ConnlibRoot",
        dependencies: [
          .product(name: "Connlib", package: "Connlib")
        ])
  ]
)

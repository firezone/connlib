//
//  Adapter.swift
//  (c) 2023 Firezone, Inc.
//  LICENSE: Apache-2.0
//
import Foundation
import NetworkExtension
import os.log

public enum AdapterError: Error {
  /// Failure to perform an operation in such state.
  case invalidState

  /// Failure to set network settings.
  case setNetworkSettings(Error)
}

/// Enum representing internal state of the `WireGuardAdapter`
private enum State {
  /// The tunnel is stopped
  case stopped

  /// The tunnel is up and running
  case started(_ handle: WrappedSession)

  /// The tunnel is temporarily shutdown due to device going offline
  case temporaryShutdown
}

// Loosely inspired from WireGuardAdapter from WireGuardKit
public class Adapter {
  private let logger = Logger(subsystem: "dev.firezone.firezone", category: "packet-tunnel")

  // Initialize callback handler after the adapter is initialized,
  // just when the callback handler needs to be used
  private lazy var callbackHandler = CallbackHandler(adapter: self)

  /// Packet tunnel provider.
  private weak var packetTunnelProvider: NEPacketTunnelProvider?

  /// Network routes monitor.
  private var networkMonitor: NWPathMonitor?

  /// Private queue used to synchronize access to `WireGuardAdapter` members.
  private let workQueue = DispatchQueue(label: "FirezoneAdapterWorkQueue")

  /// Adapter state.
  private var state: State = .stopped
  private var interfaceAddresses: InterfaceAddresses?
  private var resources: [Resource] = []

  private var isTunnelStarted = false

  public init(with packetTunnelProvider: NEPacketTunnelProvider) {
    self.packetTunnelProvider = packetTunnelProvider
  }

  deinit {
    // Cancel network monitor
    networkMonitor?.cancel()

    // Shutdown the tunnel
    if case .started(let wrappedSession) = self.state {
      self.logger.log(level: .debug, "\(#function)")
      wrappedSession.disconnect()
    }
  }

  /// Start the tunnel tunnel.
  /// - Parameters:
  ///   - completionHandler: completion handler.
  public func start(portalURL: String, token: String, completionHandler: @escaping (AdapterError?) -> Void) throws {
    workQueue.async {
      guard case .stopped = self.state else {
        completionHandler(.invalidState)
        return
      }

      let networkMonitor = NWPathMonitor()
      networkMonitor.pathUpdateHandler = { [weak self] path in
        self?.didReceivePathUpdate(path: path)
      }
      networkMonitor.start(queue: self.workQueue)

      self.callbackHandler.delegate = self

      do {
        self.state = .started(
          WrappedSession.connect(
            portalURL,
            token,
            self.callbackHandler
          )
        )
        self.networkMonitor = networkMonitor
        self.isTunnelStarted = true
        let settings = self.generateNetworkSettings(interfaceAddresses: self.interfaceAddresses, resources: self.resources)
        try self.setNetworkSettings(settings)
        completionHandler(nil)
      } catch let error as AdapterError {
        networkMonitor.cancel()
        completionHandler(error)
      } catch {
        fatalError()
      }
    }
  }

  public func stop(completionHandler: @escaping (AdapterError?) -> Void) {
    workQueue.async {
      switch self.state {
      case .started(let wrappedSession):
        wrappedSession.disconnect()
      case .temporaryShutdown:
        break

      case .stopped:
        completionHandler(.invalidState)
        return
      }

      self.networkMonitor?.cancel()
      self.networkMonitor = nil

      self.state = .stopped

      completionHandler(nil)
    }
  }

  func generateNetworkSettings(interfaceAddresses: InterfaceAddresses?, resources: [Resource]) -> NEPacketTunnelNetworkSettings {

    // Interface addresses
    let ipv4InterfaceAddresses = [interfaceAddresses?.ipv4].compactMap { $0 }
    let ipv4SubnetMasks = ipv4InterfaceAddresses.map { _ in "255.255.255.255" }

    let ipv6InterfaceAddresses = [interfaceAddresses?.ipv6].compactMap { $0 }
    let ipv6SubnetMasks = ipv6InterfaceAddresses.map { _ in NSNumber(integerLiteral: 128) }

    // Routes
    var ipv4Routes: [NEIPv4Route] = []
    var ipv6Routes: [NEIPv6Route] = []

    for resource in resources {
      switch resource.resourceLocation {
      case .dns(_, let ipv4, let ipv6):
        if !ipv4.isEmpty {
          ipv4Routes.append(NEIPv4Route(destinationAddress: ipv4, subnetMask: "255.255.255.255"))
        }
        if !ipv6.isEmpty {
          ipv6Routes.append(NEIPv6Route(destinationAddress: ipv6, networkPrefixLength: 128))
        }
      case .cidr(let addressRange):
        ipv4Routes.append(NEIPv4Route(destinationAddress: "\(addressRange.maskedAddress())", subnetMask: "\(addressRange.subnetMask())"))
      }
    }

    // DNS
    let dnsSentinel = "1.1.1.1" // The destination IP that connlib will assign our DNS proxy to
    let dnsSettings = NEDNSSettings(servers: [dnsSentinel])
    dnsSettings.matchDomains = [""] // All DNS queries must first go through the tunnel's DNS

    // Put it together

    let ipv4Settings = NEIPv4Settings(addresses: ipv4InterfaceAddresses, subnetMasks: ipv4SubnetMasks)
    let ipv6Settings = NEIPv6Settings(addresses: ipv6InterfaceAddresses, networkPrefixLengths: ipv6SubnetMasks)
    ipv4Settings.includedRoutes = ipv4Routes
    ipv6Settings.includedRoutes = ipv6Routes

    let networkSettings = NEPacketTunnelNetworkSettings(tunnelRemoteAddress: "127.0.0.1")
    networkSettings.dnsSettings = dnsSettings
    networkSettings.ipv4Settings = ipv4Settings
    networkSettings.ipv6Settings = ipv6Settings

    // We can probably do better than this; see https://www.rfc-editor.org/info/rfc4821
    // But stick with something simple for now. 1280 is the minimum that will work for IPv6.
    networkSettings.mtu = NSNumber(value: 1280)

    return networkSettings
  }

  public func setNetworkSettings(_ networkSettings: NEPacketTunnelNetworkSettings) throws {
    var systemError: Error?
    let condition = NSCondition()

    // Activate the condition
    condition.lock()
    defer { condition.unlock() }

    self.packetTunnelProvider?.setTunnelNetworkSettings(networkSettings) { error in
      systemError = error
      condition.signal()
    }

    // Packet tunnel's `setTunnelNetworkSettings` times out in certain
    // scenarios & never calls the given callback.
    let setTunnelNetworkSettingsTimeout: TimeInterval = 5  // seconds

    if condition.wait(until: Date().addingTimeInterval(setTunnelNetworkSettingsTimeout)) {
      if let systemError = systemError {
        throw AdapterError.setNetworkSettings(systemError)
      }
    }
  }

  private func updateTunnelNetworkSettings() {
    guard isTunnelStarted else {
      return
    }
    let settings = generateNetworkSettings(interfaceAddresses: self.interfaceAddresses, resources: self.resources)
    workQueue.async {
      do {
        try self.setNetworkSettings(settings)
      } catch {
        self.logger.log(level: .debug, "setNetworkSettings failed: \(error)")
      }
    }
  }

  private func didReceivePathUpdate(path: Network.NWPath) {
    #if os(macOS)
      if case .started(let wrappedSession) = self.state {
        wrappedSession.bumpSockets()
      }
    #elseif os(iOS)
      switch self.state {
      case .started(let wrappedSession):
        if path.status == .satisfied {
          wrappedSession.disableSomeRoamingForBrokenMobileSemantics()
          wrappedSession.bumpSockets()
        } else {
          //self.logger.log(.debug, "Connectivity offline, pausing backend.")
          self.state = .temporaryShutdown
          wrappedSession.disconnect()
        }

      case .temporaryShutdown:
        guard path.status == .satisfied else { return }

        self.logger.log(level: .debug, "Connectivity online, resuming backend.")

        do {
          try self.setNetworkSettings(self.lastNetworkSettings!)

          self.state = .started(
            try WrappedSession.connect("http://localhost:4568", "test-token", self.callbackHandler)
          )
        } catch {
          self.logger.log(level: .debug, "Failed to restart backend: \(error.localizedDescription)")
        }

      case .stopped:
        // no-op
        break
      }
    #else
      #error("Unsupported")
    #endif
  }
}

extension Adapter: CallbackHandlerDelegate {
  public func didUpdateResources(_ resources: [Resource]) {
    self.resources = resources
    updateTunnelNetworkSettings()
  }

  public func didUpdateInterfaceAddresses(_ interfaceAddresses: InterfaceAddresses) {
    self.interfaceAddresses = interfaceAddresses
    updateTunnelNetworkSettings()
  }
}

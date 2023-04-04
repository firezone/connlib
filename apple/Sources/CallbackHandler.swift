//
//  Callbacks.swift
//  connlib
//
//  Created by Jamil Bou Kheir on 4/3/23.
//

import NetworkExtension
import os.log

public class CallbackHandler {
  // TODO: Add a table view property here to update?
  var adapter: Adapter?
  
  init(adapter: Adapter) {
    self.adapter = adapter
  }
  
  func onUpdateResources(resourceList: ResourceList) -> Bool {
    // TODO: Also update the Resources Table UI view when this function is called
    let addresses4 =
    self.adapter?.lastNetworkSettings?.ipv4Settings?.addresses ?? ["100.100.111.2"]
    let addresses6 =
    self.adapter?.lastNetworkSettings?.ipv6Settings?.addresses ?? [
      "fd00:0222:2021:1111::2"
    ]
    
    // TODO: Use actual passed in resources to achieve split tunnel
    let ipv4Routes = [NEIPv4Route(destinationAddress: "100.64.0.0", subnetMask: "255.192.0.0")]
    let ipv6Routes = [
      NEIPv6Route(destinationAddress: "fd00:0222:2021:1111::0", networkPrefixLength: 64)
    ]
    
    return setTunnelSettingsKeepingSomeExisting(
      addresses4: addresses4, addresses6: addresses6, ipv4Routes: ipv4Routes, ipv6Routes: ipv6Routes)
  }
  
  func onSetTunnelAddresses(tunnelAddresses: TunnelAddresses) -> Bool {
    let addresses4 = [tunnelAddresses.address4.toString()]
    let addresses6 = [tunnelAddresses.address6.toString()]
    let ipv4Routes =
    Adapter.currentAdapter?.lastNetworkSettings?.ipv4Settings?.includedRoutes ?? []
    let ipv6Routes =
    Adapter.currentAdapter?.lastNetworkSettings?.ipv6Settings?.includedRoutes ?? []
    
    return setTunnelSettingsKeepingSomeExisting(
      addresses4: addresses4, addresses6: addresses6, ipv4Routes: ipv4Routes, ipv6Routes: ipv6Routes)
  }
  
  private func setTunnelSettingsKeepingSomeExisting(
    addresses4: [String], addresses6: [String], ipv4Routes: [NEIPv4Route], ipv6Routes: [NEIPv6Route]
  ) -> Bool {
    let logger = Logger(subsystem: "dev.firezone.firezone", category: "packet-tunnel")
    
    if self.adapter != nil {
      do {
        /* If the tunnel interface addresses are being updated, it's impossible for the tunnel to
         stay up due to the way WireGuard works. Still, we try not to change the tunnel's routes
         here Just In Case™.
         */
        try self.adapter!.setNetworkSettings(
          self.adapter!.generateNetworkSettings(
            addresses4: addresses4,
            addresses6: addresses6,
            ipv4Routes: ipv4Routes,
            ipv6Routes: ipv6Routes
          )
        )
        
        return true
      } catch let error {
        logger.log(level: .debug, "Error setting adapter settings: \(String(describing: error))")
        
        return false
      }
    } else {
      logger.log(level: .debug, "Adapter not initialized!")
      
      return false
    }
  }
}

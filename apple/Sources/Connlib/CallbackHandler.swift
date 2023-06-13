//
//  Callbacks.swift
//  connlib
//
//  Created by Jamil Bou Kheir on 4/3/23.
//

import NetworkExtension
import os.log

public protocol CallbackHandlerDelegate: AnyObject {
    func didUpdateResources(_ resources: [Resource])
    func didUpdateInterfaceAddresses(_ interfaceAddresses: InterfaceAddresses)
}

public class CallbackHandler {
    var adapter: Adapter
    public weak var delegate: CallbackHandlerDelegate?

    init(adapter: Adapter) {
        self.adapter = adapter
    }

    func onUpdateResources(resourceList: ResourceList) -> Bool {

        let logger = Logger(subsystem: "dev.firezone.firezone", category: "packet-tunnel")

        do {
            let resources = try resourceList.toResources()
            logger.debug("Resources updated: \(resources)")
            delegate?.didUpdateResources(resources)
        } catch {
            logger.error("Error parsing resource list: \(String(describing: error)) (JSON: \(resourceList.resources.toString()))")
            return false
        }

        return true
    }

    func onSetTunnelAddresses(tunnelAddresses: TunnelAddresses) -> Bool {
        let logger = Logger(subsystem: "dev.firezone.firezone", category: "packet-tunnel")

        let interfaceAddresses = tunnelAddresses.toInterfaceAddresses()
        logger.debug("Interface addresses updated: (\(interfaceAddresses.ipv4), \(interfaceAddresses.ipv6))")
        delegate?.didUpdateInterfaceAddresses(interfaceAddresses)

        return true
    }
}

extension ResourceList {
    enum ParseError: Error {
        case notUTF8
    }

    func toResources() throws -> [Resource] {
        let jsonString = resources.toString()
        guard let jsonData = jsonString.data(using: .utf8) else {
            throw ParseError.notUTF8
        }
        return try JSONDecoder().decode([Resource].self, from: jsonData)
    }
}

extension TunnelAddresses {
    func toInterfaceAddresses() -> InterfaceAddresses {
        InterfaceAddresses(ipv4: address4.toString(), ipv6: address6.toString())
    }
}

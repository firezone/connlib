//
//  Resource.swift
//  (c) 2023 Firezone, Inc.
//  LICENSE: Apache-2.0

import Foundation

public struct Resource: Decodable {
    enum ResourceLocation {
        case dns(domain: String, ipv4: String, ipv6: String)
        case cidr(cidrAddress: String)
    }

    let name: String
    let resourceLocation: ResourceLocation
}

// A DNS resource example:
//  {
//    "type": "dns",
//    "address": "app.posthog.com",
//    "name": "PostHog",
//    "ipv4": "100.64.0.1",
//    "ipv6": "fd00:2021:11111::1"
//  }
//
// A CIDR resource example:
//   {
//     "type": "cidr",
//     "address": "10.0.0.0/24",
//     "name": "AWS SJC VPC1",
//   }

extension Resource {
    enum ResourceKeys: String, CodingKey {
        case type
        case address
        case name
        case ipv4
        case ipv6
    }

    enum DecodeError: Error {
        case invalidType(String)
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: ResourceKeys.self)
        let name = try container.decode(String.self, forKey: .name)
        let type = try container.decode(String.self, forKey: .type)
        let resourceLocation: ResourceLocation = try {
            switch type {
            case "dns":
                let domain = try container.decode(String.self, forKey: .address)
                let ipv4 = try container.decode(String.self, forKey: .ipv4)
                let ipv6 = try container.decode(String.self, forKey: .ipv6)
                return .dns(domain: domain, ipv4: ipv4, ipv6: ipv6)
            case "cidr":
                let address = try container.decode(String.self, forKey: .address)
                return .cidr(cidrAddress: address)
            default:
                throw DecodeError.invalidType(type)
            }
        }()
        self.init(name: name, resourceLocation: resourceLocation)
    }
}

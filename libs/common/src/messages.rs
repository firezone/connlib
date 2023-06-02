//! Message types that are used by both the gateway and client.
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;

mod key;

pub use key::Key;

/// General type for handling portal's id (UUID v4)
pub type Id = Uuid;

/// Represents a wireguard peer.
#[derive(Debug, PartialEq, Eq, Deserialize, Serialize, Clone)]
pub struct Peer {
    /// Keepalive: How often to send a keep alive message.
    pub persistent_keepalive: Option<u16>,
    /// Peer's public key.
    pub public_key: Key,
    /// Peer's Ipv4 (only 1 ipv4 per peer for now and mandatory).
    pub ipv4: Ipv4Addr,
    /// Peer's Ipv6 (only 1 ipv6 per peer for now and mandatory).
    pub ipv6: Ipv6Addr,
    /// Preshared key for the given peer.
    pub preshared_key: Key,
}

/// Represent a connection request from a client to a given resource.
///
/// While this is a client-only message it's hosted in common since the tunnel
/// make use of this message type.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RequestConnection {
    /// Resource id the request is for.
    pub resource_id: Id,
    /// The preshared key the client generated for the connection that it is trying to establish.
    pub client_preshared_key: Key,
    /// Client's local RTC Session Description that the client will use for this connection.
    pub client_rtc_sdp: RTCSessionDescription,
}

/// Description of a resource from a client's perspective.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct ResourceDescription {
    /// Resource's id.
    pub id: Id,
    /// Internal resource's domain name if any.
    // TODO: this is either a dns name or a cidr
    pub address: Option<String>,
    /// Resource's ipv4 mapping.
    ///
    /// Note that this is not the actual ipv4 for the resource not even wireguard's ipv4 for the resource.
    /// This is just the mapping we use internally between a resource and its ip for intercepting packets.
    pub ipv4: Ipv4Addr,
    /// Resource's ipv6 mapping.
    ///
    /// Note that this is not the actual ipv6 for the resource not even wireguard's ipv6 for the resource.
    /// This is just the mapping we use internally between a resource and its ip for intercepting packets.
    pub ipv6: Ipv6Addr,
}

/// Represents a wireguard interface configuration.
///
/// Note that the ips are /32 for ipv4 and /128 for ipv6.
/// This is done to minimize collisions and we update the routing table manually.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct Interface {
    /// Interface's Ipv4.
    pub ipv4: Ipv4Addr,
    /// Interface's Ipv6.
    pub ipv6: Ipv6Addr,
    /// DNS that will be used to query for DNS that aren't within our resource list.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub upstream_dns: Vec<IpAddr>,
}

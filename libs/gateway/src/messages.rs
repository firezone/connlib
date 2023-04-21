use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use firezone_tunnel::RTCSessionDescription;
use libs_common::messages::{Id, Interface, Peer};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize, Clone)]
pub struct InitGateway {
    pub interface: Interface,
    pub ipv4_masquerade: bool,
    pub ipv6_masquerade: bool,
    pub resources: Vec<Resource>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Client {
    pub id: Id,
    pub peer: Peer,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ConnectionRequest {
    pub user_id: Id,
    pub client: Client,
    pub rtc_sdp: RTCSessionDescription,
    pub relays: Vec<String>,
    pub resource: Resource,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub enum Destination {
    DnsName(String),
    Ip(Vec<IpAddr>),
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct Resource {
    pub id: Id,
    pub internal_ipv4: Option<Ipv4Addr>,
    pub internal_ipv6: Option<Ipv6Addr>,
    pub resource_address: Destination,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Metrics {
    peers_metrics: Vec<Metric>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Metric {
    pub client_id: Id,
    pub resource_id: Id,
    pub rx_bytes: u32,
    pub tx_bytes: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RemoveResource {
    pub id: Id,
}

// These messages are the messages that can be recieved
// either by a client or a gateway by the client.
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum IngressMessages {
    InitGateway(InitGateway),
    ConnectionRequest(ConnectionRequest),
    AddResource(Resource),
    RemoveResource(RemoveResource),
    UpdateResource(Resource),
}

// These messages can be sent from a gateway
// to a control pane.
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum EgressMessages {
    ConnectionReady(ConnectionReady),
    Metrics(Metrics),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ConnectionReady {
    pub client_id: Id,
    pub gateway_rtc_sdp: RTCSessionDescription,
}

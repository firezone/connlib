use firezone_tunnel::RTCSessionDescription;
use serde::{Deserialize, Serialize};

use libs_common::messages::{Id, Interface, Key, RequestConnection, ResourceDescription};

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize, Clone)]
pub struct InitClient {
    pub interface: Interface,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub resources: Vec<ResourceDescription>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct RemoveResource {
    pub id: Id,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Connect {
    pub rtc_sdp: RTCSessionDescription,
    pub resource_id: Id,
    pub gateway_public_key: Key,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Relays {
    pub resource_id: Id,
    pub relays: Vec<String>,
}

// These messages are the messages that can be recieved
// by a client.
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum IngressMessages {
    InitClient(InitClient),
    Relays(Relays),
    Connect(Connect),

    // Resources: arrive in an orderly fashion
    AddResource(ResourceDescription),
    RemoveResource(RemoveResource),
    UpdateResource(ResourceDescription),
}

// These messages can be sent from a client to a control pane
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum EgressMessages {
    GetConnectionDetails(Id),
    RequestConnection(RequestConnection),
}

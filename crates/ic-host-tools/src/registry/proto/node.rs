///
/// `NodeRecord`
///
/// Protobuf registry record describing one replica node.
///

#[derive(Clone, Eq, prost::Message, PartialEq)]
pub struct NodeRecord {
    /// Raw node operator principal bytes.
    #[prost(bytes = "vec", tag = "15")]
    pub node_operator_id: Vec<u8>,
}

///
/// `NodeOperatorRecord`
///
/// Protobuf registry record describing one node operator.
///

#[derive(Clone, Eq, prost::Message, PartialEq)]
pub struct NodeOperatorRecord {
    /// Raw node operator principal bytes.
    #[prost(bytes = "vec", tag = "1")]
    pub node_operator_principal_id: Vec<u8>,
    /// Registry node allowance for the operator.
    #[prost(uint64, tag = "2")]
    pub node_allowance: u64,
    /// Raw node provider principal bytes.
    #[prost(bytes = "vec", tag = "3")]
    pub node_provider_principal_id: Vec<u8>,
    /// Registered data center identifier.
    #[prost(string, tag = "4")]
    pub dc_id: String,
}

///
/// `DataCenterRecord`
///
/// Protobuf registry record describing one data center.
///

#[derive(Clone, prost::Message, PartialEq)]
pub struct DataCenterRecord {
    /// Registered data center identifier.
    #[prost(string, tag = "1")]
    pub id: String,
    /// Registry-supplied region text.
    #[prost(string, tag = "2")]
    pub region: String,
    /// Registry-supplied owner text.
    #[prost(string, tag = "3")]
    pub owner: String,
    /// Optional data center coordinates.
    #[prost(message, optional, tag = "4")]
    pub gps: Option<Gps>,
}

///
/// `Gps`
///
/// Protobuf geographic coordinates attached to a data center record.
///

#[derive(Clone, prost::Message, PartialEq)]
pub struct Gps {
    /// Latitude in degrees.
    #[prost(float, tag = "1")]
    pub latitude: f32,
    /// Longitude in degrees.
    #[prost(float, tag = "2")]
    pub longitude: f32,
}

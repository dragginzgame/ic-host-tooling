use prost::Oneof;

///
/// `RegistryError`
///
/// Protobuf error returned by a registry canister lookup.
///

#[derive(Clone, Eq, prost::Message, PartialEq)]
pub struct RegistryError {
    /// Raw Registry error discriminant.
    #[prost(enumeration = "RegistryErrorCode", tag = "1")]
    pub code: i32,
    /// Registry-supplied failure explanation.
    #[prost(string, tag = "2")]
    pub reason: String,
    /// Raw Registry key bytes.
    #[prost(bytes = "vec", tag = "3")]
    pub key: Vec<u8>,
}

///
/// `RegistryErrorCode`
///
/// Protobuf error category returned by the registry canister.
///

#[derive(Clone, Copy, Debug, prost::Enumeration, Eq, PartialEq)]
#[repr(i32)]
pub enum RegistryErrorCode {
    /// Malformed request encoding.
    MalformedMessage = 0,
    /// The requested key is absent.
    KeyNotPresent = 1,
    /// The mutation key already exists.
    KeyAlreadyPresent = 2,
    /// The requested version is not latest.
    VersionNotLatest = 3,
    /// The requested version exceeds latest.
    VersionBeyondLatest = 4,
    /// The caller is not authorized.
    Authorization = 5,
    /// Registry internal failure.
    InternalError = 999,
}

///
/// `LargeValueChunkKeys`
///
/// Protobuf chunk keys used to retrieve a large registry value.
///

#[derive(Clone, Eq, prost::Message, PartialEq)]
pub struct LargeValueChunkKeys {
    /// `Content`-addressed SHA-256 chunk digests in retrieval order.
    #[prost(bytes = "vec", repeated, tag = "1")]
    pub chunk_content_sha256s: Vec<Vec<u8>>,
}

///
/// `RegistryGetValueRequest`
///
/// Protobuf request for a registry key at an optional version.
///

#[derive(Clone, Eq, prost::Message, PartialEq)]
pub struct RegistryGetValueRequest {
    /// Registry version carried by this message.
    #[prost(message, optional, tag = "1")]
    pub version: Option<UInt64Value>,
    /// Raw Registry key bytes.
    #[prost(bytes = "vec", tag = "2")]
    pub key: Vec<u8>,
}

///
/// `UInt64Value`
///
/// Protobuf wrapper for an unsigned 64-bit registry value.
///

#[derive(Clone, Eq, prost::Message, PartialEq)]
pub struct UInt64Value {
    /// Wrapped unsigned integer value.
    #[prost(uint64, tag = "1")]
    pub value: u64,
}

///
/// `RegistryGetValueResponse`
///
/// This is the high-capacity `get_value` response wire shape. The upstream
/// proto calls it `HighCapacityRegistryGetValueResponse`; this crate keeps the
/// focused projection name retained from Query; this is not a full upstream schema.
///

#[derive(Clone, Eq, prost::Message, PartialEq)]
pub struct RegistryGetValueResponse {
    /// Optional Registry-side failure.
    #[prost(message, optional, tag = "1")]
    pub error: Option<RegistryError>,
    /// Registry version carried by this message.
    #[prost(uint64, tag = "2")]
    pub version: u64,
    /// Inline or chunk-referenced value representation.
    #[prost(oneof = "registry_get_value_response::Content", tags = "3, 4")]
    pub content: Option<registry_get_value_response::Content>,
    /// Registry-assigned timestamp in nanoseconds.
    #[prost(uint64, tag = "5")]
    pub timestamp_nanoseconds: u64,
}

/// Wire alternatives for registry get value response.
pub mod registry_get_value_response {
    use super::{LargeValueChunkKeys, Oneof};

    ///
    /// `Content`
    ///
    /// Inline value bytes or explicit chunk references supplied by the Registry.
    ///

    #[derive(Clone, Eq, Oneof, PartialEq)]
    pub enum Content {
        /// Complete inline value bytes.
        #[prost(bytes = "vec", tag = "3")]
        Value(Vec<u8>),
        /// `Content`-addressed large-value chunk references.
        #[prost(message, tag = "4")]
        LargeValueChunkKeys(LargeValueChunkKeys),
    }
}

///
/// `RegistryGetLatestVersionResponse`
///
/// Protobuf response containing the latest registry version.
///

#[derive(Clone, Eq, prost::Message, PartialEq)]
pub struct RegistryGetLatestVersionResponse {
    /// Registry version carried by this message.
    #[prost(uint64, tag = "1")]
    pub version: u64,
}

///
/// `RegistryGetChangesSinceRequest`
///
/// Protobuf request for one certified Registry delta batch after a version.
///

#[derive(Clone, Eq, prost::Message, PartialEq)]
pub struct RegistryGetChangesSinceRequest {
    /// Last Registry version already held by the caller.
    #[prost(uint64, tag = "1")]
    pub version: u64,
}

///
/// `HighCapacityRegistryValue`
///
/// One versioned value returned in an ordinary Registry delta.
///

#[derive(Clone, Eq, prost::Message, PartialEq)]
pub struct HighCapacityRegistryValue {
    /// Registry version at which this value was mutated.
    #[prost(uint64, tag = "2")]
    pub version: u64,
    /// Inline value, deletion marker, or large-value chunk references.
    #[prost(oneof = "high_capacity_registry_value::Content", tags = "1, 3, 4")]
    pub content: Option<high_capacity_registry_value::Content>,
    /// Registry-assigned mutation timestamp in nanoseconds.
    #[prost(uint64, tag = "5")]
    pub timestamp_nanoseconds: u64,
}

/// Wire alternatives for high capacity registry value.
pub mod high_capacity_registry_value {
    use super::LargeValueChunkKeys;
    use prost::Oneof;

    ///
    /// `Content`
    ///
    /// Value representation in one high-capacity Registry delta.
    ///

    #[derive(Clone, Eq, Oneof, PartialEq)]
    pub enum Content {
        /// Complete inline value bytes.
        #[prost(bytes = "vec", tag = "1")]
        Value(Vec<u8>),
        /// True when this mutation deleted the key.
        #[prost(bool, tag = "3")]
        DeletionMarker(bool),
        /// `Content`-addressed chunks for one large value.
        #[prost(message, tag = "4")]
        LargeValueChunkKeys(LargeValueChunkKeys),
    }
}

///
/// `HighCapacityRegistryDelta`
///
/// All returned mutations for one Registry key.
///

#[derive(Clone, Eq, prost::Message, PartialEq)]
pub struct HighCapacityRegistryDelta {
    /// Raw Registry key bytes.
    #[prost(bytes = "vec", tag = "1")]
    pub key: Vec<u8>,
    /// Mutations for the key in Registry-version order.
    #[prost(message, repeated, tag = "2")]
    pub values: Vec<HighCapacityRegistryValue>,
}

///
/// `HighCapacityRegistryGetChangesSinceResponse`
///
/// Ordinary-query Registry deltas and the latest version observed by the query.
///

#[derive(Clone, Eq, prost::Message, PartialEq)]
pub struct HighCapacityRegistryGetChangesSinceResponse {
    /// Registry-side query error.
    #[prost(message, optional, tag = "1")]
    pub error: Option<RegistryError>,
    /// Latest Registry version observed by the query.
    #[prost(uint64, tag = "2")]
    pub version: u64,
    /// All key deltas since the requested version.
    #[prost(message, repeated, tag = "3")]
    pub deltas: Vec<HighCapacityRegistryDelta>,
}

///
/// `HighCapacityRegistryAtomicMutateRequest`
///
/// One atomic Registry delta committed under a certified version label.
///

#[derive(Clone, Eq, prost::Message, PartialEq)]
pub struct HighCapacityRegistryAtomicMutateRequest {
    /// Ordered mutations applied by the Registry version.
    #[prost(message, repeated, tag = "1")]
    pub mutations: Vec<HighCapacityRegistryMutation>,
    /// Preconditions checked before applying the atomic mutation.
    #[prost(message, repeated, tag = "5")]
    pub preconditions: Vec<RegistryPrecondition>,
    /// Registry-assigned mutation timestamp in nanoseconds since the Unix epoch.
    #[prost(uint64, tag = "6")]
    pub timestamp_nanoseconds: u64,
}

///
/// `HighCapacityRegistryMutation`
///
/// One mutation in a certified Registry atomic mutation request.
///

#[derive(Clone, Eq, prost::Message, PartialEq)]
pub struct HighCapacityRegistryMutation {
    /// Raw `RegistryMutation.Type` numeric discriminant.
    #[prost(int32, tag = "1")]
    pub mutation_type: i32,
    /// Raw Registry key bytes.
    #[prost(bytes = "vec", tag = "2")]
    pub key: Vec<u8>,
    /// Inline value bytes or large-value chunk references.
    ///
    /// Historical non-delete mutations with an empty legacy `bytes` value
    /// decode as `None`; the transport adapter interprets those as an empty
    /// inline value, matching the official Registry conversion.
    #[prost(oneof = "high_capacity_registry_mutation::Content", tags = "3, 4")]
    pub content: Option<high_capacity_registry_mutation::Content>,
}

/// Wire alternatives for high capacity registry mutation.
pub mod high_capacity_registry_mutation {
    use super::LargeValueChunkKeys;
    use prost::Oneof;

    ///
    /// `Content`
    ///
    /// Value representation in one high-capacity Registry mutation.
    ///

    #[derive(Clone, Eq, Oneof, PartialEq)]
    pub enum Content {
        /// Complete inline value bytes.
        #[prost(bytes = "vec", tag = "3")]
        Value(Vec<u8>),
        /// `Content`-addressed chunks that require bounded follow-up retrieval.
        #[prost(message, tag = "4")]
        LargeValueChunkKeys(LargeValueChunkKeys),
    }
}

///
/// `RegistryMutationType`
///
/// Official Registry mutation discriminants used by certified deltas.
///

#[derive(Clone, Copy, Debug, prost::Enumeration, Eq, PartialEq)]
#[repr(i32)]
pub enum RegistryMutationType {
    /// Insert a previously absent key.
    Insert = 0,
    /// Update an existing key.
    Update = 1,
    /// Delete an existing key.
    Delete = 2,
    /// Insert or update a key.
    Upsert = 4,
}

///
/// `RegistryPrecondition`
///
/// Expected key version attached to one atomic Registry mutation.
///

#[derive(Clone, Eq, prost::Message, PartialEq)]
pub struct RegistryPrecondition {
    /// Registry key whose version is constrained.
    #[prost(bytes = "vec", tag = "1")]
    pub key: Vec<u8>,
    /// Required Registry version for the key.
    #[prost(uint64, tag = "2")]
    pub expected_version: u64,
}

///
/// `RegistryCertifiedResponse`
///
/// Protobuf response returned by the Registry's certified query methods.
///

#[derive(Clone, Eq, prost::Message, PartialEq)]
pub struct RegistryCertifiedResponse {
    /// Certified mixed hash-tree witness.
    #[prost(message, optional, tag = "1")]
    pub hash_tree: Option<RegistryMixedHashTree>,
    /// CBOR system certificate authenticating the tree root.
    #[prost(bytes = "vec", tag = "2")]
    pub certificate: Vec<u8>,
}

///
/// `RegistryMixedHashTree`
///
/// Protobuf representation of a certified Registry mixed hash tree.
///

#[derive(Clone, Eq, prost::Message, PartialEq)]
pub struct RegistryMixedHashTree {
    /// Encoded tree node.
    #[prost(oneof = "registry_mixed_hash_tree::Tree", tags = "1, 2, 3, 4, 5")]
    pub tree: Option<registry_mixed_hash_tree::Tree>,
}

/// Wire alternatives for registry mixed hash tree.
pub mod registry_mixed_hash_tree {
    use super::RegistryMixedHashTree;
    use prost::Oneof;

    ///
    /// `Fork`
    ///
    /// Two child branches in a Registry mixed hash tree.
    ///

    #[derive(Clone, Eq, prost::Message, PartialEq)]
    pub struct Fork {
        /// Left child branch.
        #[prost(message, optional, boxed, tag = "1")]
        pub left_tree: Option<Box<RegistryMixedHashTree>>,
        /// Right child branch.
        #[prost(message, optional, boxed, tag = "2")]
        pub right_tree: Option<Box<RegistryMixedHashTree>>,
    }

    ///
    /// `Labeled`
    ///
    /// Label and child branch in a Registry mixed hash tree.
    ///

    #[derive(Clone, Eq, prost::Message, PartialEq)]
    pub struct Labeled {
        /// Raw branch label.
        #[prost(bytes = "vec", tag = "1")]
        pub label: Vec<u8>,
        /// `Labeled` child branch.
        #[prost(message, optional, boxed, tag = "2")]
        pub subtree: Option<Box<RegistryMixedHashTree>>,
    }

    ///
    /// `Tree`
    ///
    /// One node in a Registry mixed hash tree.
    ///

    #[derive(Clone, Eq, Oneof, PartialEq)]
    pub enum Tree {
        /// Empty tree node.
        #[prost(message, tag = "1")]
        Empty(()),
        /// `Fork` containing two child branches.
        #[prost(message, tag = "2")]
        Fork(Box<Fork>),
        /// `Labeled` child branch.
        #[prost(message, tag = "3")]
        Labeled(Box<Labeled>),
        /// Visible leaf bytes.
        #[prost(bytes, tag = "4")]
        LeafData(Vec<u8>),
        /// Pruned SHA-256 subtree digest.
        #[prost(bytes, tag = "5")]
        PrunedDigest(Vec<u8>),
    }
}

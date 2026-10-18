///
/// `SubnetListRecord`
///
/// Protobuf registry record containing the current subnet identifiers.
///

#[derive(Clone, Eq, prost::Message, PartialEq)]
pub struct SubnetListRecord {
    /// Raw registered subnet principal bytes.
    #[prost(bytes = "vec", repeated, tag = "2")]
    pub subnets: Vec<Vec<u8>>,
}

///
/// `SubnetRecord`
///
/// Protobuf registry record describing one subnet.
///

#[derive(Clone, Eq, prost::Message, PartialEq)]
pub struct SubnetRecord {
    /// Raw member node principal bytes.
    #[prost(bytes = "vec", repeated, tag = "3")]
    pub membership: Vec<Vec<u8>>,
    /// Raw subnet classification discriminant.
    #[prost(enumeration = "SubnetType", tag = "15")]
    pub subnet_type: i32,
    /// Raw cycles cost schedule discriminant.
    #[prost(enumeration = "CanisterCyclesCostSchedule", tag = "30")]
    pub canister_cycles_cost_schedule: i32,
}

///
/// `SubnetType`
///
/// Protobuf classification assigned to a subnet record.
///

#[derive(Clone, Copy, Debug, prost::Enumeration, Eq, PartialEq)]
#[repr(i32)]
pub enum SubnetType {
    /// No classification was specified.
    Unspecified = 0,
    /// Application subnet.
    Application = 1,
    /// System subnet.
    System = 2,
    /// Verified application subnet.
    VerifiedApplication = 4,
    /// `CloudEngine` subnet.
    CloudEngine = 5,
}

///
/// `CanisterCyclesCostSchedule`
///
/// Protobuf cycles cost schedule assigned to a subnet record.
///

#[derive(Clone, Copy, Debug, prost::Enumeration, Eq, PartialEq)]
#[repr(i32)]
pub enum CanisterCyclesCostSchedule {
    /// No classification was specified.
    Unspecified = 0,
    /// Normal cycles cost schedule.
    Normal = 1,
    /// Free cycles cost schedule.
    Free = 2,
}

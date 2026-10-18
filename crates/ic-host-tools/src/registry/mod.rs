//! Pure focused Registry wire projections and shard interval admission.
//!
//! Extracted from Query 0.57.0 `3c13426d31321b7f9c78828b676fe0473ab51b86`.
//! These partial Protobuf projections preserve its wire tags/discriminants;
//! they are not a complete Registry schema. Callers own byte/work budgets,
//! trusted versions, key discovery, transport, certification and replay policy.

pub mod proto;
pub mod routing_shards;

#[cfg(test)]
mod tests;

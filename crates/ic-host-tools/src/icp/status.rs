//! Installation projection over successful ICP CLI status JSON.

use std::fmt;

/// Status JSON does not establish an installed or uninstalled canister.
#[derive(Debug)]
pub enum InstallationStatusError {
    /// The complete input is not valid JSON.
    Json(serde_json::Error),
    /// Successful status output did not include the installation fact.
    MissingModuleHash,
    /// The installation fact is neither null nor a string.
    InvalidModuleHash,
}

impl fmt::Display for InstallationStatusError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(source) => write!(formatter, "parse icp canister status JSON: {source}"),
            Self::MissingModuleHash => {
                formatter.write_str("icp canister status JSON omitted module_hash")
            }
            Self::InvalidModuleHash => {
                formatter.write_str("icp canister status JSON has an invalid module_hash")
            }
        }
    }
}

impl std::error::Error for InstallationStatusError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Json(source) => Some(source),
            Self::MissingModuleHash | Self::InvalidModuleHash => None,
        }
    }
}

/// Project the installation fact from one successful, caller-captured status.
///
/// A null `module_hash` means created but uninstalled; any string means installed.
/// Missing, malformed or failed probes never establish absence. The caller must
/// check command success first and owns input capture/byte limits and JSON schema
/// requirements beyond this single fact. Unknown fields are ignored.
///
/// # Errors
/// Returns typed JSON, missing-field or invalid-field failure without guessing.
pub fn decode_installation_status(input: &[u8]) -> Result<bool, InstallationStatusError> {
    let status: serde_json::Value =
        serde_json::from_slice(input).map_err(InstallationStatusError::Json)?;
    match status.get("module_hash") {
        Some(serde_json::Value::Null) => Ok(false),
        Some(serde_json::Value::String(_)) => Ok(true),
        Some(_) => Err(InstallationStatusError::InvalidModuleHash),
        None => Err(InstallationStatusError::MissingModuleHash),
    }
}

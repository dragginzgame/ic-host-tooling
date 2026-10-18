//! Native response-bounded IC agent construction and exact endpoint binding.
//!
//! Callers admit endpoints and select response ceilings. Trust roots, identity,
//! historical verification, deadlines, retries and network calls remain theirs.

use ic_platform::agent::{Agent, AgentBuilder, AgentError};
use std::sync::OnceLock;
use url::Url;

#[cfg(test)]
mod tests;

/// Construct the SDK's builder with a parsed URL and explicit response ceiling.
///
/// Parsing before SDK construction avoids its infallible URL parser. This does
/// not impose a product endpoint policy: callers must admit schemes, hosts,
/// credentials, query parameters and fragments before use. They can customize
/// the returned native builder, including identity and verification settings.
/// No root key is fetched and no request is made by this function.
#[must_use]
pub fn agent_builder(endpoint: &Url, maximum_response_bytes: usize) -> AgentBuilder {
    Agent::builder()
        .with_url(endpoint.as_str())
        .with_max_response_body_size(maximum_response_bytes)
}

/// Construction or reuse of an endpoint-bound agent was refused.
#[derive(Debug, thiserror::Error)]
pub enum EndpointAgentError {
    /// Endpoint text could not be parsed; no binding was selected.
    #[error("invalid IC agent endpoint")]
    Endpoint(#[source] url::ParseError),
    /// Native SDK construction failed; no binding was selected.
    #[error("IC agent construction failed")]
    Build(#[source] AgentError),
    /// The exact endpoint text or response ceiling differs from the binding.
    /// The selected agent remains available for its original pair.
    #[error("IC agent endpoint or response ceiling differs from its binding")]
    BindingMismatch,
}

/// A lazily constructed native agent bound to exact endpoint text and a ceiling.
///
/// Successful first initialization fixes both values. Concurrent callers may
/// construct competing agents, but only the winning binding is retained. A
/// different pair is refused rather than silently reusing another transport.
/// Failed construction leaves the instance unbound. Cloning an initialized
/// instance retains its binding and the SDK agent's shared transport.
#[derive(Clone, Default)]
pub struct EndpointAgent {
    transport: OnceLock<(String, usize, Agent)>,
}

impl EndpointAgent {
    /// Return the agent for this exact endpoint and response ceiling.
    ///
    /// Uses SDK defaults apart from the endpoint and ceiling. Caller-owned
    /// endpoint admission must run before this method. URL parsing alone is not
    /// a trust, credential or HTTP(S) admission policy. No network call is made.
    /// Use [`agent_builder`] directly when custom identity or settings are needed.
    ///
    /// # Errors
    /// Returns malformed URL, SDK construction or conflicting binding errors.
    pub fn get(
        &self,
        endpoint: &str,
        maximum_response_bytes: usize,
    ) -> Result<&Agent, EndpointAgentError> {
        let transport = if let Some(transport) = self.transport.get() {
            transport
        } else {
            let parsed = Url::parse(endpoint).map_err(EndpointAgentError::Endpoint)?;
            let agent = agent_builder(&parsed, maximum_response_bytes)
                .build()
                .map_err(EndpointAgentError::Build)?;
            self.transport
                .get_or_init(|| (endpoint.to_owned(), maximum_response_bytes, agent))
        };
        let (selected_endpoint, selected_limit, agent) = transport;
        if selected_endpoint != endpoint || *selected_limit != maximum_response_bytes {
            return Err(EndpointAgentError::BindingMismatch);
        }
        Ok(agent)
    }
}

impl std::fmt::Debug for EndpointAgent {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Endpoints may contain credentials until the consumer admits them.
        formatter
            .debug_struct("EndpointAgent")
            .field("initialized", &self.transport.get().is_some())
            .finish_non_exhaustive()
    }
}

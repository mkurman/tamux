//! Predefined LLM provider definitions.
//!
//! This keeps the TUI's built-in provider defaults aligned with the app-wide
//! provider registry while remaining a TUI-local source for picker/config UX.

#[path = "providers/model_catalog.rs"]
mod model_catalog;

mod context;

#[path = "providers_parts/known_models_for_provider_auth.rs"]
mod known_models_for_provider_auth;
#[path = "providers_parts/normalize_model_lookup_value_to_default_model_for_provider_auth.rs"]
mod normalize_model_lookup_value_to_default_model_for_provider_auth;
#[path = "providers_parts/providers.rs"]
mod providers;

pub use known_models_for_provider_auth::*;
pub use normalize_model_lookup_value_to_default_model_for_provider_auth::*;

#[cfg(test)]
#[path = "providers/tests.rs"]
mod tests;

use super::model_catalog;
use crate::state::config::FetchedModel;

#[cfg(test)]
pub fn known_models_for_provider(provider: &str) -> Vec<FetchedModel> {
    known_models_for_provider_auth(provider, "api_key")
}

pub fn known_models_for_provider_auth(provider: &str, auth_source: &str) -> Vec<FetchedModel> {
    model_catalog::known_models_for_provider_auth(provider, auth_source)
}

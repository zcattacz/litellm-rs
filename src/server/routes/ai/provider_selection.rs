//! Provider selection helpers for AI routes

use crate::core::providers::Provider;
use crate::core::router::UnifiedRouter;
use crate::core::types::model::ProviderCapability;
use crate::utils::error::gateway_error::GatewayError;
use std::borrow::Cow;

pub struct ProviderSelection<'a> {
    pub provider: Cow<'a, Provider>,
    pub model: String,
}

pub fn select_provider_for_model<'a>(
    router: &UnifiedRouter,
    model: &str,
    capability: ProviderCapability,
) -> Result<ProviderSelection<'a>, GatewayError> {
    if model.trim().is_empty() {
        return Err(GatewayError::validation("Model is required"));
    }

    select_provider_from_unified_router(router, model, capability)
}

pub fn select_provider_for_optional_model<'a>(
    router: &UnifiedRouter,
    model: Option<&str>,
    capability: ProviderCapability,
) -> Result<(Cow<'a, Provider>, String), GatewayError> {
    let model = model.ok_or_else(|| GatewayError::validation("Model is required"))?;
    let selection = select_provider_for_model(router, model, capability)?;
    Ok((selection.provider, selection.model))
}

fn provider_supports_capability(provider: &Provider, capability: &ProviderCapability) -> bool {
    provider.capabilities().iter().any(|cap| cap == capability)
}

fn select_provider_from_unified_router<'a>(
    router: &UnifiedRouter,
    model: &str,
    capability: ProviderCapability,
) -> Result<ProviderSelection<'a>, GatewayError> {
    let deployment_id = router
        .get_deployments_for_model(model)
        .into_iter()
        .find(|id| {
            router
                .get_deployment(id)
                .map(|deployment| provider_supports_capability(&deployment.provider, &capability))
                .unwrap_or(false)
        })
        .ok_or_else(|| {
            GatewayError::validation(format!(
                "Model '{}' does not support {:?}",
                model, capability
            ))
        })?;

    let deployment = router
        .get_deployment(&deployment_id)
        .ok_or_else(|| GatewayError::internal("Selected deployment not found"))?;

    Ok(ProviderSelection {
        provider: Cow::Owned(deployment.provider.clone()),
        model: deployment.model.clone(),
    })
}

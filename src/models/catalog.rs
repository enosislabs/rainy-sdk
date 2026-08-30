//! Model-catalog domain facade.

/// Model-catalog types remain re-exported from the compatibility root while
/// this module provides a stable domain import path.
pub use super::{
    CapabilityFlag, ModelArchitecture, ModelBillingClass, ModelCatalogItem, ModelDataPolicy,
    ModelDataPolicyStatus, ModelDynamicPricing, ModelDynamicPricingTier, ModelLaunch,
    ModelLaunchAction, ModelLaunchAnimation, ModelLaunchControl, ModelLaunchGradient,
    ModelLaunchPresentation, ModelLaunchPricing, ModelLaunchSelection, ModelLaunchUi,
    ModelLaunchUiVariant, ModelLaunchVariant, ModelLaunchVariantPresentation, ModelList,
    ModelListItem, ModelPricing, ModelPricingPromo, ModelSelectionCriteria,
    ModelServiceTierPricing, ModelTier, RainyCapabilities, RainyCapabilitiesV2,
    RainyMultimodalCapabilitiesV2, RainyParametersCapabilitiesV2, RainyReasoningCapabilitiesV2,
    ReasoningControls, ReasoningMode, ReasoningPreference, ReasoningProfile, ReasoningProvider,
    ReasoningToggle, ThinkingBudget,
};

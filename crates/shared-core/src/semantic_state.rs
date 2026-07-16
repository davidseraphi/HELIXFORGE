//! Canonical semantic state vocabulary shared by Rust backends and TS frontends.

use serde::{Deserialize, Serialize};

/// Cross-cutting semantic state. Used for jobs, health checks, products, and
/// any surface that needs a single, honest state label.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticState {
    /// Blue — work is actively in progress.
    Active,
    /// Amber — paused, waiting for a person to decide or act.
    WaitingHuman,
    /// Violet — waiting for an external signal, dependency, or event.
    WaitingExternal,
    /// Green — completed and checked.
    Completed,
    /// Red — failed, unsafe, or unhealthy.
    Failed,
    /// Grey — not yet checked or explicitly unknown.
    Unknown,
}

impl SemanticState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::WaitingHuman => "waiting_human",
            Self::WaitingExternal => "waiting_external",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Unknown => "unknown",
        }
    }

    /// CSS color token name (matches `@helixforge/ui` design tokens).
    pub fn color_token(&self) -> &'static str {
        match self {
            Self::Active => "--state-active",
            Self::WaitingHuman => "--state-waiting-human",
            Self::WaitingExternal => "--state-waiting-external",
            Self::Completed => "--state-completed",
            Self::Failed => "--state-failed",
            Self::Unknown => "--state-unknown",
        }
    }

    /// Human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Active => "Active",
            Self::WaitingHuman => "Waiting for you",
            Self::WaitingExternal => "Waiting",
            Self::Completed => "Completed",
            Self::Failed => "Failed",
            Self::Unknown => "Unknown",
        }
    }
}

/// Product maturity lifecycle. Static per product in the catalog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductMaturity {
    /// Placeholder / template only.
    Scaffold,
    /// Early implementation, not yet integrated.
    Prototype,
    /// Integrated but not stable.
    Alpha,
    /// Feature-complete, stabilizing.
    Beta,
    /// Production-ready.
    Production,
}

impl ProductMaturity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Scaffold => "scaffold",
            Self::Prototype => "prototype",
            Self::Alpha => "alpha",
            Self::Beta => "beta",
            Self::Production => "production",
        }
    }

    /// Default catalog semantic state for a product at this maturity level.
    /// Scaffold products have no runtime yet; everything else is considered
    /// actively available in the catalog.
    pub fn default_semantic_state(&self) -> SemanticState {
        match self {
            Self::Scaffold => SemanticState::Unknown,
            _ => SemanticState::Active,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semantic_state_serializes_to_snake_case() {
        assert_eq!(
            serde_json::to_value(SemanticState::WaitingHuman).unwrap(),
            serde_json::json!("waiting_human")
        );
        assert_eq!(
            serde_json::to_value(SemanticState::WaitingExternal).unwrap(),
            serde_json::json!("waiting_external")
        );
    }

    #[test]
    fn product_maturity_round_trips() {
        for m in [
            ProductMaturity::Scaffold,
            ProductMaturity::Prototype,
            ProductMaturity::Alpha,
            ProductMaturity::Beta,
            ProductMaturity::Production,
        ] {
            let s = serde_json::to_string(&m).unwrap();
            let back: ProductMaturity = serde_json::from_str(&s).unwrap();
            assert_eq!(m, back);
        }
    }

    #[test]
    fn product_maturity_default_state() {
        assert_eq!(
            ProductMaturity::Scaffold.default_semantic_state(),
            SemanticState::Unknown
        );
        for m in [
            ProductMaturity::Prototype,
            ProductMaturity::Alpha,
            ProductMaturity::Beta,
            ProductMaturity::Production,
        ] {
            assert_eq!(m.default_semantic_state(), SemanticState::Active);
        }
    }

    #[test]
    fn semantic_state_as_str_and_label() {
        assert_eq!(SemanticState::Active.as_str(), "active");
        assert_eq!(SemanticState::Failed.label(), "Failed");
        assert_eq!(
            SemanticState::WaitingHuman.color_token(),
            "--state-waiting-human"
        );
    }
}

//! Product catalog — HelixForge product forges that reuse HelixCore.

use crate::semantic_state::ProductMaturity;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductTier {
    Standard,
    Frontier,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductMeta {
    pub order: u8,
    pub slug: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub tier: ProductTier,
    pub maturity: ProductMaturity,
    pub default_port: u16,
    pub nats_prefix: &'static str,
}

/// Canonical product registry (order 1..=N). HelixCore is order 0.
/// Order 21 (HelixPulse) is intentionally **last** — full cluster after 1–20.
pub const PRODUCT_CATALOG: &[ProductMeta] = &[
    ProductMeta {
        order: 1,
        slug: "helix-collab",
        title: "HelixCollab",
        description: "Real-time collaborative workspace",
        tier: ProductTier::Standard,
        maturity: ProductMaturity::Beta,
        default_port: 8101,
        nats_prefix: "helix.collab",
    },
    ProductMeta {
        order: 2,
        slug: "helix-code",
        title: "HelixCode",
        description: "AI-native collaborative IDE",
        tier: ProductTier::Standard,
        maturity: ProductMaturity::Beta,
        default_port: 8102,
        nats_prefix: "helix.code",
    },
    ProductMeta {
        order: 3,
        slug: "helix-flow",
        title: "HelixFlow",
        description: "Agentic automation & workflow engine",
        tier: ProductTier::Standard,
        maturity: ProductMaturity::Alpha,
        default_port: 8103,
        nats_prefix: "helix.flow",
    },
    ProductMeta {
        order: 4,
        slug: "helix-insights",
        title: "HelixInsights",
        description: "Predictive analytics & decision OS",
        tier: ProductTier::Standard,
        maturity: ProductMaturity::Scaffold,
        default_port: 8104,
        nats_prefix: "helix.insights",
    },
    ProductMeta {
        order: 5,
        slug: "helix-commerce",
        title: "HelixCommerce",
        description: "AI e-commerce & digital marketplace builder",
        tier: ProductTier::Standard,
        maturity: ProductMaturity::Scaffold,
        default_port: 8105,
        nats_prefix: "helix.commerce",
    },
    ProductMeta {
        order: 6,
        slug: "helix-edu",
        title: "HelixEdu",
        description: "Adaptive AI learning & certification platform",
        tier: ProductTier::Standard,
        maturity: ProductMaturity::Scaffold,
        default_port: 8106,
        nats_prefix: "helix.edu",
    },
    ProductMeta {
        order: 7,
        slug: "helix-capital",
        title: "HelixCapital",
        description: "AI financial operating system",
        tier: ProductTier::Standard,
        maturity: ProductMaturity::Scaffold,
        default_port: 8107,
        nats_prefix: "helix.capital",
    },
    ProductMeta {
        order: 8,
        slug: "helix-well",
        title: "HelixWell",
        description: "AI personal & team wellness platform",
        tier: ProductTier::Standard,
        maturity: ProductMaturity::Scaffold,
        default_port: 8108,
        nats_prefix: "helix.well",
    },
    ProductMeta {
        order: 9,
        slug: "helix-network",
        title: "HelixNetwork",
        description: "AI professional networking & opportunity engine",
        tier: ProductTier::Standard,
        maturity: ProductMaturity::Scaffold,
        default_port: 8109,
        nats_prefix: "helix.network",
    },
    ProductMeta {
        order: 10,
        slug: "helix-forge-studio",
        title: "HelixForge Studio",
        description: "No-code/low-code AI app & internal tool builder",
        tier: ProductTier::Standard,
        maturity: ProductMaturity::Scaffold,
        default_port: 8110,
        nats_prefix: "helix.forge",
    },
    ProductMeta {
        order: 11,
        slug: "helix-synthbio",
        title: "HelixSynthBio",
        description: "Synthetic biology design & virtual wet-lab",
        tier: ProductTier::Frontier,
        maturity: ProductMaturity::Scaffold,
        default_port: 8111,
        nats_prefix: "helix.synthbio",
    },
    ProductMeta {
        order: 12,
        slug: "helix-lex-prime",
        title: "HelixLexPrime",
        description: "Autonomous legal & regulatory intelligence",
        tier: ProductTier::Frontier,
        maturity: ProductMaturity::Scaffold,
        default_port: 8112,
        nats_prefix: "helix.lex",
    },
    ProductMeta {
        order: 13,
        slug: "helix-cura-prime",
        title: "HelixCuraPrime",
        description: "Enterprise clinical AI platform",
        tier: ProductTier::Frontier,
        maturity: ProductMaturity::Scaffold,
        default_port: 8113,
        nats_prefix: "helix.cura",
    },
    ProductMeta {
        order: 14,
        slug: "helix-terra-prime",
        title: "HelixTerraPrime",
        description: "Precision agriculture & climate-smart farming OS",
        tier: ProductTier::Frontier,
        maturity: ProductMaturity::Scaffold,
        default_port: 8114,
        nats_prefix: "helix.terra",
    },
    ProductMeta {
        order: 15,
        slug: "helix-climate-prime",
        title: "HelixClimatePrime",
        description: "Planetary-scale climate risk modeling & net-zero orchestration",
        tier: ProductTier::Frontier,
        maturity: ProductMaturity::Scaffold,
        default_port: 8115,
        nats_prefix: "helix.climate",
    },
    ProductMeta {
        order: 16,
        slug: "helix-orbit-prime",
        title: "HelixOrbitPrime",
        description: "Commercial space operations & satellite intelligence",
        tier: ProductTier::Frontier,
        maturity: ProductMaturity::Scaffold,
        default_port: 8116,
        nats_prefix: "helix.orbit",
    },
    ProductMeta {
        order: 17,
        slug: "helix-quantum-forge",
        title: "HelixQuantumForge",
        description: "Hybrid quantum-classical computing platform",
        tier: ProductTier::Frontier,
        maturity: ProductMaturity::Scaffold,
        default_port: 8117,
        nats_prefix: "helix.quantum",
    },
    ProductMeta {
        order: 18,
        slug: "helix-vita-prime",
        title: "HelixVitaPrime",
        description: "Precision medicine & longevity research platform",
        tier: ProductTier::Frontier,
        maturity: ProductMaturity::Scaffold,
        default_port: 8118,
        nats_prefix: "helix.vita",
    },
    ProductMeta {
        order: 19,
        slug: "helix-grid-prime",
        title: "HelixGridPrime",
        description: "Autonomous smart energy systems & renewable optimization",
        tier: ProductTier::Frontier,
        maturity: ProductMaturity::Scaffold,
        default_port: 8119,
        nats_prefix: "helix.grid",
    },
    ProductMeta {
        order: 20,
        slug: "helix-nova-labs",
        title: "HelixNovaLabs",
        description: "Open scientific discovery accelerator",
        tier: ProductTier::Frontier,
        maturity: ProductMaturity::Scaffold,
        default_port: 8120,
        nats_prefix: "helix.nova",
    },
    // --- Build LAST: after Core + products 1–20 ---
    ProductMeta {
        order: 21,
        slug: "helix-pulse",
        title: "HelixPulse",
        description: "Sovereign distributed memory & cluster data plane (modern Redis-class)",
        tier: ProductTier::Frontier,
        maturity: ProductMaturity::Scaffold,
        default_port: 8121,
        nats_prefix: "helix.pulse",
    },
];

pub fn product_by_slug(slug: &str) -> Option<&'static ProductMeta> {
    PRODUCT_CATALOG.iter().find(|p| p.slug == slug)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_orders_sequential() {
        assert_eq!(PRODUCT_CATALOG.len(), 21);
        for (i, p) in PRODUCT_CATALOG.iter().enumerate() {
            assert_eq!(p.order as usize, i + 1);
        }
        let pulse = product_by_slug("helix-pulse").expect("helix-pulse");
        assert_eq!(pulse.order, 21);
        assert_eq!(pulse.default_port, 8121);
    }
}

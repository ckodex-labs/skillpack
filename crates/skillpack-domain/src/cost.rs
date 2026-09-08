//! Code Carbon & Cost Estimation
//!
//! Like Infracost but for AI skill execution - compute, tokens, carbon footprint.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Cost estimate for skill execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostEstimate {
    /// Skill reference
    pub skill_ref: String,
    /// Estimated compute cost
    pub compute: ComputeCost,
    /// Estimated token usage
    pub tokens: TokenCost,
    /// Estimated carbon footprint
    pub carbon: CarbonFootprint,
    /// Total estimated monthly cost (USD)
    pub monthly_cost_usd: f64,
    /// Calculation timestamp
    pub estimated_at: DateTime<Utc>,
}

impl CostEstimate {
    pub fn new(skill_ref: impl Into<String>) -> Self {
        Self {
            skill_ref: skill_ref.into(),
            compute: ComputeCost::default(),
            tokens: TokenCost::default(),
            carbon: CarbonFootprint::default(),
            monthly_cost_usd: 0.0,
            estimated_at: Utc::now(),
        }
    }

    /// Calculate total monthly cost
    pub fn calculate_total(&mut self) {
        self.monthly_cost_usd = self.compute.monthly_cost_usd() + self.tokens.monthly_cost_usd();
        self.estimated_at = Utc::now();
    }

    /// Get cost tier
    pub fn tier(&self) -> CostTier {
        match self.monthly_cost_usd as u32 {
            0..=10 => CostTier::Free,
            11..=100 => CostTier::Low,
            101..=500 => CostTier::Medium,
            501..=2000 => CostTier::High,
            _ => CostTier::Enterprise,
        }
    }
}

/// Cost tier classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CostTier {
    Free,
    Low,
    Medium,
    High,
    Enterprise,
}

impl CostTier {
    pub fn emoji(&self) -> &'static str {
        match self {
            Self::Free => "🆓",
            Self::Low => "💚",
            Self::Medium => "💛",
            Self::High => "🔶",
            Self::Enterprise => "💎",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Free => "Free Tier",
            Self::Low => "$10-100/mo",
            Self::Medium => "$100-500/mo",
            Self::High => "$500-2000/mo",
            Self::Enterprise => "$2000+/mo",
        }
    }
}

/// Compute resource cost
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComputeCost {
    /// Average CPU hours per invocation
    pub cpu_hours: f64,
    /// Average GPU hours per invocation
    pub gpu_hours: f64,
    /// Average memory GB-hours per invocation
    pub memory_gb_hours: f64,
    /// Estimated invocations per month
    pub monthly_invocations: u64,
    /// Cloud provider (aws, azure, gcp)
    pub provider: Option<String>,
    /// Instance type used
    pub instance_type: Option<String>,
}

impl ComputeCost {
    /// Calculate monthly cost based on typical cloud pricing
    pub fn monthly_cost_usd(&self) -> f64 {
        // Approximate pricing (can be provider-specific)
        const CPU_HOUR_COST: f64 = 0.05;
        const GPU_HOUR_COST: f64 = 1.50;
        const MEMORY_GB_HOUR_COST: f64 = 0.01;

        let per_invocation = (self.cpu_hours * CPU_HOUR_COST)
            + (self.gpu_hours * GPU_HOUR_COST)
            + (self.memory_gb_hours * MEMORY_GB_HOUR_COST);

        per_invocation * self.monthly_invocations as f64
    }
}

/// Token usage cost (for LLM-based skills)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenCost {
    /// Average input tokens per invocation
    pub input_tokens: u64,
    /// Average output tokens per invocation
    pub output_tokens: u64,
    /// Monthly invocations
    pub monthly_invocations: u64,
    /// LLM model used
    pub model: Option<String>,
}

impl TokenCost {
    /// Calculate monthly cost based on typical LLM pricing
    pub fn monthly_cost_usd(&self) -> f64 {
        // Approximate pricing (GPT-4 tier)
        const INPUT_TOKEN_COST: f64 = 0.00003; // $30 per 1M
        const OUTPUT_TOKEN_COST: f64 = 0.00006; // $60 per 1M

        let per_invocation = (self.input_tokens as f64 * INPUT_TOKEN_COST)
            + (self.output_tokens as f64 * OUTPUT_TOKEN_COST);

        per_invocation * self.monthly_invocations as f64
    }

    /// Total tokens per invocation
    pub fn total_tokens(&self) -> u64 {
        self.input_tokens + self.output_tokens
    }
}

/// Carbon footprint estimation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CarbonFootprint {
    /// Grams of CO2 per invocation
    pub co2_grams_per_invocation: f64,
    /// Monthly CO2 in kg
    pub monthly_co2_kg: f64,
    /// Energy region (affects carbon intensity)
    pub region: Option<String>,
    /// Carbon intensity (gCO2/kWh) for the region
    pub carbon_intensity: f64,
}

impl CarbonFootprint {
    /// Calculate from compute cost
    pub fn from_compute(compute: &ComputeCost, region: &str) -> Self {
        // Carbon intensity by region (gCO2/kWh)
        let intensity = match region {
            "us-west" | "us-ca" => 150.0,    // California - low carbon
            "eu-west" | "eu-north" => 200.0, // Europe - moderate
            "us-east" => 350.0,              // US East - moderate
            "asia-south" => 600.0,           // India - higher
            _ => 400.0,                      // Global average
        };

        // Approximate energy per CPU/GPU hour
        const CPU_KWH: f64 = 0.1;
        const GPU_KWH: f64 = 0.3;

        let kwh_per_invocation = (compute.cpu_hours * CPU_KWH) + (compute.gpu_hours * GPU_KWH);

        let co2_grams = kwh_per_invocation * intensity;
        let monthly_kg = (co2_grams * compute.monthly_invocations as f64) / 1000.0;

        Self {
            co2_grams_per_invocation: co2_grams,
            monthly_co2_kg: monthly_kg,
            region: Some(region.to_string()),
            carbon_intensity: intensity,
        }
    }

    /// Get carbon rating
    pub fn rating(&self) -> CarbonRating {
        match self.co2_grams_per_invocation as u32 {
            0..=1 => CarbonRating::A,
            2..=5 => CarbonRating::B,
            6..=15 => CarbonRating::C,
            16..=50 => CarbonRating::D,
            _ => CarbonRating::F,
        }
    }
}

/// Carbon efficiency rating
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CarbonRating {
    A, // Excellent (<1g CO2)
    B, // Good (1-5g CO2)
    C, // Average (5-15g CO2)
    D, // Poor (15-50g CO2)
    F, // Very Poor (>50g CO2)
}

impl CarbonRating {
    pub fn emoji(&self) -> &'static str {
        match self {
            Self::A => "🌱",
            Self::B => "🍃",
            Self::C => "🌿",
            Self::D => "🍂",
            Self::F => "💨",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::A => "Excellent",
            Self::B => "Good",
            Self::C => "Average",
            Self::D => "Poor",
            Self::F => "Very Poor",
        }
    }
}

/// Cost comparison between skills
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostComparison {
    pub skills: Vec<CostEstimate>,
    pub cheapest: String,
    pub greenest: String,
    pub recommended: String,
}

impl CostComparison {
    pub fn compare(estimates: Vec<CostEstimate>) -> Self {
        let cheapest = estimates
            .iter()
            .min_by(|a, b| a.monthly_cost_usd.partial_cmp(&b.monthly_cost_usd).unwrap())
            .map(|e| e.skill_ref.clone())
            .unwrap_or_default();

        let greenest = estimates
            .iter()
            .min_by(|a, b| {
                a.carbon
                    .monthly_co2_kg
                    .partial_cmp(&b.carbon.monthly_co2_kg)
                    .unwrap()
            })
            .map(|e| e.skill_ref.clone())
            .unwrap_or_default();

        // Recommended: balance of cost, carbon, and quality
        let recommended = cheapest.clone(); // Simplified

        Self {
            skills: estimates,
            cheapest,
            greenest,
            recommended,
        }
    }
}

/// Monthly cost summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlySummary {
    pub period: String,
    pub total_cost_usd: f64,
    pub total_co2_kg: f64,
    pub total_invocations: u64,
    pub skills: Vec<SkillCostBreakdown>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillCostBreakdown {
    pub skill_ref: String,
    pub cost_usd: f64,
    pub co2_kg: f64,
    pub invocations: u64,
    pub percentage_of_total: f64,
}

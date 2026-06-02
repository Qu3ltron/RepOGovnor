use crate::model::Result;
use crate::schema::CostPricingRates;

pub(crate) const REASONING_TOKENS_NOT_BILLED_SEPARATELY: &str =
    "reasoning_tokens_not_billed_separately";

pub(crate) fn credit_micros(
    input_tokens: u64,
    cached_input_tokens: u64,
    output_tokens: u64,
    reasoning_tokens: u64,
    rates: &CostPricingRates,
    reasoning_token_policy: Option<&str>,
) -> Result<u64> {
    if cached_input_tokens > input_tokens {
        return Err("cached input exceeds input tokens".to_string());
    }
    if reasoning_tokens > 0 {
        let policy = reasoning_token_policy
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                "pricing snapshot requires reasoning_token_policy when reasoning tokens are present"
                    .to_string()
            })?;
        if policy != REASONING_TOKENS_NOT_BILLED_SEPARATELY {
            return Err(format!(
                "unsupported reasoning_token_policy {policy}; supported policy is {REASONING_TOKENS_NOT_BILLED_SEPARATELY}"
            ));
        }
    }

    let uncached_input_tokens = input_tokens - cached_input_tokens;
    let input = (uncached_input_tokens as u128) * (rates.input_micros_per_million as u128);
    let cached = (cached_input_tokens as u128) * (rates.cached_input_micros_per_million as u128);
    let output = (output_tokens as u128) * (rates.output_micros_per_million as u128);
    let total = (input + cached + output) / 1_000_000;
    u64::try_from(total).map_err(|_| "calculated credit amount overflows u64".to_string())
}

//! Cost arithmetic for hint generations.
//!
//! Money is kept in whole micro-dollars (millionths of a US dollar) with integer maths only, so no
//! float rounding can drift. A fraction of a micro-dollar always rounds up, because a learner
//! should never be charged less than the provider charges us.

use crate::hints::config::Price;

/// Micro-dollars in one million tokens' worth of price. Prices are stored per million tokens.
const MICROS_PER_MILLION: i64 = 1_000_000;

/// The cost of one call, in micro-dollars, rounded up.
#[must_use]
pub fn cost_micros(price: &Price, tokens_in: u32, tokens_out: u32) -> i64 {
    let input = i64::from(tokens_in).saturating_mul(price.input_per_million_micros);
    let output = i64::from(tokens_out).saturating_mul(price.output_per_million_micros);
    input
        .saturating_add(output)
        .saturating_add(MICROS_PER_MILLION - 1)
        / MICROS_PER_MILLION
}

/// The most a call can cost before it runs, in micro-dollars.
///
/// Real tokenisers use about four bytes per token, so counting one token per two bytes gives a
/// safe upper bound on input tokens. Output is counted at the full `max_output_tokens`.
#[must_use]
pub fn worst_case_micros(price: &Price, prompt_bytes: usize, max_output_tokens: u32) -> i64 {
    let tokens_in = u32::try_from(prompt_bytes.div_ceil(2)).unwrap_or(u32::MAX);
    cost_micros(price, tokens_in, max_output_tokens)
}

/// An estimate of the token count for `text_len` bytes, rounded up at about four bytes per token.
#[must_use]
pub fn estimate_tokens(text_len: usize) -> u32 {
    u32::try_from(text_len.div_ceil(4)).unwrap_or(u32::MAX)
}

/// Writes a non-negative micro-dollar amount as a decimal with exactly six places, for the
/// `numeric(10,6)` column. For example `75_000` becomes `"0.075000"`.
#[must_use]
pub fn micros_to_sql_decimal(micros: i64) -> String {
    format!(
        "{}.{:06}",
        micros / MICROS_PER_MILLION,
        micros % MICROS_PER_MILLION
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hints::config::Price;
    use proptest::prelude::*;

    /// `OpenAI` defaults from the design: $0.10 in and $0.50 out per million tokens.
    const OPENAI: Price = Price {
        input_per_million_micros: 100_000,
        output_per_million_micros: 500_000,
    };

    #[test]
    fn one_million_input_tokens_cost_the_input_price() {
        assert_eq!(cost_micros(&OPENAI, 1_000_000, 0), 100_000);
    }

    #[test]
    fn a_thousand_in_and_five_hundred_out_tokens_cost_350_micro_dollars() {
        // 1000 * 100_000 + 500 * 500_000 = 350_000_000, divided by 1_000_000.
        assert_eq!(cost_micros(&OPENAI, 1_000, 500), 350);
    }

    #[test]
    fn any_fraction_of_a_micro_dollar_rounds_up() {
        // One input token costs 0.1 micro-dollars, which rounds up to 1.
        assert_eq!(cost_micros(&OPENAI, 1, 0), 1);
    }

    #[test]
    fn no_tokens_cost_nothing() {
        assert_eq!(cost_micros(&OPENAI, 0, 0), 0);
    }

    #[test]
    fn worst_case_uses_half_the_prompt_bytes_as_input_tokens() {
        // 1000 bytes is at most 500 input tokens. 300 output tokens at the output price.
        // 500 * 100_000 + 300 * 500_000 = 200_000_000, which is 200 micro-dollars.
        assert_eq!(worst_case_micros(&OPENAI, 1_000, 300), 200);
    }

    #[test]
    fn estimate_uses_about_four_bytes_per_token_rounded_up() {
        assert_eq!(estimate_tokens(0), 0);
        assert_eq!(estimate_tokens(4), 1);
        assert_eq!(estimate_tokens(5), 2);
    }

    #[test]
    fn micro_dollars_print_as_a_six_place_decimal() {
        assert_eq!(micros_to_sql_decimal(0), "0.000000");
        assert_eq!(micros_to_sql_decimal(1), "0.000001");
        assert_eq!(micros_to_sql_decimal(75_000), "0.075000");
        assert_eq!(micros_to_sql_decimal(5_000_000), "5.000000");
    }

    proptest! {
        // P8: the worst case is never below the real cost, for any token counts inside the bound.
        #[test]
        fn worst_case_is_never_below_the_real_cost(
            prompt_bytes in 0usize..=16_384,
            max_out in 0u32..=300,
            used_in_fraction in 0u32..=100,
            used_out in 0u32..=300,
        ) {
            let worst = worst_case_micros(&OPENAI, prompt_bytes, max_out);
            let max_in = u32::try_from(prompt_bytes.div_ceil(2)).unwrap_or(u32::MAX);
            let tokens_in = max_in * used_in_fraction / 100;
            let real = cost_micros(&OPENAI, tokens_in, used_out.min(max_out));
            prop_assert!(worst >= real);
            prop_assert!(real >= 0);
        }

        // The SQL text always has exactly six places and matches the micro-dollar value.
        #[test]
        fn decimal_text_has_six_places(micros in 0i64..=1_000_000_000) {
            let text = micros_to_sql_decimal(micros);
            let places = text.split_once('.').map(|(_, frac)| frac.len());
            prop_assert_eq!(places, Some(6));
        }
    }
}

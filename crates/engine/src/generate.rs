//! Seeded, edge-biased generation of candidate inputs.

use crate::rng::SplitMix64;
use crate::schema::Schema;
use serde_json::{json, Value};
use std::collections::{HashSet, VecDeque};

/// Tunable generator defaults. Tests check properties, not these exact numbers.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GeneratorConfig {
    /// Chance, in percent, that an integer is drawn from the edge pool.
    pub edge_int_percent: u8,
    /// Chance, in percent, that a collection length is drawn from the edge lengths.
    pub edge_length_percent: u8,
    /// The largest number of candidates the enumeration stage yields.
    pub enumeration_cap: usize,
    /// The largest number of JSON nodes a generated value should hold.
    pub max_units: usize,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            edge_int_percent: 40,
            edge_length_percent: 35,
            enumeration_cap: 150,
            max_units: 256,
        }
    }
}

/// Counts the JSON nodes in a value: scalars and containers each count as one.
#[must_use]
pub fn units(value: &Value) -> usize {
    match value {
        Value::Array(items) => 1 + items.iter().map(units).sum::<usize>(),
        Value::Object(fields) => 1 + fields.values().map(units).sum::<usize>(),
        _ => 1,
    }
}

/// Builds one value that follows `schema`, using only the seeded generator.
///
/// `size_cap` limits collection lengths when the schema allows it. The
/// result is not checked here; callers check it with `Schema::contains`.
pub fn generate_value(
    schema: &Schema,
    rng: &mut SplitMix64,
    size_cap: usize,
    config: &GeneratorConfig,
) -> Value {
    match schema {
        Schema::Int { min, max } => json!(int_value(rng, *min, *max, config)),
        Schema::Str {
            alphabet,
            min_len,
            max_len,
        } => {
            let chars: Vec<char> = alphabet.chars().collect();
            if chars.is_empty() {
                return json!("");
            }
            let length = pick_length(rng, *min_len, *max_len, size_cap, config);
            let text: String = (0..length)
                .map(|_| chars[index_below(rng, chars.len())])
                .collect();
            json!(text)
        }
        Schema::IntArray {
            min_len,
            max_len,
            min,
            max,
            sorted,
            unique,
        } => {
            let width = range_width(*min, *max);
            let max_len = if *unique {
                (*max_len).min(width)
            } else {
                *max_len
            };
            let length = pick_length(rng, *min_len, max_len, size_cap, config);
            let mut items: Vec<i64> = Vec::with_capacity(length);
            for index in 0..length {
                let item = if *unique {
                    fresh_int(rng, *min, *max, &items, config)
                } else if index > 0 && rng.gen_bool(20) {
                    items[index_below(rng, index)]
                } else {
                    int_value(rng, *min, *max, config)
                };
                items.push(item);
            }
            if *sorted {
                items.sort_unstable();
            }
            json!(items)
        }
        Schema::Matrix {
            min_rows,
            max_rows,
            min_cols,
            max_cols,
            min,
            max,
        } => {
            let rows = pick_length(rng, *min_rows, *max_rows, size_cap, config);
            let cols_cap = size_cap
                .checked_div(rows)
                .map_or(size_cap, |per_row| per_row.max(*min_cols));
            let cols = pick_length(rng, *min_cols, *max_cols, cols_cap, config);
            let grid: Vec<Vec<i64>> = (0..rows)
                .map(|_| {
                    (0..cols)
                        .map(|_| int_value(rng, *min, *max, config))
                        .collect()
                })
                .collect();
            json!(grid)
        }
        Schema::Tuple { items } => Value::Array(
            items
                .iter()
                .map(|item| generate_value(item, rng, size_cap, config))
                .collect(),
        ),
    }
}

/// Picks an integer in `min..=max`, favouring edge values when configured.
fn int_value(rng: &mut SplitMix64, min: i64, max: i64, config: &GeneratorConfig) -> i64 {
    if !rng.gen_bool(config.edge_int_percent) {
        return rng.gen_range(min, max);
    }
    let target = 0_i64.clamp(min, max);
    let candidates = [
        min,
        max,
        target,
        target.saturating_sub(1).clamp(min, max),
        target.saturating_add(1).clamp(min, max),
    ];
    candidates[index_below(rng, candidates.len())]
}

/// Picks an integer not already in `items`, falling back to a linear scan.
fn fresh_int(
    rng: &mut SplitMix64,
    min: i64,
    max: i64,
    items: &[i64],
    config: &GeneratorConfig,
) -> i64 {
    for _ in 0..50 {
        let candidate = int_value(rng, min, max, config);
        if !items.contains(&candidate) {
            return candidate;
        }
    }
    (min..=max)
        .find(|value| !items.contains(value))
        .unwrap_or(min)
}

/// Picks a collection length within the bounds, favouring edge lengths.
fn pick_length(
    rng: &mut SplitMix64,
    min_len: usize,
    max_len: usize,
    size_cap: usize,
    config: &GeneratorConfig,
) -> usize {
    let high = max_len.min(size_cap).max(min_len);
    if rng.gen_bool(config.edge_length_percent) {
        let edges = [
            min_len,
            (min_len + 1).min(high),
            2_usize.clamp(min_len, high),
            high,
        ];
        edges[index_below(rng, edges.len())]
    } else {
        to_usize(rng.gen_range(to_i64(min_len), to_i64(high)))
    }
}

/// Picks an index in `0..len`. Returns 0 for an empty range.
fn index_below(rng: &mut SplitMix64, len: usize) -> usize {
    to_usize(rng.gen_range(0, to_i64(len.saturating_sub(1))))
}

/// The number of integers in `min..=max`, saturating at `usize::MAX`.
fn range_width(min: i64, max: i64) -> usize {
    let width = i128::from(max) - i128::from(min) + 1;
    usize::try_from(width).unwrap_or(usize::MAX)
}

/// Converts a length to `i64`, saturating for absurdly large values.
fn to_i64(value: usize) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

/// Converts a sampled value back to a length. Negative values become zero.
fn to_usize(value: i64) -> usize {
    usize::try_from(value).unwrap_or(0)
}

/// Where a candidate input came from, in the order the stream yields them.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Origin {
    /// An authored known-ghost seed from the problem pack.
    KnownGhost,
    /// A public sample input from the problem pack.
    Sample,
    /// A tiny exhaustive enumeration of simple values.
    Enumerated,
    /// A seeded, edge-biased random value.
    Generated,
}

/// One input the hunt may try, with the stage that produced it.
#[derive(Clone, Debug, PartialEq)]
pub struct Candidate {
    /// The JSON input to evaluate.
    pub value: Value,
    /// The stage that produced the input.
    pub origin: Origin,
}

/// Most tiny integers the enumeration keeps for each integer range.
const TINY_INTS: usize = 4;
/// Most tiny elements an array or matrix enumeration may draw from.
const TINY_ELEMENTS: usize = 3;
/// Stops the random stage after this many repeats in a row. The space is then almost surely exhausted.
const MAX_CONSECUTIVE_DUPLICATES: u32 = 100;
/// The length cap the random stage grows toward on a timeout hunt.
const TIMEOUT_CAP: u64 = 100_000;

/// The lazy, de-duplicated sequence of candidate inputs for one hunt.
///
/// Stages run in order: known ghosts, samples, a tiny enumeration and then
/// seeded random values. Nothing is built before it is asked for.
pub struct CandidateStream {
    schema: Schema,
    rng: SplitMix64,
    config: GeneratorConfig,
    seeds: VecDeque<Candidate>,
    enumeration: Odometer,
    enumerated: usize,
    random_index: u64,
    consecutive_duplicates: u32,
    seen: HashSet<String>,
    invalid_seeds: u32,
    produced: u32,
    timeout_target: bool,
}

impl CandidateStream {
    /// Creates a stream for `schema`. Known ghosts and samples that fail
    /// `Schema::contains` are skipped and counted.
    #[must_use]
    pub fn new(
        schema: Schema,
        seed: u64,
        known_ghosts: Vec<Value>,
        samples: Vec<Value>,
        config: GeneratorConfig,
    ) -> Self {
        let mut invalid_seeds = 0;
        let mut seeds = VecDeque::new();
        for (values, origin) in [
            (known_ghosts, Origin::KnownGhost),
            (samples, Origin::Sample),
        ] {
            for value in values {
                if schema.contains(&value).is_ok() {
                    seeds.push_back(Candidate { value, origin });
                } else {
                    invalid_seeds += 1;
                }
            }
        }
        let enumeration = Odometer::new(&schema);
        Self {
            schema,
            rng: SplitMix64::new(seed),
            config,
            seeds,
            enumeration,
            enumerated: 0,
            random_index: 0,
            consecutive_duplicates: 0,
            seen: HashSet::new(),
            invalid_seeds,
            produced: 0,
            timeout_target: false,
        }
    }

    /// Lets the random stage grow lengths far beyond the normal cap, for timeout hunts.
    #[must_use]
    pub fn timeout_target(mut self) -> Self {
        self.timeout_target = true;
        self
    }

    /// Counts known ghosts and samples skipped because they broke the schema.
    #[must_use]
    pub fn invalid_seeds_skipped(&self) -> u32 {
        self.invalid_seeds
    }

    /// Counts the candidates this stream has yielded so far.
    #[must_use]
    pub fn produced(&self) -> u32 {
        self.produced
    }

    /// Returns the next candidate that has not been yielded before.
    fn next_candidate(&mut self) -> Option<Candidate> {
        while let Some(candidate) = self.seeds.pop_front() {
            if self.first_time(&candidate.value) {
                return Some(candidate);
            }
        }
        while let Some(value) = self.next_enumerated() {
            if self.first_time(&value) {
                return Some(Candidate {
                    value,
                    origin: Origin::Enumerated,
                });
            }
        }
        let value = self.next_generated()?;
        Some(Candidate {
            value,
            origin: Origin::Generated,
        })
    }

    /// Records a value and reports whether this was its first appearance.
    fn first_time(&mut self, value: &Value) -> bool {
        self.seen.insert(value.to_string())
    }

    fn next_enumerated(&mut self) -> Option<Value> {
        if self.enumerated >= self.config.enumeration_cap {
            return None;
        }
        let next = self.enumeration.next_value()?;
        self.enumerated += 1;
        Some(next)
    }

    fn next_generated(&mut self) -> Option<Value> {
        while self.consecutive_duplicates < MAX_CONSECUTIVE_DUPLICATES {
            let cap = self.random_cap();
            self.random_index += 1;
            let value = generate_value(&self.schema, &mut self.rng, cap, &self.config);
            if self.first_time(&value) {
                self.consecutive_duplicates = 0;
                return Some(value);
            }
            self.consecutive_duplicates += 1;
        }
        None
    }

    /// The length cap for the next random draw. It grows with the draw count.
    fn random_cap(&self) -> usize {
        let grown = if self.timeout_target {
            2_u64
                .saturating_add(self.random_index.saturating_mul(64))
                .min(TIMEOUT_CAP)
        } else {
            let normal = u64::try_from(self.config.max_units).unwrap_or(u64::MAX);
            (2 + self.random_index / 8).min(normal)
        };
        usize::try_from(grown).unwrap_or(usize::MAX)
    }
}

impl Iterator for CandidateStream {
    type Item = Candidate;

    fn next(&mut self) -> Option<Candidate> {
        let candidate = self.next_candidate()?;
        self.produced = self.produced.saturating_add(1);
        Some(candidate)
    }
}

/// Walks every combination of tiny values, one step at a time, like an odometer.
///
/// A tuple root yields arrays of picks. Any other root yields one pick per step.
struct Odometer {
    pools: Vec<Vec<Value>>,
    digits: Vec<usize>,
    tuple: bool,
    started: bool,
    finished: bool,
}

impl Odometer {
    fn new(schema: &Schema) -> Self {
        let (pools, tuple): (Vec<Vec<Value>>, bool) = match schema {
            Schema::Tuple { items } => (items.iter().map(tiny_pool).collect(), true),
            other => (vec![tiny_pool(other)], false),
        };
        let finished = pools.is_empty() || pools.iter().any(Vec::is_empty);
        let digits = vec![0; pools.len()];
        Self {
            pools,
            digits,
            tuple,
            started: false,
            finished,
        }
    }

    fn next_value(&mut self) -> Option<Value> {
        if self.finished {
            return None;
        }
        if self.started {
            self.advance();
            if self.finished {
                return None;
            }
        }
        self.started = true;
        Some(self.current())
    }

    fn advance(&mut self) {
        for position in (0..self.digits.len()).rev() {
            self.digits[position] += 1;
            if self.digits[position] < self.pools[position].len() {
                return;
            }
            self.digits[position] = 0;
        }
        self.finished = true;
    }

    fn current(&self) -> Value {
        let picks: Vec<Value> = self
            .pools
            .iter()
            .zip(&self.digits)
            .map(|(pool, digit)| pool[*digit].clone())
            .collect();
        if self.tuple {
            Value::Array(picks)
        } else {
            picks.into_iter().next().unwrap_or(Value::Null)
        }
    }
}

/// The simplest few values of a leaf schema, most important first.
fn tiny_pool(schema: &Schema) -> Vec<Value> {
    match schema {
        Schema::Int { min, max } => tiny_ints(*min, *max)
            .into_iter()
            .map(|v| json!(v))
            .collect(),
        Schema::Str {
            alphabet,
            min_len,
            max_len,
        } => {
            let chars: Vec<char> = alphabet.chars().take(2).collect();
            (*min_len..=(*max_len).min(2))
                .flat_map(|length| all_sequences(&chars, length))
                .map(|picked| json!(picked.into_iter().collect::<String>()))
                .filter(|value| schema.contains(value).is_ok())
                .collect()
        }
        Schema::IntArray {
            min_len,
            max_len,
            min,
            max,
            ..
        } => {
            let elements: Vec<i64> = tiny_ints(*min, *max)
                .into_iter()
                .take(TINY_ELEMENTS)
                .collect();
            (*min_len..=(*max_len).min(2))
                .flat_map(|length| all_sequences(&elements, length))
                .map(|items| json!(items))
                .filter(|value| schema.contains(value).is_ok())
                .collect()
        }
        Schema::Matrix {
            min_rows,
            max_rows,
            min_cols,
            max_cols,
            min,
            max,
        } => {
            let elements: Vec<i64> = tiny_ints(*min, *max).into_iter().take(2).collect();
            let mut grids = Vec::new();
            for rows in *min_rows..=(*max_rows).min(2) {
                for cols in *min_cols..=(*max_cols).min(2) {
                    grids.extend(grids_of(&elements, rows, cols));
                }
            }
            grids
                .into_iter()
                .filter(|value| schema.contains(value).is_ok())
                .collect()
        }
        Schema::Tuple { .. } => Vec::new(),
    }
}

/// Up to four distinct integers in range: zero, its neighbours, then the bounds.
fn tiny_ints(min: i64, max: i64) -> Vec<i64> {
    let target = 0_i64.clamp(min, max);
    let candidates = [
        target,
        target.saturating_add(1),
        target.saturating_sub(1),
        min,
        max,
    ];
    let mut ints = Vec::new();
    for candidate in candidates {
        if (min..=max).contains(&candidate) && !ints.contains(&candidate) {
            ints.push(candidate);
        }
    }
    ints.truncate(TINY_INTS);
    ints
}

/// Every grid of `rows` by `cols` cells whose cells come from `elements`.
fn grids_of(elements: &[i64], rows: usize, cols: usize) -> Vec<Value> {
    if cols == 0 {
        return vec![json!(vec![Vec::<i64>::new(); rows])];
    }
    all_sequences(elements, rows * cols)
        .into_iter()
        .map(|cells| {
            let grid: Vec<Vec<i64>> = cells.chunks(cols).map(<[i64]>::to_vec).collect();
            json!(grid)
        })
        .collect()
}

/// Every sequence of `length` items drawn from `items`, in lexical order.
fn all_sequences<T: Clone>(items: &[T], length: usize) -> Vec<Vec<T>> {
    let mut sequences: Vec<Vec<T>> = vec![Vec::new()];
    for _ in 0..length {
        sequences = sequences
            .iter()
            .flat_map(|prefix| {
                items.iter().map(move |item| {
                    let mut next = prefix.clone();
                    next.push(item.clone());
                    next
                })
            })
            .collect();
    }
    sequences
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::tests::strategies::arb_schema;
    use proptest::prelude::*;
    use serde_json::json;

    fn int_array(min_len: usize, max_len: usize, min: i64, max: i64) -> Schema {
        Schema::IntArray {
            min_len,
            max_len,
            min,
            max,
            sorted: false,
            unique: false,
        }
    }

    #[test]
    fn gen_units_counts_every_node() {
        assert_eq!(units(&json!(0)), 1);
        assert_eq!(units(&json!([0, [1, 2]])), 5);
    }

    #[test]
    fn gen_equal_seeds_give_equal_values() {
        let schema = int_array(0, 10, -5, 5);
        let config = GeneratorConfig::default();
        let mut first = SplitMix64::new(42);
        let mut second = SplitMix64::new(42);
        for _ in 0..50 {
            assert_eq!(
                generate_value(&schema, &mut first, 256, &config),
                generate_value(&schema, &mut second, 256, &config),
            );
        }
    }

    #[test]
    fn gen_edge_values_appear_over_many_draws() {
        let schema = int_array(0, 3, -5, 5);
        let config = GeneratorConfig::default();
        let mut rng = SplitMix64::new(7);
        let (mut empty, mut low, mut high, mut zero, mut duplicate) =
            (false, false, false, false, false);

        for _ in 0..500 {
            let value = generate_value(&schema, &mut rng, 256, &config);
            let items: Vec<i64> = value
                .as_array()
                .expect("array")
                .iter()
                .map(|item| item.as_i64().expect("integer"))
                .collect();
            empty |= items.is_empty();
            low |= items.contains(&-5);
            high |= items.contains(&5);
            zero |= items.contains(&0);
            duplicate |= items
                .iter()
                .enumerate()
                .any(|(index, item)| items[..index].contains(item));
        }

        assert!(empty, "no empty array in 500 draws");
        assert!(low, "minimum value never drawn");
        assert!(high, "maximum value never drawn");
        assert!(zero, "zero never drawn");
        assert!(duplicate, "no duplicate element in 500 draws");
    }

    #[test]
    fn gen_unique_sorted_arrays_are_strictly_increasing() {
        let schema = Schema::IntArray {
            min_len: 0,
            max_len: 5,
            min: -3,
            max: 3,
            sorted: true,
            unique: true,
        };
        let config = GeneratorConfig::default();
        let mut rng = SplitMix64::new(11);
        for _ in 0..200 {
            let value = generate_value(&schema, &mut rng, 256, &config);
            assert!(schema.contains(&value).is_ok(), "invalid value {value}");
        }
    }

    fn take_candidates(stream: &mut CandidateStream, count: usize) -> Vec<Candidate> {
        stream.by_ref().take(count).collect()
    }

    #[test]
    fn gen_stream_yields_stages_in_order() {
        let mut stream = CandidateStream::new(
            Schema::Int { min: -5, max: 5 },
            1,
            vec![json!(3)],
            vec![json!(4)],
            GeneratorConfig::default(),
        );
        let found: Vec<(Value, Origin)> = take_candidates(&mut stream, 4)
            .into_iter()
            .map(|candidate| (candidate.value, candidate.origin))
            .collect();
        assert_eq!(
            found,
            vec![
                (json!(3), Origin::KnownGhost),
                (json!(4), Origin::Sample),
                (json!(0), Origin::Enumerated),
                (json!(1), Origin::Enumerated),
            ]
        );
    }

    #[test]
    fn gen_invalid_seed_is_skipped_and_counted() {
        let mut stream = CandidateStream::new(
            Schema::Int { min: -5, max: 5 },
            1,
            vec![],
            vec![json!("x"), json!(2)],
            GeneratorConfig::default(),
        );
        assert_eq!(stream.invalid_seeds_skipped(), 1);
        let first = stream.next().expect("a candidate");
        assert_eq!((first.value, first.origin), (json!(2), Origin::Sample));
    }

    #[test]
    fn gen_duplicates_skip_and_stream_ends() {
        let mut stream = CandidateStream::new(
            Schema::Int { min: 0, max: 0 },
            9,
            vec![json!(0)],
            vec![json!(0)],
            GeneratorConfig::default(),
        );
        let all: Vec<Candidate> = stream.by_ref().collect();
        assert_eq!(
            all,
            vec![Candidate {
                value: json!(0),
                origin: Origin::KnownGhost,
            }]
        );
    }

    #[test]
    fn gen_stream_pulls_only_what_is_taken() {
        let mut stream = CandidateStream::new(
            Schema::Int {
                min: -1000,
                max: 1000,
            },
            3,
            vec![],
            vec![],
            GeneratorConfig::default(),
        );
        assert_eq!(take_candidates(&mut stream, 5).len(), 5);
        assert_eq!(stream.produced(), 5);
    }

    #[test]
    fn gen_enumeration_stops_at_its_cap() {
        let item = Schema::IntArray {
            min_len: 0,
            max_len: 2,
            min: -1,
            max: 1,
            sorted: false,
            unique: false,
        };
        let schema = Schema::Tuple {
            items: vec![item.clone(), item.clone(), item.clone(), item],
        };
        let mut stream =
            CandidateStream::new(schema, 5, vec![], vec![], GeneratorConfig::default());
        let taken = take_candidates(&mut stream, 200);
        let enumerated = taken
            .iter()
            .filter(|candidate| candidate.origin == Origin::Enumerated)
            .count();
        assert_eq!(enumerated, 150);
    }

    #[test]
    fn gen_equal_seeds_give_equal_streams() {
        let schema = Schema::IntArray {
            min_len: 0,
            max_len: 6,
            min: -9,
            max: 9,
            sorted: false,
            unique: false,
        };
        let mut first = CandidateStream::new(
            schema.clone(),
            77,
            vec![],
            vec![],
            GeneratorConfig::default(),
        );
        let mut second =
            CandidateStream::new(schema, 77, vec![], vec![], GeneratorConfig::default());
        assert_eq!(
            take_candidates(&mut first, 100),
            take_candidates(&mut second, 100)
        );
    }

    #[test]
    fn gen_timeout_target_lifts_the_length_cap() {
        let schema = Schema::IntArray {
            min_len: 0,
            max_len: 100_000,
            min: 0,
            max: 1,
            sorted: false,
            unique: false,
        };
        let mut stream =
            CandidateStream::new(schema, 21, vec![], vec![], GeneratorConfig::default())
                .timeout_target();
        let longest = take_candidates(&mut stream, 300)
            .iter()
            .filter_map(|candidate| candidate.value.as_array().map(Vec::len))
            .max()
            .unwrap_or(0);
        assert!(longest > 256, "longest was {longest}");
    }

    proptest! {
        #[test]
        fn gen_p1_stream_candidates_are_valid_and_distinct(
            schema in arb_schema(),
            seed in any::<u64>(),
        ) {
            let mut stream = CandidateStream::new(
                schema.clone(),
                seed,
                vec![],
                vec![],
                GeneratorConfig::default(),
            );
            let candidates = take_candidates(&mut stream, 40);
            prop_assert!(!candidates.is_empty(), "stream produced nothing");
            let mut texts = std::collections::HashSet::new();
            for candidate in &candidates {
                prop_assert!(schema.contains(&candidate.value).is_ok(), "invalid {}", candidate.value);
                prop_assert!(texts.insert(candidate.value.to_string()), "repeat {}", candidate.value);
            }
        }
    }

    proptest! {
        #[test]
        fn gen_p1_generated_values_are_valid(
            schema in arb_schema(),
            seed in any::<u64>(),
            cap in 0_usize..300,
        ) {
            let mut rng = SplitMix64::new(seed);
            let config = GeneratorConfig::default();
            let value = generate_value(&schema, &mut rng, cap, &config);
            prop_assert!(schema.contains(&value).is_ok(), "invalid value {}", value);
        }
    }
}

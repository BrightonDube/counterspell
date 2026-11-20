//! Request shape, validation and scoring for break-room scores.
//!
//! The client never sends a score number; the server computes it here from a
//! tagged request that rejects unknown fields.

use axum::http::StatusCode;
use serde::Deserialize;
use serde_json::{json, Value};
use time::{format_description::well_known::Iso8601, Date};

use crate::error::ApiError;

/// Maximum JSON size of the stored `meta` document (matches the table CHECK).
const META_MAX_BYTES: usize = 1024;

/// One score submission, tagged by `game`.
#[derive(Debug, Clone, PartialEq)]
pub enum ScoreRequest {
    ChessPuzzle {
        date: Date,
        solved: bool,
        mistakes: u32,
    },
    ChessBot {
        result: BotResult,
        moves: u32,
    },
    Brain {
        level: u32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BotResult {
    Win,
    Draw,
    Loss,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PuzzleIn {
    #[serde(rename = "game")]
    _game: serde::de::IgnoredAny,
    date: String,
    solved: bool,
    mistakes: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BotIn {
    #[serde(rename = "game")]
    _game: serde::de::IgnoredAny,
    result: BotResult,
    moves: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BrainIn {
    #[serde(rename = "game")]
    _game: serde::de::IgnoredAny,
    level: u32,
}

impl<'de> Deserialize<'de> for ScoreRequest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        let tag = value.get("game").and_then(Value::as_str).unwrap_or("");
        match tag {
            "chess_puzzle" => {
                let p: PuzzleIn =
                    serde_json::from_value(value).map_err(serde::de::Error::custom)?;
                let date = Date::parse(&p.date, &Iso8601::DATE)
                    .map_err(|_| serde::de::Error::custom("date must be YYYY-MM-DD"))?;
                Ok(Self::ChessPuzzle {
                    date,
                    solved: p.solved,
                    mistakes: p.mistakes,
                })
            }
            "chess_bot" => {
                let b: BotIn = serde_json::from_value(value).map_err(serde::de::Error::custom)?;
                Ok(Self::ChessBot {
                    result: b.result,
                    moves: b.moves,
                })
            }
            "brain" => {
                let b: BrainIn = serde_json::from_value(value).map_err(serde::de::Error::custom)?;
                Ok(Self::Brain { level: b.level })
            }
            _ => Err(serde::de::Error::custom(
                "game must be chess_puzzle, chess_bot or brain",
            )),
        }
    }
}

/// Validates the request against today's SAST date and returns the stored
/// score and meta document, or the stable error code for a rejection.
pub fn score_for(req: &ScoreRequest, today: Date) -> Result<(i32, Value), ApiError> {
    match *req {
        ScoreRequest::ChessPuzzle {
            date,
            solved,
            mistakes,
        } => {
            if date != today {
                return Err(ApiError::new(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "wrong_date",
                    "The score must be for today's puzzle (SAST date).",
                ));
            }
            if !solved {
                return Err(ApiError::new(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "not_solved",
                    "An unsolved puzzle is not a score.",
                ));
            }
            if mistakes > 20 {
                return Err(ApiError::bad_request(
                    "invalid_body",
                    "mistakes must be between 0 and 20.",
                ));
            }
            let score = 100 - 15 * i32::try_from(mistakes).unwrap_or(20);
            Ok((
                score.max(10),
                json!({ "date": date.to_string(), "mistakes": mistakes }),
            ))
        }
        ScoreRequest::ChessBot { result, moves } => {
            if !(1..=400).contains(&moves) {
                return Err(ApiError::bad_request(
                    "invalid_body",
                    "moves must be between 1 and 400.",
                ));
            }
            let (score, name) = match result {
                BotResult::Win => (100, "win"),
                BotResult::Draw => (50, "draw"),
                BotResult::Loss => (0, "loss"),
            };
            Ok((score, json!({ "result": name, "moves": moves })))
        }
        ScoreRequest::Brain { level } => {
            if !(1..=99).contains(&level) {
                return Err(ApiError::bad_request(
                    "invalid_body",
                    "level must be between 1 and 99.",
                ));
            }
            let score = (i32::try_from(level).unwrap_or(99) * 10).min(990);
            Ok((score, json!({ "level": level })))
        }
    }
}

/// The game tag stored on the row.
#[must_use]
pub fn game_tag(req: &ScoreRequest) -> &'static str {
    match req {
        ScoreRequest::ChessPuzzle { .. } => "chess_puzzle",
        ScoreRequest::ChessBot { .. } => "chess_bot",
        ScoreRequest::Brain { .. } => "brain",
    }
}

/// Re-checks the meta bound before insert so the row can never fail the CHECK.
pub fn check_meta_size(meta: &Value) -> Result<(), ApiError> {
    if meta.to_string().len() > META_MAX_BYTES {
        return Err(ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "meta_too_large",
            "The score metadata is too large.",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::Month;

    fn today() -> Date {
        Date::from_calendar_date(2026, Month::November, 13).expect("valid")
    }

    fn puzzle(date: &str, solved: bool, mistakes: u32) -> ScoreRequest {
        serde_json::from_value(json!({
            "game": "chess_puzzle", "date": date, "solved": solved, "mistakes": mistakes
        }))
        .expect("valid request")
    }

    #[test]
    fn puzzle_scores() {
        let d = today().to_string();
        assert_eq!(
            score_for(&puzzle(&d, true, 0), today()).map(|(s, _)| s),
            Ok(100)
        );
        assert_eq!(
            score_for(&puzzle(&d, true, 20), today()).map(|(s, _)| s),
            Ok(10)
        );
    }

    #[test]
    fn puzzle_rejects() {
        assert_eq!(
            score_for(&puzzle("2026-11-14", true, 0), today())
                .expect_err("wrong date")
                .code(),
            "wrong_date"
        );
        assert_eq!(
            score_for(&puzzle("2026-11-13", false, 0), today())
                .expect_err("unsolved")
                .code(),
            "not_solved"
        );
        assert!(score_for(&puzzle("2026-11-13", true, 21), today()).is_err());
    }

    #[test]
    fn unknown_field_rejected() {
        let r: Result<ScoreRequest, _> = serde_json::from_value(json!({
            "game": "chess_puzzle", "date": "2026-11-13", "solved": true,
            "mistakes": 0, "extra": 1
        }));
        assert!(r.is_err());
    }

    #[test]
    fn bot_and_brain_scores() {
        let win: ScoreRequest = serde_json::from_value(json!({
            "game": "chess_bot", "result": "win", "moves": 41
        }))
        .expect("valid");
        assert_eq!(score_for(&win, today()).map(|(s, _)| s), Ok(100));
        for level in [1, 99] {
            let b: ScoreRequest = serde_json::from_value(json!({
                "game": "brain", "level": level
            }))
            .expect("valid");
            assert!(score_for(&b, today()).is_ok());
        }
        for level in [0, 100] {
            let b: Result<ScoreRequest, _> =
                serde_json::from_value(json!({ "game": "brain", "level": level }));
            assert!(b.is_ok());
            assert!(score_for(&b.expect("parsed"), today()).is_err());
        }
    }
}

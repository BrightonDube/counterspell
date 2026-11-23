//! The single-page route table, so a fallback hit can tell a real client-side
//! route from a genuinely unknown path and answer an honest 404 for the latter.

use std::{fs, io, path::Path};

/// File name of the deployed route manifest. The web build ships it next to `200.html`.
pub const ROUTES_MANIFEST: &str = "routes.json";

/// The declared client-side routes the SPA may legitimately claim.
#[derive(Clone, Debug, Default)]
pub struct SpaRoutes {
    patterns: Vec<Vec<Segment>>,
}

#[derive(Clone, Debug, PartialEq)]
enum Segment {
    Exact(String),
    /// `*` — exactly one segment.
    Param,
    /// `**` — the rest of the path, including nothing.
    Rest,
}

impl SpaRoutes {
    /// Reads `<static_dir>/routes.json`.
    ///
    /// # Errors
    ///
    /// Returns the read error, or `InvalidData` when the file is not a valid manifest.
    pub fn load(static_dir: &Path) -> io::Result<Self> {
        let raw = fs::read_to_string(static_dir.join(ROUTES_MANIFEST))?;
        Self::parse(&raw).map_err(|message| io::Error::new(io::ErrorKind::InvalidData, message))
    }

    /// Parses `{"routes": ["/", "/problems/*", "/docs/**"]}` into a route table.
    ///
    /// # Errors
    ///
    /// Returns a message naming the malformed manifest entry.
    pub fn parse(raw: &str) -> Result<Self, String> {
        let manifest: serde_json::Value = serde_json::from_str(raw)
            .map_err(|error| format!("routes.json is not valid JSON: {error}"))?;
        let entries = manifest
            .get("routes")
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| "routes.json must contain a \"routes\" array".to_string())?;
        entries
            .iter()
            .map(|entry| {
                let route = entry
                    .as_str()
                    .ok_or_else(|| "routes.json entries must be strings".to_string())?;
                Self::pattern(route)
            })
            .collect::<Result<Vec<_>, _>>()
            .map(|patterns| Self { patterns })
    }

    fn pattern(route: &str) -> Result<Vec<Segment>, String> {
        if route == "/" {
            return Ok(Vec::new());
        }
        if !route.starts_with('/') || route.ends_with('/') {
            return Err(format!(
                "route {route:?} must start with '/' and not end with one"
            ));
        }
        let pattern = route
            .split('/')
            .skip(1)
            .map(|segment| match segment {
                "" => Err(format!("route {route:?} has an empty segment")),
                "*" => Ok(Segment::Param),
                "**" => Ok(Segment::Rest),
                text => Ok(Segment::Exact(text.to_owned())),
            })
            .collect::<Result<Vec<_>, _>>()?;
        if pattern
            .iter()
            .position(|segment| *segment == Segment::Rest)
            .is_some_and(|rest| rest + 1 != pattern.len())
        {
            return Err(format!("route {route:?} may only use ** at the end"));
        }
        Ok(pattern)
    }

    /// True when `path` is a declared client-side route. A single trailing slash
    /// is ignored (`/dashboard/` counts as `/dashboard`); interior empty segments
    /// do not, so `/settings//account` is not a route. Path segments are
    /// percent-decoded like SvelteKit's router (`/%64ashboard` is `/dashboard`),
    /// but a segment that would decode to something containing `/` or `%` keeps
    /// its raw form, so encoded separators (`/break%2Fmemory`) and double
    /// encodings never create routes the client does not have.
    #[must_use]
    pub fn is_app_route(&self, path: &str) -> bool {
        if !path.starts_with('/') {
            return false;
        }
        let mut requested: Vec<std::borrow::Cow<'_, str>> = path
            .split('/')
            .skip(1)
            .map(Self::decode_segment)
            .collect();
        if requested.last().is_some_and(|last| last.is_empty()) {
            requested.pop();
        }
        self.patterns
            .iter()
            .any(|pattern| Self::matches(pattern, &requested))
    }

    /// Decodes a path segment for route matching. Keeps the raw segment when
    /// decoding is impossible or would smuggle a `/` or `%` across a boundary
    /// the router already drew.
    fn decode_segment(segment: &str) -> std::borrow::Cow<'_, str> {
        match percent_encoding::percent_decode_str(segment).decode_utf8() {
            Ok(decoded) if !decoded.contains('/') && !decoded.contains('%') => decoded,
            _ => std::borrow::Cow::Borrowed(segment),
        }
    }

    fn matches(pattern: &[Segment], requested: &[std::borrow::Cow<'_, str>]) -> bool {
        match pattern.split_first() {
            None => requested.is_empty(),
            Some((Segment::Rest, _)) => true,
            Some((Segment::Param, rest)) => {
                !requested.is_empty() && Self::matches(rest, &requested[1..])
            }
            Some((Segment::Exact(text), rest)) => {
                requested
                    .first()
                    .is_some_and(|head| **head == text.as_str())
                    && Self::matches(rest, &requested[1..])
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SpaRoutes;

    fn routes() -> SpaRoutes {
        SpaRoutes::parse(r#"{"routes":["/","/dashboard","/problems/*","/docs/**","/problems"]}"#)
            .expect("valid manifest")
    }

    #[test]
    fn exact_and_root_routes_match() {
        let routes = routes();
        assert!(routes.is_app_route("/"));
        assert!(routes.is_app_route("/dashboard"));
        assert!(routes.is_app_route("/problems"));
        assert!(!routes.is_app_route("/settings"));
    }

    #[test]
    fn star_matches_exactly_one_segment() {
        let star_only = SpaRoutes::parse(r#"{"routes":["/problems/*"]}"#).expect("valid");
        assert!(star_only.is_app_route("/problems/two-sum"));
        assert!(!star_only.is_app_route("/problems"));
        assert!(!star_only.is_app_route("/problems/"));
        assert!(!star_only.is_app_route("/problems/a/b"));
    }

    #[test]
    fn double_star_consumes_the_rest() {
        let routes = routes();
        assert!(routes.is_app_route("/docs"));
        assert!(routes.is_app_route("/docs/a"));
        assert!(routes.is_app_route("/docs/a/b/c"));
    }

    #[test]
    fn a_trailing_slash_is_ignored_but_empty_segments_do_not_match() {
        let routes = routes();
        assert!(routes.is_app_route("/dashboard/"));
        assert!(!routes.is_app_route("/dashboard//"));
        assert!(!routes.is_app_route("//dashboard"));
    }

    #[test]
    fn decoded_segments_match_but_encoded_separators_do_not() {
        let routes = routes();
        assert!(routes.is_app_route("/%64ashboard"));
        assert!(routes.is_app_route("/problems/two%2Dsum"));
        assert!(!routes.is_app_route("/break%2Fmemory"));
        assert!(!routes.is_app_route("/%2Fdocs/a"));
        assert!(!routes.is_app_route("/%2526ashboard"));
    }

    #[test]
    fn non_slash_paths_never_match() {
        let routes = routes();
        assert!(!routes.is_app_route("*"));
        assert!(!routes.is_app_route("dashboard"));
    }

    #[test]
    fn malformed_manifests_are_rejected() {
        for raw in [
            "not json",
            "{}",
            r#"{"routes":"dashboard"}"#,
            r#"{"routes":[42]}"#,
            r#"{"routes":["dashboard"]}"#,
            r#"{"routes":["/dashboard/"]}"#,
            r#"{"routes":["/a//b"]}"#,
            r#"{"routes":["/a/**/b"]}"#,
        ] {
            assert!(SpaRoutes::parse(raw).is_err(), "{raw}");
        }
    }
}

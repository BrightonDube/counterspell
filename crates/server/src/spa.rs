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
        let routes = manifest
            .get("routes")
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| "routes.json must contain a \"routes\" array".to_string())?;
        routes
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
        route
            .split('/')
            .skip(1)
            .map(|segment| match segment {
                "" => Err(format!("route {route:?} has an empty segment")),
                "*" => Ok(Segment::Param),
                "**" => Ok(Segment::Rest),
                text => Ok(Segment::Exact(text.to_owned())),
            })
            .collect()
    }

    /// True when `path` is a declared client-side route. Empty segments — including a
    /// trailing slash — are ignored, so `/dashboard/` counts as `/dashboard`.
    #[must_use]
    pub fn is_app_route(&self, path: &str) -> bool {
        let requested: Vec<&str> = path.split('/').skip(1).filter(|s| !s.is_empty()).collect();
        self.patterns
            .iter()
            .any(|pattern| Self::matches(pattern, &requested))
    }

    fn matches(pattern: &[Segment], requested: &[&str]) -> bool {
        match pattern.split_first() {
            None => requested.is_empty(),
            Some((Segment::Rest, _)) => true,
            Some((Segment::Param, rest)) => {
                !requested.is_empty() && Self::matches(rest, &requested[1..])
            }
            Some((Segment::Exact(text), rest)) => {
                requested.first().is_some_and(|head| *head == text.as_str())
                    && Self::matches(rest, &requested[1..])
            }
        }
    }
}

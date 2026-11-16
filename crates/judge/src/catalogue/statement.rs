//! Reads the constraints section of a problem statement (problem-catalogue C10, CAT-I18).

/// Returns the bullet items under the `## Constraints` heading, in order and trimmed.
///
/// The section ends at the next heading. A statement without the section gives an empty list.
/// The grimoire validator already requires the section, so an empty result is a fallback.
#[must_use]
pub fn constraints(statement: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut in_section = false;
    for line in statement.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            in_section = trimmed == "## Constraints";
            continue;
        }
        if in_section {
            if let Some(item) = trimmed
                .strip_prefix("- ")
                .or_else(|| trimmed.strip_prefix("* "))
            {
                items.push(item.trim().to_string());
            }
        }
    }
    items
}

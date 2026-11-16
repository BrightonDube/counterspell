//! Cache identity for AI hints: a hash of the learner's code shape, and the cache key built from it.
//!
//! The AST hash uses the tree-sitter node kinds of the code, in order. Comments and whitespace are
//! not nodes that count, so a reformat or a new comment gives the same hash. Code that does not
//! parse falls back to hashing its bytes. Nothing here runs the code.

use sha2::{Digest, Sha256};
use tree_sitter::{Node, Parser};

use crate::validate::grammar_for;

/// Hashes the node kinds of `code` in `language`, skipping comments. Falls back to hashing the
/// bytes when the language is unknown or the code has syntax errors.
#[must_use]
pub fn ast_hash(language: &str, code: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    if let Some(kinds) = syntax_kinds(language, code) {
        hasher.update(b"ast\0");
        for kind in kinds {
            push_field(&mut hasher, kind.as_bytes());
        }
    } else {
        hasher.update(b"bytes\0");
        push_field(&mut hasher, code.as_bytes());
    }
    hasher.finalize().into()
}

/// Builds the cache key for one hint: the problem, the rung, the code's AST hash and the ghost
/// input. Each field is length-prefixed, so two different sets of fields never give the same
/// bytes.
#[must_use]
pub fn cache_key(problem_slug: &str, rung: u8, ast_hash: &[u8; 32], ghost_input: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    push_field(&mut hasher, problem_slug.as_bytes());
    push_field(&mut hasher, &[rung]);
    push_field(&mut hasher, ast_hash);
    push_field(&mut hasher, ghost_input.as_bytes());
    hasher.finalize().into()
}

/// Adds one field to the hash, preceded by its length.
fn push_field(hasher: &mut Sha256, bytes: &[u8]) {
    let length = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
    hasher.update(length.to_be_bytes());
    hasher.update(bytes);
}

/// The node kinds of a clean parse, in document order, with comments left out. `None` means the
/// language is unknown, the parse failed, or the tree has an error.
fn syntax_kinds(language: &str, code: &str) -> Option<Vec<&'static str>> {
    let grammar = grammar_for(language).ok()?;
    let mut parser = Parser::new();
    parser.set_language(&grammar).ok()?;
    let tree = parser.parse(code, None)?;
    if tree.root_node().has_error() {
        return None;
    }
    let mut kinds = Vec::new();
    // An explicit stack keeps deep code from using deep recursion.
    let mut pending: Vec<Node<'_>> = vec![tree.root_node()];
    while let Some(node) = pending.pop() {
        if node.kind() == "comment" {
            continue;
        }
        kinds.push(node.kind());
        let mut cursor = node.walk();
        let children: Vec<Node<'_>> = node.children(&mut cursor).collect();
        pending.extend(children.into_iter().rev());
    }
    Some(kinds)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_same_code_gives_the_same_hash_every_time() {
        let code = "def f(x):\n    return x + 1\n";

        assert_eq!(ast_hash("python", code), ast_hash("python", code));
    }

    #[test]
    fn a_different_structure_gives_a_different_hash() {
        assert_ne!(
            ast_hash("python", "x = 1\n"),
            ast_hash("python", "x = 1 + 2\n")
        );
    }

    #[test]
    fn whitespace_and_comments_do_not_change_the_python_hash() {
        let plain = "x = 1\n";
        let spaced_and_commented = "x   =   1   # the first value\n\n\n";

        assert_eq!(
            ast_hash("python", plain),
            ast_hash("python", spaced_and_commented)
        );
    }

    #[test]
    fn whitespace_and_comments_do_not_change_the_javascript_hash() {
        let plain = "let a = 1;\n";
        let spaced_and_commented = "let   a=1; // the first value\n";

        assert_eq!(
            ast_hash("javascript", plain),
            ast_hash("javascript", spaced_and_commented)
        );
    }

    #[test]
    fn an_unsupported_language_falls_back_to_hashing_the_bytes() {
        assert_ne!(ast_hash("cobol", "A"), ast_hash("cobol", "B"));
        assert_eq!(ast_hash("cobol", "A"), ast_hash("cobol", "A"));
    }

    #[test]
    fn cache_key_changes_when_any_input_changes() {
        let hash = ast_hash("python", "x = 1\n");
        let base = cache_key("count-pairs", 2, &hash, "[0, 0]");

        assert_eq!(base, cache_key("count-pairs", 2, &hash, "[0, 0]"));
        assert_ne!(base, cache_key("other-problem", 2, &hash, "[0, 0]"));
        assert_ne!(base, cache_key("count-pairs", 3, &hash, "[0, 0]"));
        assert_ne!(
            base,
            cache_key(
                "count-pairs",
                2,
                &ast_hash("python", "x = 1 + 2\n"),
                "[0, 0]"
            )
        );
        assert_ne!(base, cache_key("count-pairs", 2, &hash, "[0, 1]"));
    }

    #[test]
    fn cache_key_fields_cannot_run_together() {
        let hash = ast_hash("python", "x = 1\n");

        assert_ne!(
            cache_key("ab", 1, &hash, "c"),
            cache_key("a", 1, &hash, "bc")
        );
    }
}

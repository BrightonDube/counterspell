//! The driver-protocol signature a pack's function maps to (counterspell-mcp-power T05/T10).
//!
//! The judge writes this value to `signature.json` inside the sandbox, so the shape is part of
//! the driver protocol: `{"kind": ...}` descriptors, never the `meta.toml` type strings. It lives
//! in Grimoire so the catalogue importer and `counterspell-mcp` build it the same way.

use counterspell_grimoire::meta::Param;
use counterspell_grimoire::record::{signature_descriptor, Signature};
use serde_json::json;

fn param(name: &str, kind: &str) -> Param {
    Param {
        name: name.to_owned(),
        kind: kind.to_owned(),
    }
}

/// Every launch type maps to its protocol descriptor, in call order.
#[test]
fn each_launch_type_maps_to_its_protocol_descriptor() {
    let signature = Signature {
        function: "solve".to_owned(),
        params: vec![
            param("n", "int"),
            param("text", "str"),
            param("values", "list[int]"),
            param("grid", "list[list[int]]"),
        ],
        returns: "list[int]".to_owned(),
    };

    let descriptor = signature_descriptor(&signature).expect("every type is a launch type");

    assert_eq!(
        descriptor,
        json!({
            "protocol_version": 1,
            "function": "solve",
            "parameters": [
                {"name": "n", "type": {"kind": "int"}},
                {"name": "text", "type": {"kind": "string"}},
                {"name": "values", "type": {"kind": "list", "items": {"kind": "int"}}},
                {"name": "grid", "type": {"kind": "list", "items": {"kind": "list", "items": {"kind": "int"}}}}
            ],
            "returns": {"kind": "list", "items": {"kind": "int"}}
        })
    );
}

/// A type outside the launch set is refused with its name, never guessed at.
#[test]
fn a_type_outside_the_launch_set_is_refused_by_name() {
    let in_parameter = Signature {
        function: "f".to_owned(),
        params: vec![param("x", "float")],
        returns: "int".to_owned(),
    };
    let in_return = Signature {
        function: "f".to_owned(),
        params: vec![param("x", "int")],
        returns: "dict".to_owned(),
    };

    assert_eq!(
        signature_descriptor(&in_parameter).expect_err("float is not a launch type"),
        "float"
    );
    assert_eq!(
        signature_descriptor(&in_return).expect_err("dict is not a launch type"),
        "dict"
    );
}

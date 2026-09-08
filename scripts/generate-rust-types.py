#!/usr/bin/env python3
"""
Generate Rust types from client-model.schema.json.
Produces serde-deriving structs that mirror the canonical data model.
"""

import json
import sys
from pathlib import Path

RUST_HEADER_TEMPLATE = """// Auto-generated from {schema_path}
// Generated at: {timestamp}
// Do not edit manually. Run `make generate-types` to regenerate.

use serde::{{Deserialize, Serialize}};
use chrono::{{DateTime, Utc}};
use uuid::Uuid;

"""

TYPE_MAP = {
    "string": {
        "default": "String",
        "date-time": "DateTime<Utc>",
        "uuid": "Uuid",
        "uri": "String",
    },
    "integer": "i64",
    "number": "f64",
    "boolean": "bool",
    "array": "Vec<{}>",
    "object": "serde_json::Value",
}


def rust_type(prop: dict, prop_name: str, defs: dict) -> str:
    if "$ref" in prop:
        ref_name = prop["$ref"].split("/")[-1]
        return ref_name

    ptype = prop.get("type")
    fmt = prop.get("format", "default")

    if ptype == "string":
        if "enum" in prop:
            return pascal_case(prop_name)
        return TYPE_MAP["string"].get(fmt, "String")

    if ptype == "integer":
        return "i64"

    if ptype == "number":
        return "f64"

    if ptype == "boolean":
        return "bool"

    if ptype == "array":
        item_type = rust_type(prop.get("items", {}), f"{prop_name}_item", defs)
        return f"Vec<{item_type}>"

    if ptype == "object":
        return "serde_json::Map<String, serde_json::Value>"

    return "serde_json::Value"


def pascal_case(s: str) -> str:
    return "".join(word.capitalize() for word in s.split("_"))


def snake_case(s: str) -> str:
    result = []
    for i, ch in enumerate(s):
        if ch.isupper() and i > 0:
            result.append("_")
        result.append(ch.lower())
    return "".join(result)


RUST_KEYWORDS = {
    "as",
    "break",
    "const",
    "continue",
    "crate",
    "else",
    "enum",
    "extern",
    "false",
    "fn",
    "for",
    "if",
    "impl",
    "in",
    "let",
    "loop",
    "match",
    "mod",
    "move",
    "mut",
    "pub",
    "ref",
    "return",
    "self",
    "Self",
    "static",
    "struct",
    "super",
    "trait",
    "true",
    "type",
    "unsafe",
    "use",
    "where",
    "while",
    "async",
    "await",
    "dyn",
    "abstract",
    "become",
    "box",
    "do",
    "final",
    "macro",
    "override",
    "priv",
    "typeof",
    "unsized",
    "virtual",
    "yield",
}


def sanitize_enum_variant(v: str) -> str:
    v = v.replace("+", "Plus").replace("-", "_").replace("/", "_").replace(".", "_")
    v = v.replace("@", "_").replace(":", "_")
    if v[0].isdigit():
        v = "_" + v
    # Convert to CamelCase
    parts = v.split("_")
    return "".join(p.capitalize() for p in parts if p)


def escape_field_name(name: str) -> str:
    if name in RUST_KEYWORDS:
        return f"r#{name}"
    return name


def generate_enum(name: str, values: list) -> str:
    variants = []
    for v in values:
        variant = sanitize_enum_variant(v)
        variants.append(f'    #[serde(rename = "{v}")]')
        variants.append(f"    {variant},")
    body = "\n".join(variants)
    return f"""#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum {name} {{
{body}
}}

"""


def generate_struct(name: str, schema: dict, defs: dict) -> str:
    required = set(schema.get("required", []))
    lines = [
        "#[derive(Debug, Clone, Serialize, Deserialize)]",
        f"pub struct {name} {{",
    ]

    for prop_name, prop in schema.get("properties", {}).items():
        is_required = prop_name in required
        rtype = rust_type(prop, prop_name, defs)
        field_name = escape_field_name(snake_case(prop_name))

        if "enum" in prop and "$ref" not in prop:
            enum_name = pascal_case(prop_name)
            # enums are generated separately
            rtype = enum_name

        if prop_name == "const":
            continue  # handled by enum constraint

        if is_required:
            lines.append(f"    pub {field_name}: {rtype},")
        else:
            lines.append(f"    pub {field_name}: Option<{rtype}>,")

    lines.append("}")
    lines.append("")
    return "\n".join(lines)


def main():
    if len(sys.argv) < 3:
        print(f"Usage: {sys.argv[0]} <schema.json> <output.rs> [timestamp]")
        sys.exit(1)

    schema_path = Path(sys.argv[1])
    output_path = Path(sys.argv[2])
    timestamp = sys.argv[3] if len(sys.argv) > 3 else "unknown"
    output_path.parent.mkdir(parents=True, exist_ok=True)

    with open(schema_path) as f:
        schema = json.load(f)

    defs = schema.get("definitions", {})
    header = RUST_HEADER_TEMPLATE.format(schema_path=schema_path, timestamp=timestamp)
    output = [header]

    # First pass: collect all enums needed
    enums = {}
    for def_name, def_schema in defs.items():
        for prop_name, prop in def_schema.get("properties", {}).items():
            if "enum" in prop and "$ref" not in prop:
                enum_name = pascal_case(prop_name)
                if enum_name not in enums:
                    enums[enum_name] = prop["enum"]

    # Generate standalone enum definitions for top-level enums too
    for def_name, def_schema in defs.items():
        if def_schema.get("type") == "string" and "enum" in def_schema:
            enums[def_name] = def_schema["enum"]

    # Write enums
    for enum_name, values in enums.items():
        output.append(generate_enum(enum_name, values))

    # Write structs
    for def_name in defs:
        def_schema = defs[def_name]
        if def_schema.get("type") == "object":
            output.append(generate_struct(def_name, def_schema, defs))

    # Write root schema struct
    root_schema = {
        "properties": schema.get("properties", {}),
        "required": schema.get("required", []),
    }
    output.append(generate_struct("ClientModel", root_schema, defs))

    with open(output_path, "w") as f:
        f.write("\n".join(output))

    print(f"Generated {output_path}")


if __name__ == "__main__":
    main()

---
name: agentskills-rich
version: 1.2.0
description: Convert tabular CSV data into validated JSON records with schema inference. Use when transforming CSV exports into structured JSON, inferring column types, or validating rows against a target schema.
license: Apache-2.0
author: fixtures-team
source: https://example.com/docs/csv-json
allowed-tools: [Read, Write, Bash]
---

# CSV to JSON Conversion

This skill converts CSV files into JSON records with inferred or declared
schemas, and validates each row before emitting output.

## Workflow

1. Read the CSV header and sample the first 100 rows.
2. Infer column types (string, integer, float, boolean, ISO date).
3. Emit one JSON object per row; report rows that fail validation.

## Example

```bash
python scripts/convert.py input.csv --schema references/schema-rules.md
```

Expected output shape:

```json
{ "row": 1, "record": { "id": 42, "name": "example" }, "valid": true }
```

## Verification

Run the fixtures under evals/ and verify each produced record matches the
expected JSON in evals/expected.json. See references/schema-rules.md for the
full inference table (sha256 pinned per release).

## Deep reference

Type-inference edge cases and locale handling live in
references/schema-rules.md — read it before changing inference behavior.

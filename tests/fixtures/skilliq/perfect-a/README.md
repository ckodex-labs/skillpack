# Email Validator Skill

Validates email address format and MX record reachability for AI agent workflows.

## Installation

Add this skill to your agent's skill bundle using the CNSB CLI:

```bash
cnsb add urn:ckodex:skill:acme:prod:platform:core:email-validator@1.2.0
```

Verify installation:

```bash
skillpack assess ./
```

## Configuration

The skill accepts the following parameters in the invocation payload:

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `email` | string | required | The email address to validate |
| `dns_lookup` | boolean | true | Enable MX record verification |
| `timeout_ms` | integer | 3000 | DNS lookup timeout in milliseconds |

All parameters are passed as a JSON object in the `params` field of the skill invocation envelope.

## Examples

Basic validation without DNS:

```json
{
  "skill": "email-validator",
  "params": {
    "email": "user@example.com",
    "dns_lookup": false
  }
}
```

Full validation with DNS and extended timeout:

```json
{
  "skill": "email-validator",
  "params": {
    "email": "user@example.com",
    "dns_lookup": true,
    "timeout_ms": 5000
  }
}
```

## Response Format

Successful validation:

```json
{
  "valid": true,
  "reason": "RFC5322 syntax valid; MX record found",
  "mx_records": ["mail.example.com"]
}
```

Syntax failure:

```json
{
  "valid": false,
  "reason": "missing @ symbol"
}
```

DNS timeout (non-fatal):

```json
{
  "valid": "unknown",
  "reason": "dns_timeout"
}
```

## Compatibility

This skill requires agent runtime version 0.8.0 or later. It has no external runtime dependencies beyond a functioning DNS resolver available to the host process. The binary is statically linked and does not require any shared libraries beyond the system libc.

## Contributing

Contributions must pass the existing test suite and include tests for new edge cases. Run `cargo test` before opening a pull request. Coverage must remain at or above 90% for the validation module.

---
id: urn:ckodex:skill:acme:prod:platform:core:email-validator
version: 1.2.0
name: Email Validator
description: Validates email address format and MX record reachability
author: platform-team@acme.corp
license: Apache-2.0
tags: [validation, email, networking]
---

# Email Validator

The Email Validator skill provides two-phase email address validation for AI agent workflows. In the first phase, it performs RFC 5322 syntax checking using a deterministic regex parser. In the second phase, it performs an optional DNS MX record lookup to verify the domain can receive mail.

## Usage

Invoke with a single string argument containing the email address to validate. The skill returns a structured JSON result with a `valid` boolean, a `reason` string, and an optional `mx_records` array when DNS lookup is enabled.

## Configuration

Set `dns_lookup: false` in the invocation parameters to skip the MX phase. This is recommended for high-throughput scenarios where DNS latency is unacceptable. The timeout for DNS lookups defaults to 3000 milliseconds and can be raised up to 30000 milliseconds for environments with higher network latency.

## Error Handling

Network errors during DNS lookup are treated as non-fatal. The skill returns `valid: unknown` with a `reason` of `dns_timeout` or `dns_error` rather than failing the workflow. This prevents transient network conditions from blocking downstream tasks. The calling agent receives a structured result in all code paths — there are no unhandled panics.

## Compliance

This skill complies with RFC 5321 and RFC 5322 address format requirements. The regex pattern is derived from the W3C HTML5 email pattern with additional restrictions to exclude bare IP addresses and internationalized domain names that are not IDNA-encoded. The skill does not store email addresses between invocations; all processing is stateless and ephemeral. No PII is written to logs.

## Testing

Unit tests cover the following cases: valid simple addresses, addresses with subdomains, addresses with plus-addressing (`user+tag@example.com`), addresses with quoted local parts (`"user name"@example.com`), empty string input, null input, local parts exceeding 64 characters, domains without a TLD, domains with numeric TLDs, and addresses containing Unicode characters in the local part. Integration tests cover DNS timeout behavior using a mock resolver that simulates NXDOMAIN, SERVFAIL, and timeout conditions.

## Performance

The regex phase completes in under 50 microseconds on commodity hardware. DNS lookup latency is bounded by the configured timeout. The skill is safe to invoke concurrently from multiple goroutines; the underlying regex automaton is compiled once at load time and shared read-only.

## Versioning

This is version 1.2.0. The 1.x series maintains backwards-compatible input/output contracts. The `mx_records` field was added in 1.1.0 and is absent when `dns_lookup: false`. Consumers must treat absent fields as optional, not as errors.

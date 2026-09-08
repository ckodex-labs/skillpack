# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 1.2.x   | yes       |
| 1.1.x   | yes       |
| < 1.1   | no        |

## Reporting a Vulnerability

Report security vulnerabilities to security@acme.corp. Do not open public issues for security bugs.

Response time: 48 hours acknowledgement, 7 days remediation for critical issues.

## Threat Model

This skill processes untrusted email address strings. The primary attack surface is the regex engine. The regex pattern is a finite automaton with no backtracking, which eliminates ReDoS risk. DNS lookup results are treated as untrusted data; the skill does not follow CNAME chains beyond a depth of 2 to prevent amplification.

## Data Handling

Email addresses passed to this skill are not logged, stored, or transmitted to any external service beyond the configured DNS resolver. The skill is stateless between invocations.

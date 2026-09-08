---
id: urn:ckodex:skill:acme:dev:platform:core:broken
version: "1.0.0
name: Broken Skill
description: This frontmatter is intentionally malformed
---

# Broken Skill

This skill has malformed YAML frontmatter to test parser error handling. The `version` field opens a quoted string that is never closed, which causes any standard YAML parser to return a parse error rather than a valid document.

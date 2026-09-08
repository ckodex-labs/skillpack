---
id: urn:ckodex:skill:acme:dev:platform:core:simple-tool
version: 1.0.0
name: Simple Tool
description: A simple tool with no lifecycle hooks defined
author: dev-team@acme.corp
license: MIT
---

# Simple Tool

A minimal skill with no lifecycle management. Suitable for stateless one-shot tools that do not require installation or cleanup procedures. The skill binary is self-contained and needs no environment setup.

## Usage

Invoke directly with input parameters. No setup required. Pass a `query` string and receive a `result` string. The skill performs no side effects and does not retain state between invocations.

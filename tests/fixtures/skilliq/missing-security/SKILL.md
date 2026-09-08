---
id: urn:ckodex:skill:acme:dev:platform:core:file-processor
version: 0.3.0
name: File Processor
description: Reads and transforms structured data files
author: dev-team@acme.corp
license: MIT
---

# File Processor

Reads CSV, JSON, and YAML files and applies transformation pipelines. Supports streaming for large files to avoid loading entire datasets into memory at once. Returns a transformed document in the requested output format.

## Usage

Pass a file path and a list of transformations. Returns a transformed document. The transformation pipeline is defined as a list of named operations applied in sequence. Each operation receives the output of the previous step.

## Notes

This is a development version. Not yet ready for production deployment. The streaming mode is not yet implemented for all transformation types.

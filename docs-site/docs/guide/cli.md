# CLI Reference

## Commands

### skillpack check

Assess a skill pack and display results.

```bash
skillpack check [PATH] [OPTIONS]
```

**Arguments:**

| Argument | Description          | Default |
| -------- | -------------------- | ------- |
| `PATH`   | Path or URI to skill | `.`     |

**Options:**

| Option            | Description                      |
| ----------------- | -------------------------------- |
| `--min-score <N>` | Minimum score to pass (0-150)    |
| `--format <FMT>`  | Output format: text, json, sarif |
| `--quiet`         | Only show final score            |

**Examples:**

```bash
# Check local skill
skillpack check ./my-skill

# Remote OCI
skillpack check oci://ghcr.io/org/skill:v1

# With minimum score
skillpack check . --min-score 120
```

---

### skillpack grade

Check if a skill meets minimum grade.

```bash
skillpack grade [PATH] [OPTIONS]
```

**Options:**

| Option          | Description            | Default |
| --------------- | ---------------------- | ------- |
| `--minimum, -m` | Minimum required grade | C       |

**Exit Codes:**

| Code | Meaning             |
| ---- | ------------------- |
| 0    | Grade meets minimum |
| 1    | Grade below minimum |

**Examples:**

```bash
# Require A grade
skillpack grade . --minimum A

# CI gate
skillpack grade . --minimum S || exit 1
```

---

### skillpack report

Generate assessment report in various formats.

```bash
skillpack report [PATH] [OPTIONS]
```

**Options:**

| Option         | Description           | Default |
| -------------- | --------------------- | ------- |
| `--format, -f` | json, sarif, markdown | json    |
| `--output, -o` | Output file           | stdout  |

**Examples:**

```bash
# JSON report
skillpack report . --format json

# SARIF for GitHub
skillpack report . -f sarif -o results.sarif

# Markdown for docs
skillpack report . -f markdown -o QUALITY.md
```

---

## Environment Variables

| Variable            | Description                 |
| ------------------- | --------------------------- |
| `SKILLPACK_CACHE_DIR` | Custom cache directory      |
| `SKILLPACK_LOG_LEVEL` | Logging: trace, debug, info |
| `NO_COLOR`          | Disable colored output      |

## Exit Codes

| Code | Meaning                   |
| ---- | ------------------------- |
| 0    | Success                   |
| 1    | Assessment failed minimum |
| 2    | Invalid arguments         |
| 3    | Skill not found           |
| 4    | Network error             |

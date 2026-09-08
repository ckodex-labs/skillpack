# Broken Skill

This fixture tests malformed YAML frontmatter detection.

## Purpose

Used in the SkillPack test suite to verify that the IdentityManifest checker correctly reports a parse error when SKILL.md contains invalid YAML in the frontmatter block. The `version` field contains an unterminated quoted string, which is a well-defined YAML parse error.

## Expected Behavior

The checker should surface a parse error issue rather than silently scoring the skill. The score for the IdentityManifest dimension should reflect that the manifest could not be validated.

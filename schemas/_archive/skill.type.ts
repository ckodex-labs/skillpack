/**
 * ⚠️  TOMBSTONE — ARCHIVED FILE
 * Moved from schemas/skill.type.ts to schemas/_archive/skill.type.ts on 2026-05-30.
 * This file predates client-model.schema.json and is no longer consumed by any client runtime.
 * For canonical types, see schemas/skills-specs-next/client-model.schema.json.
 * Retained for historical reference only. Do not import in new code.
 */

/**
 * Cloud-Native Skills Bundle (CNSB) Types
 *
 * This module provides type definitions for the Cloud-Native Skills Bundle v1 specification
 * as defined in https://schemas.ckodex.org/cnsb/v1/cnsb.schema.json
 */

/**
 * Governance Assurance Level (GAL) - referenced from common types
 * Values: 0-5, where higher values require higher assurance levels
 */
export type GALValue = 0 | 1 | 2 | 3 | 4 | 5;

/**
 * Atomic Security Control (ASC) - referenced from common types
 */
export interface ASC {
  readonly id: string;
  readonly version?: string;
  readonly parameters?: Record<string, unknown>;
}

/**
 * Label Map - referenced from common types
 */
export type LabelMap = Record<string, string>;

/**
 * URN - Uniform Resource Name
 */
export type URN = string;

/**
 * Resource Metadata - referenced from common types
 */
export interface ResourceMetadata {
  readonly name: string;
  readonly version: string;
  readonly description?: string;
  readonly labels?: LabelMap;
  readonly annotations?: Record<string, string>;
  readonly createdAt?: string;
  readonly updatedAt?: string;
}

/**
 * Skill Parameter Definition
 */
export interface SkillParam {
  readonly name: string;
  readonly type?: string;
  readonly description?: string;
  readonly required?: boolean;
  readonly schemaRef?: string;
}

/**
 * Skill Example
 */
export interface SkillExample {
  readonly name: string;
  readonly description?: string;
  readonly input?: Record<string, unknown>;
  readonly expectedOutput?: Record<string, unknown>;
}

/**
 * Skill Definition
 */
export interface Skill {
  readonly id: string;
  readonly entry: string;
  readonly description?: string;
  readonly galMin: GALValue;
  readonly galMax: GALValue;
  readonly asc?: ASC[];
  readonly inputs?: SkillParam[];
  readonly outputs?: SkillParam[];
  readonly examples?: SkillExample[];
  readonly labels?: LabelMap;
}

/**
 * Lifecycle Hook Definition
 */
export interface LifecycleHook {
  readonly command?: string;
  readonly script?: string;
  readonly timeout?: string;
  readonly retries?: number;
  readonly env?: LabelMap;
}

/**
 * Lifecycle Operations
 */
export interface Lifecycle {
  readonly pack?: LifecycleHook;
  readonly unpack?: LifecycleHook;
  readonly install?: LifecycleHook;
  readonly upgrade?: LifecycleHook;
  readonly uninstall?: LifecycleHook;
  readonly verify?: LifecycleHook;
  readonly preInstall?: LifecycleHook;
  readonly postInstall?: LifecycleHook;
}

/**
 * Cloud-Native Skills Bundle (CNSB) v1
 */
export interface SkillBundle {
  readonly apiVersion: "cnsb.ckodex.org/v1";
  readonly kind: "SkillBundle";
  readonly metadata: ResourceMetadata;
  readonly skills: Skill[];
  readonly dependencies?: URN[];
  readonly lifecycle?: Lifecycle;
}

/**
 * Type guard for SkillBundle
 */
export function isSkillBundle(obj: unknown): obj is SkillBundle {
  if (typeof obj !== "object" || obj === null) {
    return false;
  }

  const bundle = obj as Record<string, unknown>;

  // Check required fields
  if (bundle.apiVersion !== "cnsb.ckodex.org/v1") {
    return false;
  }

  if (bundle.kind !== "SkillBundle") {
    return false;
  }

  if (!bundle.metadata || typeof bundle.metadata !== "object") {
    return false;
  }

  if (!Array.isArray(bundle.skills) || bundle.skills.length === 0) {
    return false;
  }

  // Validate each skill
  for (const skill of bundle.skills) {
    if (!isSkill(skill)) {
      return false;
    }
  }

  // Validate dependencies if present
  if (bundle.dependencies !== undefined) {
    if (!Array.isArray(bundle.dependencies)) {
      return false;
    }
    for (const dep of bundle.dependencies) {
      if (typeof dep !== "string") {
        return false;
      }
    }
  }

  return true;
}

/**
 * Type guard for Skill
 */
export function isSkill(obj: unknown): obj is Skill {
  if (typeof obj !== "object" || obj === null) {
    return false;
  }

  const skill = obj as Record<string, unknown>;

  // Check required fields
  if (typeof skill.id !== "string") {
    return false;
  }

  if (typeof skill.entry !== "string") {
    return false;
  }

  if (
    typeof skill.galMin !== "number" ||
    skill.galMin < 0 ||
    skill.galMin > 5
  ) {
    return false;
  }

  if (
    typeof skill.galMax !== "number" ||
    skill.galMax < 0 ||
    skill.galMax > 5
  ) {
    return false;
  }

  if (skill.galMin > skill.galMax) {
    return false;
  }

  // Validate optional fields
  if (
    skill.description !== undefined &&
    typeof skill.description !== "string"
  ) {
    return false;
  }

  if (skill.asc !== undefined && !Array.isArray(skill.asc)) {
    return false;
  }

  if (skill.inputs !== undefined && !Array.isArray(skill.inputs)) {
    return false;
  }

  if (skill.outputs !== undefined && !Array.isArray(skill.outputs)) {
    return false;
  }

  if (skill.examples !== undefined && !Array.isArray(skill.examples)) {
    return false;
  }

  if (skill.labels !== undefined && typeof skill.labels !== "object") {
    return false;
  }

  return true;
}

/**
 * Type guard for SkillParam
 */
export function isSkillParam(obj: unknown): obj is SkillParam {
  if (typeof obj !== "object" || obj === null) {
    return false;
  }

  const param = obj as Record<string, unknown>;

  // Check required fields
  if (typeof param.name !== "string") {
    return false;
  }

  // Check optional fields
  if (param.type !== undefined && typeof param.type !== "string") {
    return false;
  }

  if (
    param.description !== undefined &&
    typeof param.description !== "string"
  ) {
    return false;
  }

  if (param.required !== undefined && typeof param.required !== "boolean") {
    return false;
  }

  if (param.schemaRef !== undefined && typeof param.schemaRef !== "string") {
    return false;
  }

  return true;
}

/**
 * Type guard for SkillExample
 */
export function isSkillExample(obj: unknown): obj is SkillExample {
  if (typeof obj !== "object" || obj === null) {
    return false;
  }

  const example = obj as Record<string, unknown>;

  // Check required fields
  if (typeof example.name !== "string") {
    return false;
  }

  // Check optional fields
  if (
    example.description !== undefined &&
    typeof example.description !== "string"
  ) {
    return false;
  }

  if (example.input !== undefined && typeof example.input !== "object") {
    return false;
  }

  if (
    example.expectedOutput !== undefined &&
    typeof example.expectedOutput !== "object"
  ) {
    return false;
  }

  return true;
}

/**
 * Parse and validate a SkillBundle from JSON
 */
export function parseSkillBundle(json: unknown): SkillBundle {
  if (!isSkillBundle(json)) {
    throw new Error("Invalid SkillBundle format");
  }

  return json;
}

/**
 * Validate a SkillBundle
 */
export function validateSkillBundle(bundle: SkillBundle): {
  valid: boolean;
  errors: string[];
} {
  const errors: string[] = [];

  // Validate apiVersion
  if (bundle.apiVersion !== "cnsb.ckodex.org/v1") {
    errors.push(`Invalid apiVersion: ${bundle.apiVersion}`);
  }

  // Validate kind
  if (bundle.kind !== "SkillBundle") {
    errors.push(`Invalid kind: ${bundle.kind}`);
  }

  // Validate metadata
  if (!bundle.metadata.name) {
    errors.push("Missing metadata.name");
  }

  if (!bundle.metadata.version) {
    errors.push("Missing metadata.version");
  }

  // Validate skills
  if (!Array.isArray(bundle.skills) || bundle.skills.length === 0) {
    errors.push("Skills array must have at least one skill");
  } else {
    bundle.skills.forEach((skill, index) => {
      const skillErrors = validateSkill(skill);
      skillErrors.forEach((error) => {
        errors.push(`Skill[${index}]: ${error}`);
      });
    });
  }

  // Validate dependencies
  if (bundle.dependencies) {
    bundle.dependencies.forEach((dep, index) => {
      if (typeof dep !== "string" || !dep.startsWith("urn:")) {
        errors.push(`Dependency[${index}]: Invalid URN format`);
      }
    });
  }

  return {
    valid: errors.length === 0,
    errors,
  };
}

/**
 * Validate a Skill
 */
export function validateSkill(skill: Skill): string[] {
  const errors: string[] = [];

  // Validate required fields
  if (!skill.id) {
    errors.push("Missing id");
  }

  if (!skill.entry) {
    errors.push("Missing entry");
  }

  // Validate GAL constraints
  if (skill.galMin < 0 || skill.galMin > 5) {
    errors.push(`Invalid galMin: ${skill.galMin}`);
  }

  if (skill.galMax < 0 || skill.galMax > 5) {
    errors.push(`Invalid galMax: ${skill.galMax}`);
  }

  if (skill.galMin > skill.galMax) {
    errors.push("galMin must be <= galMax");
  }

  // Validate inputs
  if (skill.inputs) {
    skill.inputs.forEach((param, index) => {
      const paramErrors = validateSkillParam(param);
      paramErrors.forEach((error) => {
        errors.push(`Input[${index}]: ${error}`);
      });
    });
  }

  // Validate outputs
  if (skill.outputs) {
    skill.outputs.forEach((param, index) => {
      const paramErrors = validateSkillParam(param);
      paramErrors.forEach((error) => {
        errors.push(`Output[${index}]: ${error}`);
      });
    });
  }

  return errors;
}

/**
 * Validate a SkillParam
 */
export function validateSkillParam(param: SkillParam): string[] {
  const errors: string[] = [];

  if (!param.name) {
    errors.push("Missing name");
  }

  if (param.type && typeof param.type !== "string") {
    errors.push("Invalid type");
  }

  if (param.schemaRef && !isValidURI(param.schemaRef)) {
    errors.push("Invalid schemaRef URI");
  }

  return errors;
}

/**
 * Check if a string is a valid URI
 */
function isValidURI(uri: string): boolean {
  try {
    new URL(uri);
    return true;
  } catch {
    return false;
  }
}

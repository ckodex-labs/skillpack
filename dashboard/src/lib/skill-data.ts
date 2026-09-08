/**
 * Fable Skills Fleet — Skill Content Model
 *
 * Data-driven skill pages. Each skill carries identity, authority,
 * capabilities, boundaries, economics, trust, scorecard, and failure story.
 * Matches the campaign spec YAML schema.
 */

// ---------- Type definitions ----------

export interface SkillIdentity {
  kind: string;
  category: string;
  qrTarget: string;
  funFact: string;
}

export interface SkillAuthority {
  rootFabric: string;
  tenant: string;
  namespace: string;
  workspace: string;
  plane: string;
  environment: string;
  project: string;
  resource: string;
}

export interface SkillBoundaries {
  can: string[];
  cannot: string[];
}

export interface SkillEconomicsData {
  unit: string;
  estimatedCostBand: 'low' | 'medium' | 'high';
  dominantCostDrivers: string[];
  optimization: string[];
}

export interface SkillTrust {
  sbom: 'present' | 'absent' | 'optional';
  aiBom: 'present' | 'absent' | 'optional';
  signature: 'required' | 'optional' | 'absent';
  provenance: 'required' | 'optional' | 'absent';
  sandbox: string;
  network: string;
  secrets: string;
}

export interface SkillScorecard {
  capabilityFit: number;
  safetyEnvelope: number;
  evidenceQuality: number;
  interoperability: number;
  performanceReadiness: number;
}

export interface FableSkill {
  id: string;
  name: string;
  tagline: string;
  status: 'verified' | 'draft' | 'deprecated';
  gal: number;
  identity: SkillIdentity;
  authority: SkillAuthority;
  capabilities: string[];
  boundaries: SkillBoundaries;
  economics: SkillEconomicsData;
  trust: SkillTrust;
  scorecard: SkillScorecard;
  failureStory: string;
}

// ---------- Skill data ----------

const SKILLS: FableSkill[] = [
  {
    id: 'oscal-cartographer',
    name: 'OSCAL Cartographer',
    tagline: 'Turn compliance artifacts into evidence-native control maps.',
    status: 'verified',
    gal: 2,
    identity: {
      kind: 'skill',
      category: 'governance',
      qrTarget: '/skills/oscal-cartographer',
      funFact:
        'This skill treats compliance as cartography: controls are the terrain, evidence is the trail, and receipts are the compass.',
    },
    authority: {
      rootFabric: 'ckodex',
      tenant: 'noufel-labs',
      namespace: 'governance',
      workspace: 'oscal',
      plane: 'evidence',
      environment: 'gap',
      project: 'fable-skills',
      resource: 'skill.oscal-cartographer',
    },
    capabilities: [
      'Parse OSCAL catalogs and profiles',
      'Map controls to evidence artifacts',
      'Generate audit-ready evidence cards',
      'Emit signed receipts',
    ],
    boundaries: {
      can: [
        'Generate evidence maps',
        'Identify missing artifacts',
        'Produce traceability reports',
      ],
      cannot: [
        'Approve compliance autonomously',
        'Modify production systems',
        'Override missing authority',
      ],
    },
    economics: {
      unit: 'skill_run',
      estimatedCostBand: 'low',
      dominantCostDrivers: [
        'model_tokens',
        'asset_generation',
        'evidence_storage',
      ],
      optimization: [
        'cache_static_topology',
        'compress_3d_assets',
        'lazy_load_motion',
      ],
    },
    trust: {
      sbom: 'present',
      aiBom: 'optional',
      signature: 'required',
      provenance: 'required',
      sandbox: 'wasm',
      network: 'denied_by_default',
      secrets: 'denied',
    },
    scorecard: {
      capabilityFit: 92,
      safetyEnvelope: 95,
      evidenceQuality: 88,
      interoperability: 90,
      performanceReadiness: 84,
    },
    failureStory:
      'If OSCAL evidence is incomplete, the skill does not infer compliance. It emits a gap receipt, marks the coverage as incomplete, and recommends the next evidence artifact required for promotion.',
  },
  {
    id: 'semantic-tracer',
    name: 'Semantic Tracer',
    tagline: 'Trace meaning through code, claims, and evidence graphs.',
    status: 'verified',
    gal: 3,
    identity: {
      kind: 'skill',
      category: 'evidence',
      qrTarget: '/skills/semantic-tracer',
      funFact:
        'Semantic Tracer was born from the observation that most traceability tools track files, not meaning. This skill traces claims.',
    },
    authority: {
      rootFabric: 'ckodex',
      tenant: 'noufel-labs',
      namespace: 'evidence',
      workspace: 'tracer',
      plane: 'evidence',
      environment: 'gap',
      project: 'fable-skills',
      resource: 'skill.semantic-tracer',
    },
    capabilities: [
      'Trace claims across source, evidence, and proof artifacts',
      'Build semantic dependency graphs',
      'Detect orphaned evidence and unresolved claims',
      'Generate claim-resolution reports',
    ],
    boundaries: {
      can: [
        'Traverse evidence graphs',
        'Identify broken claim chains',
        'Produce semantic coverage reports',
      ],
      cannot: [
        'Modify evidence artifacts',
        'Resolve claims autonomously',
        'Access cross-tenant evidence',
      ],
    },
    economics: {
      unit: 'skill_run',
      estimatedCostBand: 'medium',
      dominantCostDrivers: [
        'model_tokens',
        'graph_traversal',
        'evidence_indexing',
      ],
      optimization: [
        'incremental_graph_updates',
        'cache_resolved_claims',
        'batch_evidence_queries',
      ],
    },
    trust: {
      sbom: 'present',
      aiBom: 'present',
      signature: 'required',
      provenance: 'required',
      sandbox: 'wasm',
      network: 'denied_by_default',
      secrets: 'denied',
    },
    scorecard: {
      capabilityFit: 88,
      safetyEnvelope: 91,
      evidenceQuality: 94,
      interoperability: 86,
      performanceReadiness: 82,
    },
    failureStory:
      'When claim chains are incomplete, the tracer does not guess missing links. It emits a gap-in-chain receipt identifying which claims lack evidence, which evidence lacks proof, and what the next attestation step should be.',
  },
  {
    id: 'agentisc-trustwall',
    name: 'AgentISC TrustWall',
    tagline: 'Enforce trust boundaries for autonomous agent operations.',
    status: 'verified',
    gal: 4,
    identity: {
      kind: 'skill',
      category: 'security',
      qrTarget: '/skills/agentisc-trustwall',
      funFact:
        'TrustWall treats every agent action as potentially hostile until it carries a valid proof chain. Trust is earned, not assumed.',
    },
    authority: {
      rootFabric: 'ckodex',
      tenant: 'noufel-labs',
      namespace: 'security',
      workspace: 'agentisc',
      plane: 'policy',
      environment: 'gap',
      project: 'fable-skills',
      resource: 'skill.agentisc-trustwall',
    },
    capabilities: [
      'Enforce GAL-level access boundaries',
      'Validate proof chains before action execution',
      'Quarantine unauthorized agent operations',
      'Emit security attestation receipts',
    ],
    boundaries: {
      can: [
        'Block unauthorized cross-tenant actions',
        'Validate cryptographic proof chains',
        'Emit quarantine and audit receipts',
      ],
      cannot: [
        'Weaken security policies autonomously',
        'Grant elevated permissions',
        'Bypass proof chain requirements',
      ],
    },
    economics: {
      unit: 'skill_run',
      estimatedCostBand: 'low',
      dominantCostDrivers: [
        'policy_evaluation',
        'proof_verification',
        'attestation_storage',
      ],
      optimization: [
        'cache_policy_decisions',
        'batch_proof_verification',
        'lazy_attestation_signing',
      ],
    },
    trust: {
      sbom: 'present',
      aiBom: 'present',
      signature: 'required',
      provenance: 'required',
      sandbox: 'wasm',
      network: 'denied_by_default',
      secrets: 'denied',
    },
    scorecard: {
      capabilityFit: 95,
      safetyEnvelope: 98,
      evidenceQuality: 96,
      interoperability: 88,
      performanceReadiness: 90,
    },
    failureStory:
      'When TrustWall encounters an agent action without a valid proof chain, it does not allow the action to proceed. It quarantines the request, emits a security incident receipt, and notifies the governance plane for human review.',
  },
];

// ---------- Data access ----------

export function getAllSkills(): FableSkill[] {
  return SKILLS;
}

export function getSkill(id: string): FableSkill | undefined {
  return SKILLS.find((s) => s.id === id);
}

export function getSkillIds(): string[] {
  return SKILLS.map((s) => s.id);
}

/** GAL level display names */
export const GAL_NAMES: Record<number, string> = {
  0: 'Manual',
  1: 'Assisted',
  2: 'Supervised',
  3: 'Conditional',
  4: 'High autonomy',
  5: 'Sovereign',
};

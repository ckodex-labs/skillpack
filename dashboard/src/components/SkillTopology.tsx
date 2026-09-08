'use client';

/**
 * Skill topology diagram — DS-3 rebuild.
 * Flow: Inputs → Skill Core → Policy Gate → Evidence → Receipt.
 *
 * GUARDRAILS §7: the diagram sits on DotBg v3; structure is ink
 * hairline; nodes are square; arrows are the typed variants —
 * kernel-flow (ink double-stroke, no head) between stages and one
 * attested edge (hash ticks + violet head) into the receipt, which
 * asserts the proof object passed as `digest`.
 */

import { DotBgSurface, formatDigest } from './ds3';

interface SkillTopologyProps {
  skillName: string;
  /** Content digest of the skill record — the proof object behind the attested edge. */
  digest: string;
}

const NODE_Y = 56;
const NODE_H = 84;
const MID_Y = NODE_Y + NODE_H / 2;

interface TopologyNode {
  x: number;
  w: number;
  title: string;
  detail: string;
  tag: string;
  kind: 'quiet' | 'kernel' | 'proof';
}

function nodeStroke(kind: TopologyNode['kind']): React.CSSProperties {
  switch (kind) {
    case 'quiet':
      return { stroke: 'var(--ck-hairline-strong)', strokeWidth: 1 };
    case 'kernel':
      return { stroke: 'var(--ck-fg-1)', strokeWidth: 1 };
    case 'proof':
      return { stroke: 'var(--ck-proof)', strokeWidth: 1.5 };
    default: {
      const exhaustive: never = kind;
      return exhaustive;
    }
  }
}

/* kernel-flow — ink double-stroke, no head */
function KernelFlow({ x1, x2 }: { x1: number; x2: number }) {
  return (
    <g style={{ stroke: 'var(--ck-fg-1)' }} strokeWidth={1}>
      <line x1={x1} y1={MID_Y - 2} x2={x2} y2={MID_Y - 2} />
      <line x1={x1} y1={MID_Y + 2} x2={x2} y2={MID_Y + 2} />
    </g>
  );
}

/* attested — ink line + hash ticks + violet head (requires a proof object) */
function AttestedEdge({ x1, x2 }: { x1: number; x2: number }) {
  const ticks = [0.3, 0.5, 0.7].map((t) => x1 + (x2 - x1) * t);
  return (
    <g>
      <line x1={x1} y1={MID_Y} x2={x2 - 8} y2={MID_Y} style={{ stroke: 'var(--ck-fg-1)' }} strokeWidth={1} />
      {ticks.map((tx) => (
        <line
          key={tx}
          x1={tx - 2}
          y1={MID_Y - 4}
          x2={tx + 2}
          y2={MID_Y + 4}
          style={{ stroke: 'var(--ck-fg-1)' }}
          strokeWidth={1}
        />
      ))}
      <polygon
        points={`${x2 - 8},${MID_Y - 5} ${x2},${MID_Y} ${x2 - 8},${MID_Y + 5}`}
        style={{ fill: 'var(--ck-proof)' }}
      />
    </g>
  );
}

export function SkillTopology({ skillName, digest }: SkillTopologyProps) {
  const slug = skillName.replace(/\s/g, '-');
  const titleId = `topology-title-${slug}`;
  const descId = `topology-desc-${slug}`;

  const nodes: TopologyNode[] = [
    { x: 20, w: 150, title: 'Inputs', detail: 'catalogs · profiles', tag: 'source', kind: 'quiet' },
    { x: 214, w: 170, title: 'Skill Core', detail: skillName, tag: 'kernel', kind: 'kernel' },
    { x: 428, w: 150, title: 'Policy Gate', detail: 'OPA · GAL check', tag: 'gate', kind: 'quiet' },
    { x: 622, w: 150, title: 'Evidence', detail: 'DCA · attestation', tag: 'proof', kind: 'proof' },
    { x: 816, w: 170, title: 'Receipt', detail: formatDigest(digest), tag: 'artifact', kind: 'quiet' },
  ];

  return (
    <div className="topology-container">
      <DotBgSurface />
      <svg
        role="img"
        aria-labelledby={`${titleId} ${descId}`}
        viewBox="0 0 1006 216"
        className="topology-svg"
        xmlns="http://www.w3.org/2000/svg"
      >
        {/* React 19 requires a single string child for <title>; an
            interpolation array causes a hydration mismatch. */}
        <title id={titleId}>{`${skillName} topology`}</title>
        <desc id={descId}>
          Data flow: inputs enter the skill core, pass through a policy gate, produce evidence, and
          emit a signed receipt.
        </desc>

        {nodes.map((node) => (
          <g key={node.title}>
            <rect
              x={node.x}
              y={NODE_Y}
              width={node.w}
              height={NODE_H}
              style={{ fill: 'var(--ck-bg-1)', ...nodeStroke(node.kind) }}
            />
            {node.kind === 'kernel' ? (
              <rect
                x={node.x + 3}
                y={NODE_Y + 3}
                width={node.w - 6}
                height={NODE_H - 6}
                style={{ fill: 'none', stroke: 'var(--ck-fg-1)' }}
                strokeWidth={1}
              />
            ) : null}
            <text
              x={node.x + node.w / 2}
              y={NODE_Y + 30}
              textAnchor="middle"
              style={{
                fill: 'var(--ck-fg-1)',
                fontSize: 13,
                fontWeight: 600,
                fontFamily: 'var(--ck-ff-ui)',
              }}
            >
              {node.title}
            </text>
            <text
              x={node.x + node.w / 2}
              y={NODE_Y + 50}
              textAnchor="middle"
              style={{ fill: 'var(--ck-fg-3)', fontSize: 11, fontFamily: 'var(--ck-ff-mono)' }}
            >
              {node.detail}
            </text>
            <text
              x={node.x + node.w / 2}
              y={NODE_Y + 70}
              textAnchor="middle"
              style={{
                fill: node.kind === 'proof' ? 'var(--ck-proof)' : 'var(--ck-fg-mute)',
                fontSize: 11,
                letterSpacing: '0.12em',
                fontFamily: 'var(--ck-ff-mono)',
              }}
            >
              {node.tag.toUpperCase()}
            </text>
          </g>
        ))}

        <KernelFlow x1={170} x2={214} />
        <KernelFlow x1={384} x2={428} />
        <KernelFlow x1={578} x2={622} />
        <AttestedEdge x1={772} x2={816} />

        <line
          x1={20}
          y1={182}
          x2={986}
          y2={182}
          style={{ stroke: 'var(--ck-hairline)' }}
          strokeWidth={1}
        />
        <text
          x={20}
          y={202}
          style={{ fill: 'var(--ck-fg-3)', fontSize: 11, fontFamily: 'var(--ck-ff-mono)' }}
        >
          topology · {skillName} · inputs → core → gate → evidence → receipt
        </text>
      </svg>
    </div>
  );
}

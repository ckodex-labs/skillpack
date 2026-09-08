'use client';

import type { SkillAuthority } from '../lib/skill-data';

interface AuthorityPathProps {
  authority: SkillAuthority;
}

const PATH_LABELS: { key: keyof SkillAuthority; label: string }[] = [
  { key: 'rootFabric', label: 'root' },
  { key: 'tenant', label: 'tenant' },
  { key: 'namespace', label: 'ns' },
  { key: 'workspace', label: 'ws' },
  { key: 'plane', label: 'plane' },
  { key: 'environment', label: 'env' },
  { key: 'project', label: 'proj' },
  { key: 'resource', label: 'resource' },
];

export function AuthorityPath({ authority }: AuthorityPathProps) {
  return (
    <nav
      className="authority-path"
      aria-label="Authority path"
    >
      {PATH_LABELS.map((item, idx) => (
        <span key={item.key} style={{ display: 'contents' }}>
          <span className="authority-node">
            <span className="authority-node-label">{item.label}</span>
            <span className="authority-node-value">
              {authority[item.key]}
            </span>
          </span>
          {idx < PATH_LABELS.length - 1 && (
            <span className="authority-separator" aria-hidden="true">
              →
            </span>
          )}
        </span>
      ))}
    </nav>
  );
}

'use client';

import type { SkillBoundaries } from '../lib/skill-data';

interface CapabilityListProps {
  capabilities: string[];
  boundaries: SkillBoundaries;
}

/* ⊢ asserted · ⊘ denied — from the CNDL safe-set; a denial is
   governance, not an emergency, so both render in ink/tone. */
function CapabilityItems({ items, denied = false }: { items: string[]; denied?: boolean }) {
  return (
    <ul className="capability-list" role="list">
      {items.map((item) => (
        <li key={item} className={`capability-item${denied ? ' capability-cannot' : ''}`}>
          <span className="capability-glyph" aria-hidden="true">
            {denied ? '⊘' : '⊢'}
          </span>
          <span className="capability-item-text">{item}</span>
        </li>
      ))}
    </ul>
  );
}

export function CapabilityList({ capabilities, boundaries }: CapabilityListProps) {
  return (
    <div className="section-grid">
      <div className="section-card">
        <div className="ck-label section-label">Capabilities</div>
        <CapabilityItems items={capabilities} />
      </div>

      <div className="section-card">
        <div className="ck-label section-label">Safe operating bounds</div>

        <div style={{ marginBottom: 'var(--ck-sp-4)' }}>
          <div className="ck-label" style={{ marginBottom: 'var(--ck-sp-2)' }}>
            Can
          </div>
          <CapabilityItems items={boundaries.can} />
        </div>

        <div>
          <div className="ck-label" style={{ marginBottom: 'var(--ck-sp-2)' }}>
            Cannot
          </div>
          <CapabilityItems items={boundaries.cannot} denied />
        </div>
      </div>
    </div>
  );
}

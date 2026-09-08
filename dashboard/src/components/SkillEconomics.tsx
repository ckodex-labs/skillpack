'use client';

import type { SkillEconomicsData } from '../lib/skill-data';

interface SkillEconomicsProps {
  economics: SkillEconomicsData;
}

/* Cost band is routine state — tone and gauge length, never
   green/amber/red. */
const BAND_PERCENT: Record<SkillEconomicsData['estimatedCostBand'], number> = {
  low: 25,
  medium: 55,
  high: 85,
};

export function SkillEconomics({ economics }: SkillEconomicsProps) {
  const percent = BAND_PERCENT[economics.estimatedCostBand];

  return (
    <div className="section-card">
      <div className="economics-header">
        <div className="ck-label" style={{ margin: 0 }}>
          Skill economics
        </div>
        <span className="trust-badge trust-badge--asserted">
          {economics.estimatedCostBand} cost
        </span>
      </div>

      <div
        className="economics-gauge"
        role="meter"
        aria-label="Cost band indicator"
        aria-valuenow={percent}
        aria-valuemin={0}
        aria-valuemax={100}
      >
        <div className="economics-gauge-fill" style={{ width: `${percent}%` }} />
      </div>

      <div className="ck-annot" style={{ marginBottom: 'var(--ck-sp-3)' }}>
        unit: {economics.unit}
      </div>

      <div style={{ marginBottom: 'var(--ck-sp-4)' }}>
        <div className="ck-label" style={{ marginBottom: 'var(--ck-sp-2)' }}>
          Dominant cost drivers
        </div>
        <ul className="economics-drivers">
          {economics.dominantCostDrivers.map((driver) => (
            <li key={driver} className="economics-driver">
              {driver.replace(/_/g, ' ')}
            </li>
          ))}
        </ul>
      </div>

      <div>
        <div className="ck-label" style={{ marginBottom: 'var(--ck-sp-2)' }}>
          Optimization
        </div>
        <ul className="economics-drivers">
          {economics.optimization.map((opt) => (
            <li key={opt} className="economics-driver">
              {opt.replace(/_/g, ' ')}
            </li>
          ))}
        </ul>
      </div>
    </div>
  );
}

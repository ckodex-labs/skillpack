'use client';

import { DimensionScore } from '../lib/client';

export interface DimensionChartProps {
  dimensions: DimensionScore[];
  title?: string;
}

/* Dimension scores as horizontal bars. Hand-rolled markup instead of
   recharts: the chart library hard-codes hue and shadowed tooltips
   that cannot resolve through the DS-3 token budget. Ink is data;
   value is carried by length and the numeric label, never hue. */
export function DimensionChart({ dimensions, title }: DimensionChartProps) {
  return (
    <div>
      {title && (
        <div className="ck-label" style={{ marginBottom: 'var(--ck-sp-4)' }}>
          {title}
        </div>
      )}
      {dimensions.map((dim) => {
        const max = dim.maxScore || 100;
        const ratio = max > 0 ? dim.score / max : 0;
        return (
          <div key={dim.dimensionId} className="dim-bar-row">
            <div className="dim-bar-head">
              <span>{dim.dimensionName}</span>
              <span className="dim-bar-score">
                {dim.score.toFixed(1)}/{max}
              </span>
            </div>
            <div
              className="dim-bar-track"
              role="meter"
              aria-label={`${dim.dimensionName} score`}
              aria-valuenow={dim.score}
              aria-valuemin={0}
              aria-valuemax={max}
            >
              <div
                className={`dim-bar-fill${ratio < 0.6 ? ' dim-bar-fill--low' : ''}`}
                style={{ width: `${Math.min(100, ratio * 100)}%` }}
              />
            </div>
          </div>
        );
      })}
    </div>
  );
}

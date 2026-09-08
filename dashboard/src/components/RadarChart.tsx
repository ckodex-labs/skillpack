'use client';

import { DimensionScore } from '../lib/client';

interface RadarChartProps {
  dimensions: DimensionScore[];
  size?: number;
}

/* Radar web as inline SVG painted through --ck-* tokens (the old
   canvas version hard-coded hex and could not follow themes).
   Structure is hairline, data is ink — no hue in a chart. */
export function RadarChart({ dimensions, size = 300 }: RadarChartProps) {
  const centerX = size / 2;
  const centerY = size / 2;
  const radius = size * 0.35;
  const numPoints = dimensions.length;
  const angleStep = (2 * Math.PI) / Math.max(1, numPoints);

  const pointAt = (index: number, value: number) => {
    const angle = -Math.PI / 2 + index * angleStep;
    return {
      x: centerX + Math.cos(angle) * radius * value,
      y: centerY + Math.sin(angle) * radius * value,
    };
  };

  const dataPoints = dimensions.map((dim, i) => pointAt(i, dim.score / 100));
  const polygon = dataPoints.map((p) => `${p.x.toFixed(1)},${p.y.toFixed(1)}`).join(' ');

  return (
    <div className="radar-chart" style={{ display: 'flex', justifyContent: 'center' }}>
      <svg
        width={size}
        height={size}
        viewBox={`0 0 ${size} ${size}`}
        role="img"
        aria-label={`Radar chart of ${numPoints} dimension scores`}
      >
        {[1, 2, 3, 4].map((ring) => (
          <circle
            key={ring}
            cx={centerX}
            cy={centerY}
            r={(radius * ring) / 4}
            fill="none"
            style={{ stroke: 'var(--ck-hairline)' }}
          />
        ))}

        {dimensions.map((dim, i) => {
          const axis = pointAt(i, 1);
          const label = pointAt(i, 1 + 22 / radius);
          return (
            <g key={dim.dimensionId}>
              <line
                x1={centerX}
                y1={centerY}
                x2={axis.x}
                y2={axis.y}
                style={{ stroke: 'var(--ck-hairline-strong)' }}
              />
              <text
                x={label.x}
                y={label.y}
                textAnchor="middle"
                dominantBaseline="middle"
                style={{
                  fill: 'var(--ck-fg-3)',
                  fontFamily: 'var(--ck-ff-mono)',
                  fontSize: 'var(--ck-fs-micro)',
                }}
              >
                {dim.dimensionName.slice(0, 10)}
              </text>
            </g>
          );
        })}

        <polygon
          points={polygon}
          style={{
            fill: 'color-mix(in oklab, var(--ck-fg-1) 14%, transparent)',
            stroke: 'var(--ck-fg-1)',
            strokeWidth: 1.5,
          }}
        />

        {dataPoints.map((p, i) => (
          <rect
            key={i}
            x={p.x - 2.5}
            y={p.y - 2.5}
            width={5}
            height={5}
            style={{ fill: 'var(--ck-fg-1)' }}
          />
        ))}
      </svg>
    </div>
  );
}

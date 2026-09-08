'use client';

import React from 'react';
import type { Grade, DimensionScore } from '../lib/client';

export interface ScorecardProps {
  skillName: string;
  version: string;
  grade: Grade;
  totalScore: number;
  dimensions: DimensionScore[];
  issueCount: number;
  /** Rubric the grade was computed under (cnsb | agentskills). */
  profile?: 'cnsb' | 'agentskills';
}

/* Rank carries no hue — higher grades carry heavier ink:
   S+/S invert, A/B take the 2px contour, C/D sit in tone,
   F renders as the contradicted fill. */
function gradeVariant(grade: Grade): string {
  switch (grade) {
    case 'S+':
    case 'S':
      return 'grade-stamp--top';
    case 'A':
    case 'B':
      return 'grade-stamp--mid';
    case 'C':
    case 'D':
      return 'grade-stamp--low';
    case 'F':
      return 'grade-stamp--fail';
    default: {
      const exhaustive: never = grade;
      return exhaustive;
    }
  }
}

export function SkillScorecard({
  skillName,
  version,
  grade,
  totalScore,
  dimensions,
  issueCount,
  profile,
}: ScorecardProps) {
  return (
    <div className="scorecard">
      <div
        style={{
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'flex-start',
          gap: 'var(--ck-sp-4)',
        }}
      >
        <div>
          <h2 className="ck-h3" style={{ margin: 0 }}>
            {skillName}
          </h2>
          {version ? <span className="ck-annot">v{version}</span> : null}
          {profile ? (
            <span className="ck-annot" title="Assessment rubric">
              {version ? ' · ' : ''}
              {profile}
            </span>
          ) : null}
        </div>
        <div className={`grade-stamp ${gradeVariant(grade)}`} title={`Grade ${grade}`}>
          {grade}
        </div>
      </div>

      <div style={{ marginTop: 'var(--ck-sp-4)', display: 'flex', gap: 'var(--ck-sp-6)' }}>
        <div>
          <div className="ck-label">Score</div>
          <div className="scorecard-metric-value">{totalScore.toFixed(1)}</div>
        </div>
        <div>
          <div className="ck-label">Issues</div>
          <div className="scorecard-metric-value">{issueCount}</div>
        </div>
      </div>

      <div style={{ marginTop: 'var(--ck-sp-5)' }}>
        {dimensions.map((dim) => {
          const ratio = dim.maxScore > 0 ? dim.score / dim.maxScore : 0;
          return (
            <div key={dim.dimensionId} className="dim-bar-row">
              <div className="dim-bar-head">
                <span>{dim.dimensionName}</span>
                <span className="dim-bar-score">
                  {dim.score}/{dim.maxScore}
                </span>
              </div>
              <div className="dim-bar-track">
                <div
                  className={`dim-bar-fill${ratio < 0.6 ? ' dim-bar-fill--low' : ''}`}
                  style={{ width: `${Math.min(100, ratio * 100)}%` }}
                />
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}

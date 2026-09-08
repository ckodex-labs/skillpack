'use client';

import React, { useEffect, useState, useSyncExternalStore, Suspense } from 'react';
import { useSearchParams } from 'next/navigation';
import { getAllSkills } from '../lib/skill-data';
import {
  NavLinks,
  SkillCard,
  SkillScorecard,
  DimensionChart,
  IssuesList,
  RadarChart,
} from './index';
import {
  AuthorityFooter,
  Button,
  EvidenceMargin,
  PageShell,
  ThemeSwitch,
  type Receipt,
} from './ds3';
import { skillpackClient, type AssessmentResult } from '../lib/client';

function utcStamp(iso?: string): string {
  const d = iso ? new Date(iso) : new Date();
  return d.toISOString().slice(0, 16).replace('T', ' ') + ' UTC';
}

/* Client render timestamp for receipts. Captured once per page load;
   empty during SSR so hydration stays consistent. */
let clientRenderStamp = '';
function readRenderStamp(): string {
  if (!clientRenderStamp) clientRenderStamp = utcStamp();
  return clientRenderStamp;
}
const subscribeNever = () => () => {};

function fleetReceipts(renderedAt: string): Receipt[] {
  const skills = getAllSkills();
  const verified = skills.filter((s) => s.status === 'verified').length;
  return [
    {
      time: renderedAt,
      event: 'generated · fleet gallery',
      details: [`${skills.length} skills registered`],
    },
    {
      time: renderedAt,
      event: 'evaluated · trust posture',
      details: [`${verified}/${skills.length} verified · signature + provenance required`],
    },
  ];
}

function assessmentReceipts(
  renderedAt: string,
  assessment: AssessmentResult | null,
  error: string | null,
): Receipt[] {
  if (error) {
    return [{ time: renderedAt, event: 'rejected · assessment unavailable', details: [error] }];
  }
  if (!assessment) {
    return [{ time: renderedAt, event: 'generated · assessment requested' }];
  }
  return [
    {
      time: utcStamp(assessment.assessedAt),
      event: 'evaluated · workspace assessment',
      details: [
        `grade ${assessment.grade} · score ${assessment.totalScore.toFixed(1)}`,
        `${assessment.issues.length} issues`,
      ],
    },
  ];
}

function FableDashboardContent() {
  const searchParams = useSearchParams();
  const skills = getAllSkills();
  const currentTab = searchParams.get('view') === 'assessment' ? 'assessment' : 'fleet';

  const [assessment, setAssessment] = useState<AssessmentResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [viewMode, setViewMode] = useState<'bars' | 'radar'>('bars');
  const renderedAt = useSyncExternalStore(subscribeNever, readRenderStamp, () => '');

  const loading = currentTab === 'assessment' && !assessment && !error;

  useEffect(() => {
    if (currentTab !== 'assessment' || assessment || error) return undefined;
    let cancelled = false;
    skillpackClient
      .assess('.')
      .then((data) => {
        if (!cancelled) setAssessment(data);
      })
      .catch((err) => {
        console.error('Failed to load assessment:', err);
        if (!cancelled)
          setError(
            'The skillPack assessment service did not respond. Verify it is running on http://localhost:50051.',
          );
      });
    return () => {
      cancelled = true;
    };
  }, [currentTab, assessment, error]);

  const receipts =
    currentTab === 'fleet'
      ? fleetReceipts(renderedAt)
      : assessmentReceipts(renderedAt, assessment, error);

  return (
    <PageShell
      variant="console"
      brand="CKODEX"
      crumb="Fable Skills Fleet"
      headerRight={<ThemeSwitch />}
      nav={<NavLinks />}
      margin={<EvidenceMargin entries={receipts} />}
      footer={<AuthorityFooter level="internal" environment="gap" mode="enforce" />}
    >
      {currentTab === 'fleet' ? (
        <>
          <header className="fleet-header">
            <div className="ck-label fleet-eyebrow">CKODEX · Fable Skills Fleet</div>
            <h1 className="ck-display fleet-title">Fable Skills Fleet</h1>
            <p className="fleet-subtitle">
              A governed fleet of skill hero pages and live workspace assessment.
            </p>
          </header>
          <section aria-label="Skills fleet">
            <div className="fleet-grid">
              {skills.map((skill) => (
                <SkillCard key={skill.id} skill={skill} />
              ))}
            </div>
          </section>
        </>
      ) : (
        <section aria-label="Workspace assessment">
          {loading ? (
            <div className="loading-state">
              <span className="progress-indicator">assessing …</span>
              <p>Running repository skillpack assessment.</p>
            </div>
          ) : error ? (
            <div className="empty-state">
              <p>{error}</p>
            </div>
          ) : !assessment ? (
            <div className="empty-state">
              <p>No assessment data available.</p>
            </div>
          ) : (
            <>
              <div className="dashboard-header">
                <h2 className="ck-h2" style={{ margin: 0 }}>
                  Active workspace profile
                </h2>
                <div className="ck-row-2" role="group" aria-label="Chart view">
                  <Button
                    variant={viewMode === 'bars' ? 'quiet' : 'ghost'}
                    aria-pressed={viewMode === 'bars'}
                    onClick={() => setViewMode('bars')}
                  >
                    Dimension bars
                  </Button>
                  <Button
                    variant={viewMode === 'radar' ? 'quiet' : 'ghost'}
                    aria-pressed={viewMode === 'radar'}
                    onClick={() => setViewMode('radar')}
                  >
                    Radar web
                  </Button>
                </div>
              </div>

              <div className="dashboard-grid">
                <SkillScorecard
                  skillName={assessment.skillId}
                  version=""
                  grade={assessment.grade}
                  totalScore={assessment.totalScore}
                  dimensions={assessment.dimensions}
                  issueCount={assessment.issues.length}
                />

                <div className="chart-panel">
                  {viewMode === 'bars' ? (
                    <DimensionChart dimensions={assessment.dimensions} title="Dimension scores" />
                  ) : (
                    <RadarChart dimensions={assessment.dimensions} />
                  )}
                </div>
              </div>

              <IssuesList issues={assessment.issues} />
            </>
          )}
        </section>
      )}
    </PageShell>
  );
}

export function FableDashboard() {
  return (
    <Suspense
      fallback={
        <div className="loading-state">
          <span className="progress-indicator">loading …</span>
          <p>Loading dashboard.</p>
        </div>
      }
    >
      <FableDashboardContent />
    </Suspense>
  );
}

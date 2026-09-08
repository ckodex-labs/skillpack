'use client';

import { Issue } from '../lib/client';

interface IssuesListProps {
  issues: Issue[];
}

/* Severity is encoded in ink weight, never traffic-light color:
   error → ⊭ (contradicted, full ink), warning → △, info → ○ (tone).
   Red is reserved for quarantine / active emergency protocols. */
function severityGlyph(severity: Issue['severity']): string {
  switch (severity) {
    case 'error':
      return '⊭';
    case 'warning':
      return '△';
    case 'info':
      return '○';
    default: {
      const exhaustive: never = severity;
      return exhaustive;
    }
  }
}

export function IssuesList({ issues }: IssuesListProps) {
  if (issues.length === 0) {
    return (
      <div className="issues-empty">
        <span aria-hidden="true">⊢ </span>
        no issues found
      </div>
    );
  }

  return (
    <div className="issues-list">
      <div className="ck-label" style={{ marginBottom: 'var(--ck-sp-4)' }}>
        Issues ({issues.length})
      </div>
      <ul>
        {issues.map((issue, idx) => (
          <li key={idx} className={`issue issue-${issue.severity}`}>
            <span className="severity-glyph" aria-hidden="true">
              {severityGlyph(issue.severity)}
            </span>
            <div className="issue-content">
              <p className="issue-message">{issue.message}</p>
              {issue.path && (
                <span className="issue-location">
                  {issue.path}
                  {issue.line && `:${issue.line}`}
                </span>
              )}
              <span className="issue-dimension">{issue.dimensionId}</span>
            </div>
          </li>
        ))}
      </ul>
    </div>
  );
}

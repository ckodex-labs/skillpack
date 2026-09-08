import Link from 'next/link';
import { AuthorityFooter, PageShell, ThemeSwitch } from '../../components/ds3';

export const metadata = {
  title: 'Guide — Fable Skills Fleet',
  description:
    'Architecture overview, deployment guide, accessibility policy, and evidence model for the Fable Skills Fleet.',
};

export default function GuidePage() {
  return (
    <PageShell
      variant="reading"
      brand="CKODEX"
      crumb="Deployment guide"
      headerRight={<ThemeSwitch />}
      footer={<AuthorityFooter level="internal" environment="gap" mode="enforce" />}
    >
      <div className="ck-measure">
        <Link href="/" className="back-link">
          ← Back to fleet
        </Link>

        <h1 className="ck-h1" style={{ margin: '0 0 var(--ck-sp-3)' }}>
          Fable Skills Fleet Guide
        </h1>
        <p style={{ color: 'var(--ck-fg-3)', margin: '0 0 var(--ck-sp-8)' }}>
          Architecture, deployment, accessibility, and evidence documentation for the governed
          skill gallery.
        </p>

        <section className="guide-section" aria-label="Architecture overview">
          <h2 className="ck-h2">Architecture overview</h2>
          <p>
            Fable Skills Fleet is a <strong>Next.js 16</strong> application with React 19. Each
            skill page is data-driven from a content model.
          </p>
          <p>
            The design system is <strong>CKODEX-DS-3 &quot;Evidence Editorial&quot;</strong>,
            vendored from the ckodex-design skill. All color resolves through the closed semantic
            budget (paper · ink · tone · rust · violet · red); every page is a shell with real
            landmarks and a persistent Evidence Margin; the Authority Footer states the handling
            level. Four themes are mandatory: ledger (default), vault, hc, and automatic
            forced-colors.
          </p>
          <pre className="guide-code">{`src/
├── app/
│   ├── ckodex-ds3-core.css      # Vendored DS-3 tokens + themes + type
│   ├── ckodex-ds3-surfaces.css  # Vendored DS-3 surfaces + shells
│   ├── globals.css              # App layout on --ck-* tokens
│   ├── page.tsx                 # Fleet home — console shell
│   ├── guide/page.tsx           # This guide — reading shell
│   └── skills/
│       └── [skillId]/
│           ├── page.tsx         # Skill hero — document shell
│           ├── evidence/        # Evidence sub-page
│           └── topology/        # Topology sub-page
├── components/
│   ├── ds3/                     # DS-3 component ports (PageShell,
│   │                            #   EvidenceMargin, AuthorityFooter, …)
│   ├── SkillHero.tsx            # Hero with one focal action
│   ├── SkillCard.tsx            # Fleet card — quiet square surface
│   ├── SkillTopology.tsx        # SVG topology on DotBg v3
│   └── ...
├── lib/
│   ├── skill-data.ts            # Skill content model
│   ├── digest.ts                # Content digests (proof objects)
│   └── client.ts                # API client (assessment)
└── generated/                   # Generated type definitions`}</pre>
        </section>

        <section className="guide-section" aria-label="Build process">
          <h2 className="ck-h2">Build process</h2>
          <pre className="guide-code">{`# Install dependencies
npm install

# Development server
npm run dev

# Production build
npm run build

# Start production server
npm start`}</pre>
          <p>
            The generated output is expected in <code>.next/</code> for Next.js deployment, or{' '}
            <code>out/</code> if static export is configured.
          </p>
        </section>

        <section className="guide-section" aria-label="Local development">
          <h2 className="ck-h2">Local development</h2>
          <p>
            Run <code>npm run dev</code> and open <code>http://localhost:3000</code>. The
            development server supports hot module replacement.
          </p>
          <div className="guide-callout">
            <strong>Themes:</strong> the header switch cycles <code>ledger</code> /{' '}
            <code>vault</code> / <code>hc</code> via <code>data-theme</code>. Forced-colors is
            never gated: emulate it in Chrome DevTools with{' '}
            <code>forced-colors: active</code>, and test <code>prefers-reduced-motion: reduce</code>{' '}
            the same way.
          </div>
        </section>

        <section className="guide-section" aria-label="Asset generation policy">
          <h2 className="ck-h2">Asset generation policy</h2>
          <ul>
            <li>
              <strong>Original assets only.</strong> All generated images and 3D assets must be
              original work, not copies of protected material.
            </li>
            <li>
              <strong>Store prompt receipts.</strong> Every AI-generated asset must include a
              prompt receipt documenting what was requested.
            </li>
            <li>
              <strong>Store image receipts.</strong> Generated images carry metadata linking back
              to the generation prompt and model version.
            </li>
            <li>
              <strong>Mood-board only for inspiration.</strong> External visual references may be
              used as mood-boards but never copied directly.
            </li>
          </ul>
        </section>

        <section className="guide-section" aria-label="Accessibility policy">
          <h2 className="ck-h2">Accessibility policy</h2>
          <p>
            Target: <strong>WCAG 2.2 AAA</strong>. Minimum: <strong>AA</strong>.
          </p>
          <ul>
            <li>
              <strong>Reduced motion:</strong> animations respect{' '}
              <code>prefers-reduced-motion: reduce</code>. No rotation, pulsing, or glow anywhere;
              motion tiers are 160/280/640&nbsp;ms on the governed ease.
            </li>
            <li>
              <strong>Keyboard navigation:</strong> all interactive elements are focusable and
              operable with keyboard alone. Focus rings come from the theme tokens.
            </li>
            <li>
              <strong>Semantic landmarks:</strong> every page is a shell with{' '}
              <code>&lt;header&gt;</code>, <code>&lt;nav&gt;</code>, <code>&lt;main&gt;</code>,{' '}
              <code>&lt;aside&gt;</code> (the Evidence Margin), and <code>&lt;footer&gt;</code>{' '}
              (the Authority Footer).
            </li>
            <li>
              <strong>Color independence:</strong> no information is conveyed by color alone. The
              semantic budget pairs every hue with a glyph and a text label.
            </li>
            <li>
              <strong>Contrast:</strong> APCA targets |Lc| ≥ 75 for 11–13&nbsp;px text, ≥ 60 for
              14&nbsp;px and above. Type floor: 11&nbsp;px.
            </li>
            <li>
              <strong>Skip link:</strong> a hidden skip-to-content link is provided for screen
              reader and keyboard users.
            </li>
          </ul>
        </section>

        <section className="guide-section" aria-label="Performance budget">
          <h2 className="ck-h2">Performance budget</h2>
          <div className="section-card" style={{ marginBottom: 'var(--ck-sp-4)' }}>
            {[
              { label: 'LCP', value: '≤ 2500ms' },
              { label: 'INP', value: '≤ 200ms' },
              { label: 'CLS', value: '≤ 0.1' },
              { label: 'Initial JS', value: '≤ 180 KB' },
            ].map((row) => (
              <div
                key={row.label}
                style={{
                  display: 'flex',
                  justifyContent: 'space-between',
                  padding: 'var(--ck-sp-2) 0',
                  borderBottom: '1px solid var(--ck-hairline)',
                }}
              >
                <span className="ck-label">{row.label}</span>
                <span className="ck-evidence">{row.value}</span>
              </div>
            ))}
          </div>
        </section>

        <section className="guide-section" aria-label="Evidence model">
          <h2 className="ck-h2">Evidence model</h2>
          <p>Every skill page is backed by a content model carrying:</p>
          <ul>
            <li>
              <strong>Identity:</strong> kind, category, QR target, field note
            </li>
            <li>
              <strong>Authority:</strong> full CKODEX authority chain from root fabric to resource
            </li>
            <li>
              <strong>Trust:</strong> SBOM, AI-BOM, signature, provenance, sandbox, network,
              secrets
            </li>
            <li>
              <strong>Scorecard:</strong> capability fit, safety envelope, evidence quality,
              interoperability, performance readiness
            </li>
            <li>
              <strong>Failure story:</strong> what safe failure looks like
            </li>
          </ul>
          <p>Required evidence receipts for promotion:</p>
          <ul>
            <li>Build receipt</li>
            <li>Asset generation receipt</li>
            <li>Accessibility receipt</li>
            <li>Performance receipt</li>
            <li>Policy decision receipt</li>
            <li>Provenance receipt</li>
            <li>Deployment preview receipt</li>
            <li>Promotion receipt</li>
          </ul>
        </section>

        <section className="guide-section" aria-label="Cloudflare deployment">
          <h2 className="ck-h2">Cloudflare deployment guide</h2>
          <h3 className="ck-h3">Recommended target</h3>
          <p>
            Use <strong>Cloudflare Pages</strong> for the static skill hero fleet. Cloudflare
            Workers with Static Assets can be added later if the site requires edge logic, APIs,
            authenticated previews, or dynamic personalization.
          </p>

          <h3 className="ck-h3">Build</h3>
          <pre className="guide-code">{`npm install
npm run build`}</pre>

          <h3 className="ck-h3">Preview locally</h3>
          <pre className="guide-code">{`npx wrangler pages dev .next`}</pre>

          <h3 className="ck-h3">Deploy preview</h3>
          <pre className="guide-code">{`npx wrangler pages deploy .next \\
  --project-name fable-skills-fleet \\
  --branch preview`}</pre>

          <h3 className="ck-h3">Deploy production (gated)</h3>
          <div className="guide-callout guide-callout-warning">
            <strong>Production deployment is gated.</strong> Required receipts before promotion:
            build, accessibility, performance, policy decision, provenance, rollback, and human
            promotion approval.
          </div>
          <pre className="guide-code">{`# After all receipts are present and human approval is obtained:
npx wrangler pages deploy .next \\
  --project-name fable-skills-fleet \\
  --branch main`}</pre>
        </section>

        <section className="guide-section" aria-label="Rollback guide">
          <h2 className="ck-h2">Rollback guide</h2>
          <p>
            Use the Cloudflare dashboard or Wrangler deployment history to restore a previous
            known-good deployment.
          </p>
          <div className="guide-callout guide-callout-warning">
            <strong>Secrets:</strong> no secrets should be committed to the repository. Use
            Cloudflare dashboard variables or CI-scoped secrets for deployment tokens.
          </div>
        </section>

        <section className="guide-section" aria-label="Promotion gates">
          <h2 className="ck-h2">Promotion gates</h2>

          <h3 className="ck-h3">Preview gate</h3>
          <ul>
            <li>Build passed</li>
            <li>Guide route exists</li>
            <li>No secret leakage</li>
            <li>QR targets safe</li>
            <li>Accessibility minimum AA</li>
            <li>Provenance receipts present</li>
          </ul>

          <h3 className="ck-h3">Canary gate</h3>
          <ul>
            <li>Lighthouse budget passed</li>
            <li>Reduced motion passed</li>
            <li>Signed build attestation</li>
            <li>Policy bundle passed</li>
          </ul>

          <h3 className="ck-h3">Production gate</h3>
          <ul>
            <li>Human approval</li>
            <li>WCAG AAA or exception receipt</li>
            <li>Rollback plan present</li>
            <li>Cloudflare preview verified</li>
            <li>Analytics privacy reviewed</li>
            <li>Campaign risk accepted</li>
          </ul>
        </section>
      </div>
    </PageShell>
  );
}

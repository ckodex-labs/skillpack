# Ckodex Design System

**CKODEX-DS-3 · v3.0.0** — "Evidence Editorial"
Governance-first design system for AI safety and regulated infrastructure tooling. Token-driven, proof-native, APCA-themed. The identity is an **evidence ledger**: warm paper, ink structure, a closed semantic color budget, and the Evidence Margin as the signature primitive.

> v3 promoted 2026-06-11 from `explorations/DS-3 Evidence Editorial - Exploration.html` (two feedback rounds, all recommendations locked). `styles.css` is the canonical entry point. DS-2 is archived at `archive/ds2-styles.css` (+ `archive/ds2-preview/`); the DS-1 substrate stays at `colors_and_type.css`. The legacy deep blue is **demoted, not deleted** — it survives as the Vault theme's ground.

The constitution:

1. **The palette is the policy** — color is an assertion.
2. **The margin keeps the books** — evidence is never a tab.
3. **A violet mark requires a proof object.**
4. **A red mark requires an emergency protocol.**

---

## 1 · Product context

Ckodex (ckodex.com) makes governance infrastructure for AI safety and regulated systems. The product surface centres on three concerns:

| Concern | What it is | Visual vocabulary |
|---|---|---|
| **Kernels** | Sealed computation crates that enforce invariants | octagon marks, ink double-stroke edges |
| **Proofs** | PCA → UCA → Rekor attestation chains | violet seals, receipts, digest law, hash ticks |
| **Governance** | Mode, policy, and deployment gates | claim chips, gates, the Evidence Margin |

Because the product is a regulated tool chain — not a consumer app — every visual element is load-bearing. A component that looks decorative is a packaging defect.

---

## 2 · Index

| File / folder | Purpose |
|---|---|
| `README.md` | You are here. |
| `GUARDRAILS.md` | The DS-3 constitution as lintable assertions. |
| `SKILL.md` | Agent-Skills front-matter for use in Claude Code. |
| `styles.css` | **Canonical entry point** — DS-3 tokens, four themes, type, surfaces, controls, evidence vocabulary, page shells. Link this. |
| `dotbg.js` | DotBg v2 engineering-canvas generator (ledger / vault / hc). |
| `components/` | React components (+ `.d.ts`): Button, QuietCard, StateChip, EvidenceHash, EvidenceMargin, ProvenanceStamp, PageShell. |
| `preview/` | DS-3 preview cards (Design System tab). |
| `assets/logos/` | All logo SVGs. D (retired) is not shipped. |
| `fonts/` | Self-hosted JetBrains Mono (+ retired DS-1 families). |
| `explorations/` | The DS-3 RFC exploration that produced this canon. |
| `archive/` | DS-2 stylesheet + preview cards, frozen. |
| `colors_and_type.css` | DS-1 v1.4.0 substrate — archived, do not link for new work. |
| `redesign-v2/` · `ui_kits/governance-console/` | DS-2-era prototypes; they keep their DS-2 look via pinned imports. |

---

## 3 · Content fundamentals

### Voice

Ckodex copy is **terse, declarative, and proof-literate**. Verbs are specific (`attest`, `promote`, `deny`, `seal`), not marketing ones.

- **Pronoun:** neither *you* nor *we*. The subject is the artifact or the invariant. "The kernel refuses the call" — not "we protect you."
- **Casing:** sentence case everywhere except `.ck-label` (ALL-CAPS mono, letter-spaced).
- **Tense:** simple present. Specs describe behaviour, not promises.
- **Numbers:** always precise. `1.793:1`, `10 px`, `304 px`. Never "around".
- **Emoji:** forbidden. `✓ ✗` forbidden. Glyphs come from the CNDL safe-set (see GUARDRAILS §6).

### Verbatim invariants

Canonical sentences render **literally, never paraphrased**. Flagship:

> `mode changes deployment, not governance semantics`

Typeset via `.ck-invariant` — 11 px ink JetBrains Mono 600. Any paraphrase, even tightening, is a regression.

---

## 4 · Visual foundations

### The semantic budget

Six roles, closed set — nothing else carries hue. **The palette is the policy.**

| Token | Ledger | Vault | Role |
|---|---|---|---|
| `paper` | `#F6F1E8` | `#0A1322` | Ground |
| `ink` | `#211B14` | `#EAE5DA` | Text · structure · data |
| `tone` | `#6E6457` | `#9A9284` | Routine state — pass/idle/info are tone, not green |
| `accent.rust` | `#B4532A` | `#D2693A` | User intent · focal action · **≤2 per view** |
| `proof.violet` | `#6D28D9` | `#A78BFA` | Cryptographic proof exists — attestations only |
| `emergency.red` | `#B91C1C` | `#F87171` | Containment · quarantine · active EP only |

Demotions: teal = structural stroke in Vault only; yellow retires; lavender → violet.

### Themes — four, mandatory

`data-theme="ledger|vault|hc"` + automatic forced-colors. Ledger (paper) is default and canonical; Vault is the operational night-shift theme carrying the legacy deep blue; HC is black/white with gold focus; forced-colors remaps to system keywords and always wins. Every design must resolve in all four.

### Typography

- **Instrument Serif** (display) — speaks rarely; one weight, scale carries rank.
- **Geist** (interface) — 400/500/600. Quiet, never decorative.
- **JetBrains Mono** (evidence) — self-hosted from `fonts/`, **ligatures off**; receipts, digests, labels, invariants, CNDL notation.

Scale: display `clamp(40–64px)` · h1 32 · h2 24 · h3 18 · body 15 · body-sm 13.5 · caption 12.5 · **floor 11 px**. APCA: `|Lc| ≥ 75` for 11–13 px, `≥ 60` for 14 px+.

### Geometry & spacing

- **Quiet is square.** Default container = square + 1 px hairline (`.ck-quiet`).
- **The chamfer means sealed.** The 10 px cut corner appears only on sealed/attested surfaces (`.ck-sealed`, 2 px contour). The cut corner is vocabulary, not decoration.
- Spacing: 4 px proof-grid (`--ck-sp-*`). Registers: nav 232 px · margin 304 px · measure 72 ch.
- Backgrounds are **flat** — paper carries no vignette, no gradients-as-decoration, no drop shadows.

### The Evidence Margin — signature primitive

A persistent `<aside>` (280–320 px, always rightmost, never a tab) holding **receipts**: `generated · transformed · evaluated · rejected · approved · exported · signed`. Violet rule = proof object exists; red rule = active EP. The **provenance stamp** gates export: the artifact leaves with an evidence envelope, or it does not leave. Digest law: `sha256:9f3c…a217` — prefix required, middle-ellipsis, operable.

**Time Machine:** because the margin keeps the books, any past state is reconstructable — a replay cursor scrubs the receipt timeline and *derives* the artifact's state at that instant (see `preview/component-time-machine.html`). The cursor never edits the ledger.

### Claim states

Six, closed set — never "detected": `⊢ observed · ⇝ inferred · ○ claimed · ◆ attested · ⊭ contradicted · ⊘ quarantined`. Only attested earns violet; only quarantined earns red.

### Page shells — composition is semantic

Every page is a shell with real landmarks (`header / nav / main / aside / footer`):

- **console** — nav rail · main · Evidence Margin. Operator surfaces.
- **document** — main · Evidence Margin. Editorial/report surfaces.
- **reading** — single 72 ch column; receipts collapse to the footer.

CSS: `.ck-shell`, `.ck-shell--document`, `.ck-shell--reading` + `__header/__nav/__main/__margin/__footer`, `.ck-masthead`, `.ck-nav__item`. React: `PageShell`.

### Motion

Resolve (1.8 s, four beats) stays the single ceremony; tiers micro 160 / state 280 / scene 640 ms on the governed ease. No rotation, pulsing, or glow. One kinetic focal element per surface, max.

### Diagrams & DotBg

Diagrams sit on **DotBg v2** (`dotbg.js`): ledger = flat paper, ink dots at 0.12, tone furniture; vault = deep ink + subtle vignette; hc = explicit 0.60 dots. The budget (rust/violet/red) never appears in a background. Edges are the five typed arrows — the attested head is now **violet** (it asserts a proof object); emergency is **red**. See `preview/brand-diagrams.html`.

---

## 5 · Components (React)

All require `styles.css`. Each has a `.d.ts` next to it.

| Component | Purpose |
|---|---|
| `Button` | `primary` (rust, ≤2/view) · `quiet` · `ghost` · `emergency` (red, EP only) |
| `QuietCard` | Default square container; `sealed` renders the chamfered contour |
| `StateChip` | The six claim states with safe-set glyphs |
| `EvidenceHash` | Operable digest under the digest law — click to copy, ⊛ to inspect |
| `EvidenceMargin` | The signature primitive — receipts + provenance stamp |
| `ProvenanceStamp` | The ExportGate's signature |
| `PageShell` | console / document / reading shells with semantic landmarks |

---

## 6 · Caveats & substitutions

- **Geist + Instrument Serif load from Google Fonts** (`@import` in `styles.css`). JetBrains Mono is self-hosted. If the system must run offline, supply Geist/Instrument Serif files and we will pin `@font-face` rules.
- **No live code / no Figma:** the DS-2-era console kit is a faithful reconstruction. Attach the real console repo or Figma to tighten new DS-3 surfaces against the source of truth.
- **CNDL 2.0 spec and primitives.md** were referenced in the original guardrails but never attached — notation rules are encoded from the summary.

See `GUARDRAILS.md` for the full set of lintable assertions.

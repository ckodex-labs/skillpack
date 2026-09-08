/* CKODEX-DS-3 · ProvenanceStamp — the ExportGate's signature.
   The artifact leaves with an evidence envelope, or it does not leave.
   Violet because a stamp asserts a proof object exists. Requires the
   vendored DS-3 stylesheets. */

export interface ProvenanceStampProps {
  digest: string;
  verb?: string;
  className?: string;
}

export function ProvenanceStamp({ digest, verb = 'sealed', className = '' }: ProvenanceStampProps) {
  const m = digest.match(/^([a-z0-9-]+:)(.+)$/i);
  const prefix = m ? m[1] : '';
  const hex = m ? m[2] : digest;
  const display = prefix + (hex.length > 12 ? `${hex.slice(0, 4)}…${hex.slice(-4)}` : hex);
  return (
    <span className={`ck-stamp ${className}`.trim()} title={digest}>
      <span aria-hidden="true">◆</span>
      {verb} · {display}
    </span>
  );
}

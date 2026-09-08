/* CKODEX-DS-3 · digest format law — algorithm prefix required,
   middle-ellipsis truncation: sha256:9f3c…a217. Shared by server
   and client components. */

export function formatDigest(digest: string): string {
  const m = digest.match(/^([a-z0-9-]+:)(.+)$/i);
  const prefix = m ? m[1] : '';
  const hex = m ? m[2] : digest;
  return prefix + (hex.length > 12 ? `${hex.slice(0, 4)}…${hex.slice(-4)}` : hex);
}

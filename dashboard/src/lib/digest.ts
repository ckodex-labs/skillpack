/* Content digests for skill records — the proof object behind the
   sealed AuthorityFooter, the provenance stamp, and evidence hashes.
   A content digest asserts exactly what it is: sha256 over the
   canonical JSON of the skill record. Server-only (node:crypto);
   import from server components, never from client components. */

import { createHash } from 'node:crypto';
import type { FableSkill } from './skill-data';

export function skillContentDigest(skill: FableSkill): string {
  const canonical = JSON.stringify(skill);
  return 'sha256:' + createHash('sha256').update(canonical).digest('hex');
}

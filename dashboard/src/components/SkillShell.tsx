import type { ReactNode } from 'react';
import type { FableSkill } from '../lib/skill-data';
import { AuthorityFooter, PageShell, ThemeSwitch } from './ds3';
import { SkillMargin } from './SkillMargin';

interface SkillShellProps {
  skill: FableSkill;
  /** Content digest of the skill record. */
  digest: string;
  /** UTC timestamp at which the digest was generated. */
  generatedAt: string;
  crumb: string;
  children: ReactNode;
}

/* Document shell for skill pages: main · Evidence Margin · Authority
   Footer. The footer is `sealed` (violet) only when the record is
   attested — verified with required signature and provenance — and
   then it carries the content digest as its proof object. */
export function SkillShell({ skill, digest, generatedAt, crumb, children }: SkillShellProps) {
  const attested =
    skill.status === 'verified' &&
    skill.trust.signature === 'required' &&
    skill.trust.provenance === 'required';

  return (
    <PageShell
      variant="document"
      brand="CKODEX"
      crumb={crumb}
      urn={skill.authority.resource}
      headerRight={<ThemeSwitch />}
      margin={<SkillMargin skill={skill} digest={digest} generatedAt={generatedAt} />}
      footer={
        attested ? (
          <AuthorityFooter
            level="sealed"
            digest={digest}
            authority={skill.authority.rootFabric}
            policySet={skill.authority.namespace}
          />
        ) : (
          <AuthorityFooter
            level="internal"
            authority={skill.authority.rootFabric}
            policySet={skill.authority.namespace}
            environment={skill.authority.environment}
            mode="enforce"
          />
        )
      }
    >
      {children}
    </PageShell>
  );
}

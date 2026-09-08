import { notFound } from 'next/navigation';
import { getSkill, getSkillIds } from '../../../../lib/skill-data';
import { skillContentDigest } from '../../../../lib/digest';
import { SkillShell } from '../../../../components/SkillShell';
import { EvidencePage } from './EvidencePage';

interface EvidencePageProps {
  params: Promise<{ skillId: string }>;
}

export async function generateStaticParams() {
  return getSkillIds().map((id) => ({ skillId: id }));
}

export async function generateMetadata({ params }: EvidencePageProps) {
  const { skillId } = await params;
  const skill = getSkill(skillId);
  if (!skill) return { title: 'Not found' };
  return {
    title: `Evidence — ${skill.name} — Fable Skills Fleet`,
    description: `Trust posture and evidence for ${skill.name}`,
  };
}

export default async function Page({ params }: EvidencePageProps) {
  const { skillId } = await params;
  const skill = getSkill(skillId);

  if (!skill) {
    notFound();
  }

  const digest = skillContentDigest(skill);
  const generatedAt = new Date().toISOString().slice(0, 16).replace('T', ' ') + ' UTC';

  return (
    <SkillShell
      skill={skill}
      digest={digest}
      generatedAt={generatedAt}
      crumb={`${skill.name} · evidence`}
    >
      <EvidencePage skill={skill} digest={digest} />
    </SkillShell>
  );
}

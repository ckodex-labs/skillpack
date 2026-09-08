import { notFound } from 'next/navigation';
import { getSkill, getSkillIds } from '../../../../lib/skill-data';
import { skillContentDigest } from '../../../../lib/digest';
import { SkillShell } from '../../../../components/SkillShell';
import { TopologyPage } from './TopologyPage';

interface TopologyPageProps {
  params: Promise<{ skillId: string }>;
}

export async function generateStaticParams() {
  return getSkillIds().map((id) => ({ skillId: id }));
}

export async function generateMetadata({ params }: TopologyPageProps) {
  const { skillId } = await params;
  const skill = getSkill(skillId);
  if (!skill) return { title: 'Not found' };
  return {
    title: `Topology — ${skill.name} — Fable Skills Fleet`,
    description: `Data flow topology for ${skill.name}`,
  };
}

export default async function Page({ params }: TopologyPageProps) {
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
      crumb={`${skill.name} · topology`}
    >
      <TopologyPage skill={skill} digest={digest} />
    </SkillShell>
  );
}

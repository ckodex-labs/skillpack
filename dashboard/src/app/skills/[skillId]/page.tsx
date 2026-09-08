import { notFound } from 'next/navigation';
import { getSkill, getSkillIds } from '../../../lib/skill-data';
import { skillContentDigest } from '../../../lib/digest';
import { SkillShell } from '../../../components/SkillShell';
import { SkillHeroPage } from './SkillHeroPage';

interface SkillPageProps {
  params: Promise<{ skillId: string }>;
}

export async function generateStaticParams() {
  return getSkillIds().map((id) => ({ skillId: id }));
}

export async function generateMetadata({ params }: SkillPageProps) {
  const { skillId } = await params;
  const skill = getSkill(skillId);
  if (!skill) return { title: 'Skill not found' };
  return {
    title: `${skill.name} — Fable Skills Fleet`,
    description: skill.tagline,
  };
}

export default async function SkillPage({ params }: SkillPageProps) {
  const { skillId } = await params;
  const skill = getSkill(skillId);

  if (!skill) {
    notFound();
  }

  const digest = skillContentDigest(skill);
  const generatedAt = new Date().toISOString().slice(0, 16).replace('T', ' ') + ' UTC';

  return (
    <SkillShell skill={skill} digest={digest} generatedAt={generatedAt} crumb={skill.name}>
      <SkillHeroPage skill={skill} digest={digest} />
    </SkillShell>
  );
}

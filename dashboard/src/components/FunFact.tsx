'use client';

interface FunFactProps {
  text: string;
}

export function FunFact({ text }: FunFactProps) {
  return (
    <aside className="fun-fact" aria-label="Field note about this skill">
      <div className="ck-label fun-fact-label">Field note</div>
      <p className="fun-fact-text">{text}</p>
    </aside>
  );
}

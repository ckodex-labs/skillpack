'use client';

interface FailureStoryProps {
  text: string;
}

/* A failure story documents designed behavior — routine state, so it
   renders in ink, not red. Red requires an active emergency protocol. */
export function FailureStory({ text }: FailureStoryProps) {
  return (
    <aside className="failure-story" aria-label="What safe failure looks like">
      <div className="ck-label failure-story-label">When this skill fails</div>
      <p className="failure-story-text">{text}</p>
    </aside>
  );
}

import { useState } from 'react';
import { useRegisterVote } from '../../hooks/useNotes';

interface VoteButtonProps {
  subjectUri: string;
  initialCount: number;
}

export function VoteButton({ subjectUri, initialCount }: VoteButtonProps) {
  const [count, setCount] = useState(initialCount);
  const [voted, setVoted] = useState(false);
  const voteMutation = useRegisterVote();

  const handleVote = () => {
    if (voted) return;
    setVoted(true);
    setCount((c) => c + 1);
    voteMutation.mutate(
      { subjectUri },
      {
        onSuccess: (data) => setCount(data.totalVotes),
        onError: () => {
          setVoted(false);
          setCount((c) => c - 1);
        },
      }
    );
  };

  return (
    <button
      onClick={handleVote}
      disabled={voted}
      className={`inline-flex items-center gap-1.5 px-2.5 py-1 rounded-lg text-sm font-medium transition ${
        voted
          ? 'bg-academic/10 text-academic border border-academic/30'
          : 'bg-surface-alt text-text-secondary hover:bg-surface-hover border border-border'
      }`}
    >
      <svg className="w-4 h-4" fill={voted ? 'currentColor' : 'none'} viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
        <path strokeLinecap="round" strokeLinejoin="round" d="M5 15l7-7 7 7" />
      </svg>
      {count}
    </button>
  );
}

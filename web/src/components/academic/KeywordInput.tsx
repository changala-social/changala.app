import { useState } from 'react';
import { useAddKeyword } from '../../hooks/useKeywords';

interface KeywordInputProps {
  sessionUri: string;
}

export function KeywordInput({ sessionUri }: KeywordInputProps) {
  const [text, setText] = useState('');
  const [submitted, setSubmitted] = useState(false);
  const [error, setError] = useState('');
  const addKeyword = useAddKeyword();

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!text.trim() || submitted) return;
    setError('');
    addKeyword.mutate(
      { sessionUri, text: text.trim() },
      {
        onSuccess: () => setSubmitted(true),
        onError: (err) => {
          if (err.message.includes('already')) {
            setSubmitted(true);
          } else {
            setError(err.message);
          }
        },
      }
    );
  };

  if (submitted) {
    return (
      <div className="rounded-lg border border-live/30 bg-live/5 p-4 text-center">
        <p className="text-sm text-live font-medium">Keyword submitted!</p>
      </div>
    );
  }

  return (
    <form onSubmit={handleSubmit} className="flex gap-2">
      <input
        type="text"
        value={text}
        onChange={(e) => setText(e.target.value)}
        placeholder="Enter a keyword..."
        maxLength={128}
        className="flex-1 px-3 py-2 text-sm rounded-lg bg-surface-alt border border-border text-text placeholder:text-text-muted focus:outline-none focus:ring-2 focus:ring-live/50 focus:border-live transition"
      />
      <button
        type="submit"
        disabled={!text.trim() || addKeyword.isPending}
        className="px-4 py-2 text-sm font-medium rounded-lg bg-live text-white hover:bg-live/90 disabled:opacity-50 disabled:cursor-not-allowed transition"
      >
        {addKeyword.isPending ? '...' : 'Submit'}
      </button>
      {error && <p className="text-xs text-cancelled mt-1">{error}</p>}
    </form>
  );
}

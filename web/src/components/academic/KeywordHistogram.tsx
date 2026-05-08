import { useKeywordHistogram } from '../../hooks/useKeywords';
import { LoadingSpinner } from '../common/LoadingSpinner';
import { useEffect, useState } from 'react';

interface KeywordHistogramProps {
  sessionUri: string;
  live?: boolean;
}

export function KeywordHistogram({ sessionUri, live = false }: KeywordHistogramProps) {
  const { data, isLoading } = useKeywordHistogram(sessionUri, live);
  const [countdown, setCountdown] = useState('');

  useEffect(() => {
    if (!data?.windowExpiresAt || !data.windowOpen) return;
    const interval = setInterval(() => {
      const diff = new Date(data.windowExpiresAt!).getTime() - Date.now();
      if (diff <= 0) { setCountdown('Closed'); clearInterval(interval); return; }
      const mins = Math.floor(diff / 60000);
      const secs = Math.floor((diff % 60000) / 1000);
      setCountdown(`${mins}:${secs.toString().padStart(2, '0')}`);
    }, 1000);
    return () => clearInterval(interval);
  }, [data?.windowExpiresAt, data?.windowOpen]);

  if (isLoading) return <LoadingSpinner size="sm" />;
  if (!data) return null;

  const maxCount = Math.max(...data.entries.map((e) => e.count), 1);

  return (
    <div className="rounded-lg border border-border bg-surface p-4">
      <div className="flex items-center justify-between mb-3">
        <h3 className="text-sm font-semibold text-text">Keyword Histogram</h3>
        <div className="flex items-center gap-2 text-xs text-text-muted">
          <span>{data.totalSubmissions} submissions</span>
          {data.windowOpen && countdown && (
            <span className="px-2 py-0.5 rounded-full bg-live/10 text-live font-medium">{countdown} remaining</span>
          )}
        </div>
      </div>
      {data.entries.length === 0 ? (
        <p className="text-sm text-text-muted text-center py-4">No keywords yet</p>
      ) : (
        <div className="space-y-2">
          {data.entries
            .sort((a, b) => b.count - a.count)
            .map((entry) => (
              <div key={entry.text} className="flex items-center gap-3">
                <span className="text-sm text-text w-32 truncate text-right">{entry.text}</span>
                <div className="flex-1 bg-surface-alt rounded-full h-5 overflow-hidden">
                  <div
                    className="h-full bg-academic/20 rounded-full transition-all duration-500 flex items-center justify-end pr-2"
                    style={{ width: `${(entry.count / maxCount) * 100}%` }}
                  >
                    <span className="text-[10px] font-medium text-academic">{entry.count}</span>
                  </div>
                </div>
              </div>
            ))}
        </div>
      )}
    </div>
  );
}

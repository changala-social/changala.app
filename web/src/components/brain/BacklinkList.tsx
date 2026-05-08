import { Link } from 'react-router-dom';
import { useBacklinks } from '../../hooks/useBrain';
import { AuthorName } from '../common/AuthorName';
import { LoadingSpinner } from '../common/LoadingSpinner';

interface BacklinkListProps {
  nodeUri: string;
}

export function BacklinkList({ nodeUri }: BacklinkListProps) {
  const { data, isLoading } = useBacklinks(nodeUri);

  if (isLoading) return <LoadingSpinner size="sm" />;
  if (!data?.backlinks?.length) return null;

  return (
    <div className="rounded-lg border border-border bg-surface p-4">
      <h3 className="text-sm font-semibold text-text mb-3">
        Backlinks <span className="text-text-muted font-normal">({data.backlinks.length})</span>
      </h3>
      <div className="space-y-2">
        {data.backlinks.map((bl) => (
          <Link
            key={bl.fromUri}
            to={`/brain/${encodeURIComponent(bl.fromUri)}`}
            className="flex items-center justify-between p-2 rounded hover:bg-surface-hover transition"
          >
            <div className="flex-1 min-w-0">
              <p className="text-sm font-medium text-text truncate">{bl.fromTitle}</p>
              <div className="flex items-center gap-2 mt-0.5">
                <AuthorName did={bl.fromAuthorDid} />
                {bl.label && <span className="text-xs text-brain">{bl.label}</span>}
              </div>
            </div>
            <svg className="w-4 h-4 text-text-muted shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M9 5l7 7-7 7" />
            </svg>
          </Link>
        ))}
      </div>
    </div>
  );
}

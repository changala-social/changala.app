import { Link } from 'react-router-dom';
import type { BrainNode } from '../../generated/types';
import { AuthorName } from '../common/AuthorName';
import { VoteButton } from '../common/VoteButton';

interface BrainNodeCardProps {
  node: BrainNode;
}

export function BrainNodeCard({ node }: BrainNodeCardProps) {
  return (
    <div className="flex items-start gap-3 p-4 rounded-lg border border-border bg-surface hover:bg-surface-hover transition">
      <VoteButton subjectUri={node.uri} initialCount={node.voteCount} />
      <Link to={`/brain/${encodeURIComponent(node.uri)}`} className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <h3 className="font-semibold text-text truncate">{node.title}</h3>
          <span className="text-xs px-1.5 py-0.5 rounded bg-brain/10 text-brain uppercase shrink-0">{node.format}</span>
        </div>
        {node.summary && <p className="text-sm text-text-secondary mt-1 line-clamp-2">{node.summary}</p>}
        <div className="flex flex-wrap items-center gap-1.5 mt-2">
          {node.tags.map((tag) => (
            <span key={tag} className="text-xs px-2 py-0.5 rounded-full bg-brain/10 text-brain">
              {tag}
            </span>
          ))}
        </div>
        <div className="flex items-center gap-3 mt-2 text-xs text-text-muted">
          <AuthorName did={node.authorDid} />
          {node.academicRef && (
            <span className="px-1.5 py-0.5 rounded bg-academic/10 text-academic">Academic</span>
          )}
          <span>{new Date(node.createdAt).toLocaleDateString()}</span>
        </div>
      </Link>
    </div>
  );
}

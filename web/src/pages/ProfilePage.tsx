import { useParams } from 'react-router-dom';
import { useMemberships } from '../hooks/useAuth';
import { useBrainFeed } from '../hooks/useBrain';
import { BrainNodeCard } from '../components/brain/BrainNodeCard';
import { LoadingSpinner } from '../components/common/LoadingSpinner';
import { ErrorMessage } from '../components/common/ErrorMessage';
import { AuthorName } from '../components/common/AuthorName';
import type { BrainNode, Membership } from '../generated/types';
import { useState } from 'react';

const ROLE_BADGE_STYLES: Record<string, string> = {
  admin: 'bg-cancelled/10 text-cancelled border-cancelled/30',
  classRep: 'bg-academic/10 text-academic border-academic/30',
  faculty: 'bg-brain/10 text-brain border-brain/30',
  student: 'bg-surface-hover text-text-secondary border-border',
};

const ROLE_LABELS: Record<string, string> = {
  admin: 'Admin',
  classRep: 'Class Rep',
  faculty: 'Faculty',
  student: 'Student',
};

export default function ProfilePage() {
  const { did: rawDid } = useParams<{ did: string }>();
  const did = rawDid ? decodeURIComponent(rawDid) : '';

  const [brainCursor, setBrainCursor] = useState<string | undefined>(undefined);
  const [accNodes, setAccNodes] = useState<BrainNode[]>([]);

  const {
    data: membershipData,
    isLoading: membershipLoading,
    error: membershipError,
  } = useMemberships(did);

  const {
    data: brainData,
    isLoading: brainLoading,
    error: brainError,
    refetch: brainRefetch,
  } = useBrainFeed(brainCursor);

  const memberships = membershipData?.memberships ?? [];

  // Filter brain nodes by this author (client-side filter since the hook
  // may not natively support an author param)
  const rawNodes = brainData?.nodes ?? [];
  const authorNodes = rawNodes.filter((n) => n.authorDid === did);
  const brainNodes = brainCursor && accNodes.length > 0
    ? [...accNodes, ...authorNodes]
    : authorNodes;

  const handleLoadMore = () => {
    if (brainData?.cursor) {
      setAccNodes(brainNodes);
      setBrainCursor(brainData.cursor);
    }
  };

  if (!did) {
    return (
      <div className="max-w-3xl mx-auto px-4 py-8">
        <ErrorMessage title="Invalid profile" message="No DID provided in the URL." />
      </div>
    );
  }

  return (
    <div className="max-w-3xl mx-auto px-4 py-8">
      {/* Profile header */}
      <div className="rounded-lg border border-border bg-surface p-6 mb-8">
        <h1 className="text-xl font-bold text-text">
          <AuthorName did={did} />
        </h1>
        <p className="text-sm text-text-muted mt-1 font-mono break-all">{did}</p>

        {/* Memberships */}
        <div className="mt-5">
          <h2 className="text-sm font-semibold text-text-secondary uppercase tracking-wide mb-3">
            Institution Memberships
          </h2>
          {membershipLoading ? (
            <LoadingSpinner size="sm" />
          ) : membershipError ? (
            <ErrorMessage
              title="Failed to load memberships"
              message={membershipError instanceof Error ? membershipError.message : 'Unknown error'}
            />
          ) : memberships.length === 0 ? (
            <p className="text-sm text-text-muted">No verified memberships.</p>
          ) : (
            <div className="flex flex-wrap gap-3">
              {memberships.map((m: Membership) => (
                <div
                  key={`${m.institutionDid}-${m.role}`}
                  className="flex items-center gap-2 px-3 py-1.5 rounded-lg border border-border bg-surface-hover text-sm"
                >
                  <span className="text-text font-medium">{m.institutionDomain}</span>
                  <span
                    className={`text-xs px-2 py-0.5 rounded-full border font-medium ${
                      ROLE_BADGE_STYLES[m.role] ?? ROLE_BADGE_STYLES.student
                    }`}
                  >
                    {ROLE_LABELS[m.role] ?? m.role}
                  </span>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>

      {/* Brain nodes */}
      <div>
        <h2 className="text-lg font-semibold text-text mb-4">Brain Nodes</h2>

        {brainLoading && accNodes.length === 0 ? (
          <LoadingSpinner size="lg" />
        ) : brainError ? (
          <ErrorMessage
            title="Failed to load brain nodes"
            message={brainError instanceof Error ? brainError.message : 'Unknown error'}
            retry={() => brainRefetch()}
          />
        ) : brainNodes.length === 0 ? (
          <div className="text-center py-12">
            <p className="text-text-muted">No brain nodes yet.</p>
          </div>
        ) : (
          <>
            <div className="grid gap-4 sm:grid-cols-2">
              {brainNodes.map((node) => (
                <BrainNodeCard key={node.uri} node={node} />
              ))}
            </div>

            {brainData?.cursor && (
              <div className="flex justify-center mt-8">
                <button
                  onClick={handleLoadMore}
                  disabled={brainLoading}
                  className="px-5 py-2 rounded-lg border border-border bg-surface text-text text-sm font-medium hover:bg-surface-hover transition disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  {brainLoading ? 'Loading…' : 'Load more'}
                </button>
              </div>
            )}
          </>
        )}
      </div>
    </div>
  );
}

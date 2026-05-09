import { useParams, Link } from "react-router-dom";
import { useMemberships, useRole } from "../hooks/useAuth";
import { useBrainFeed } from "../hooks/useBrain";
import { useAuth } from "../context/AuthContext";
import { BrainNodeCard } from "../components/brain/BrainNodeCard";
import { LoadingSpinner } from "../components/common/LoadingSpinner";
import { ErrorMessage } from "../components/common/ErrorMessage";
import { AuthorName } from "../components/common/AuthorName";
import type { BrainNode, Membership } from "../generated/types";
import { useState } from "react";

const ROLE_BADGE_STYLES: Record<string, string> = {
  admin: "bg-cancelled/10 text-cancelled border-cancelled/30",
  classRep: "bg-academic/10 text-academic border-academic/30",
  faculty: "bg-brain/10 text-brain border-brain/30",
  student: "bg-surface-hover text-text-secondary border-border",
};

const ROLE_LABELS: Record<string, string> = {
  admin: "Administrator",
  classRep: "Class Representative",
  faculty: "Faculty",
  student: "Student",
};

export default function ProfilePage() {
  const { did: rawDid } = useParams<{ did: string }>();
  const did = rawDid ? decodeURIComponent(rawDid) : "";
  const { did: authedDid, handle: authedHandle } = useAuth();
  const isOwnProfile = !!(authedDid && did === authedDid);

  const [brainCursor, setBrainCursor] = useState<string | undefined>(undefined);
  const [accNodes, setAccNodes] = useState<BrainNode[]>([]);

  const {
    data: membershipData,
    isLoading: membershipLoading,
    error: membershipError,
  } = useMemberships(did);

  const { data: roleData } = useRole(did);

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
  const brainNodes =
    brainCursor && accNodes.length > 0
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
        <ErrorMessage
          title="Invalid profile"
          message="No DID provided in the URL."
        />
      </div>
    );
  }

  return (
    <div className="max-w-3xl mx-auto px-4 py-8">
      {/* Profile header */}
      <div className="rounded-lg border border-border bg-surface p-6 mb-8">
        <div className="flex items-start justify-between gap-4">
          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-3">
              <h1 className="text-xl font-bold text-text">
                {isOwnProfile && authedHandle ? (
                  `@${authedHandle}`
                ) : (
                  <AuthorName did={did} />
                )}
              </h1>
              {roleData?.role && (
                <span
                  className={`text-xs px-2.5 py-1 rounded-full border font-medium ${
                    ROLE_BADGE_STYLES[roleData.role] ??
                    ROLE_BADGE_STYLES.student
                  }`}
                >
                  {ROLE_LABELS[roleData.role] ?? roleData.role}
                </span>
              )}
            </div>
            <p className="text-xs text-text-muted mt-1 font-mono break-all">
              {did}
            </p>
            {isOwnProfile && (
              <p className="text-xs text-brain mt-1">This is your profile</p>
            )}
          </div>
          {isOwnProfile && (
            <div className="flex gap-2 shrink-0">
              <Link
                to="/brain/new"
                className="text-xs px-3 py-1.5 rounded-md bg-brain text-white hover:bg-brain/90 transition"
              >
                + Brain Node
              </Link>
              <Link
                to="/dashboard"
                className="text-xs px-3 py-1.5 rounded-md border border-border text-text-secondary hover:bg-surface-hover transition"
              >
                Dashboard
              </Link>
            </div>
          )}
        </div>

        {/* Memberships */}
        <div className="mt-6">
          <h2 className="text-sm font-semibold text-text-secondary uppercase tracking-wide mb-3">
            Institution Memberships
          </h2>
          {membershipLoading ? (
            <LoadingSpinner size="sm" />
          ) : membershipError ? (
            <ErrorMessage
              title="Failed to load memberships"
              message={
                membershipError instanceof Error
                  ? membershipError.message
                  : "Unknown error"
              }
            />
          ) : memberships.length === 0 ? (
            <div className="rounded-lg border border-border bg-surface-alt p-4 text-center">
              <p className="text-sm text-text-muted">
                No verified institution memberships.
              </p>
              <p className="text-xs text-text-muted mt-1">
                {isOwnProfile
                  ? "Verify your institution email from the Dashboard to join."
                  : "This user has not verified an institution email."}
              </p>
              {isOwnProfile && (
                <Link
                  to="/dashboard"
                  className="text-xs text-academic hover:underline mt-2 inline-block"
                >
                  Go to Dashboard →
                </Link>
              )}
            </div>
          ) : (
            <div className="space-y-3">
              {memberships.map((m: Membership) => (
                <div
                  key={`${m.institutionDid}-${m.role}`}
                  className="flex items-center justify-between p-3 rounded-lg border border-border bg-surface-alt"
                >
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2">
                      <span className="text-sm font-semibold text-text">
                        {m.institutionDomain}
                      </span>
                      <span
                        className={`text-xs px-2 py-0.5 rounded-full border font-medium ${
                          ROLE_BADGE_STYLES[m.role] ?? ROLE_BADGE_STYLES.student
                        }`}
                      >
                        {ROLE_LABELS[m.role] ?? m.role}
                      </span>
                    </div>
                    <div className="flex items-center gap-3 mt-1 text-xs text-text-muted">
                      <span>
                        Verified {new Date(m.verifiedAt).toLocaleDateString()}
                      </span>
                    </div>
                  </div>
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
            message={
              brainError instanceof Error ? brainError.message : "Unknown error"
            }
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
                  {brainLoading ? "Loading…" : "Load more"}
                </button>
              </div>
            )}
          </>
        )}
      </div>
    </div>
  );
}

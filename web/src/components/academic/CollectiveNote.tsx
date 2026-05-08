import {
  useCollectiveNote,
  useEditProposals,
  useAcceptEdit,
  useRejectEdit,
} from "../../hooks/useNotes";
import { useNoteContent } from "../../hooks/useNotes";
import { ContentRenderer } from "../content/ContentRenderer";
import { AuthorName } from "../common/AuthorName";
import { LoadingSpinner } from "../common/LoadingSpinner";

interface CollectiveNoteProps {
  sessionUri: string;
  isClassRep?: boolean;
}

export function CollectiveNote({
  sessionUri,
  isClassRep = false,
}: CollectiveNoteProps) {
  const { data: collective, isLoading } = useCollectiveNote(sessionUri);
  const { data: content } = useNoteContent(
    collective?.ringRef?.cid || "",
    collective?.ringRef?.ringDid || "",
  );
  const { data: proposals } = useEditProposals(sessionUri);
  const acceptEdit = useAcceptEdit();
  const rejectEdit = useRejectEdit();

  if (isLoading) return <LoadingSpinner size="sm" />;
  if (!collective)
    return <p className="text-sm text-text-muted">No collective note yet</p>;

  return (
    <div className="space-y-4">
      <div className="rounded-lg border border-border bg-surface p-4">
        <div className="flex items-center justify-between mb-3">
          <h3 className="text-sm font-semibold text-text">Collective Note</h3>
          <span className="text-xs text-text-muted">v{collective.version}</span>
        </div>
        {content && (
          <ContentRenderer format={content.format} content={content.content} />
        )}
        <div className="mt-3 flex flex-wrap gap-1">
          {collective.contributorDids.map((did) => (
            <AuthorName key={did} did={did} />
          ))}
        </div>
      </div>

      {proposals?.proposals && proposals.proposals.length > 0 && (
        <div className="rounded-lg border border-border bg-surface p-4">
          <h4 className="text-sm font-semibold text-text mb-3">
            Edit Proposals
          </h4>
          <div className="space-y-3">
            {proposals.proposals.map((p) => (
              <div
                key={p.proposalUri}
                className="p-3 rounded border border-border bg-surface-alt"
              >
                <div className="flex items-center justify-between">
                  <AuthorName did={p.proposerDid} />
                  <span
                    className={`text-xs px-2 py-0.5 rounded-full ${
                      p.status === "pending"
                        ? "bg-rescheduled/10 text-rescheduled"
                        : p.status === "accepted"
                          ? "bg-live/10 text-live"
                          : "bg-cancelled/10 text-cancelled"
                    }`}
                  >
                    {p.status}
                  </span>
                </div>
                {p.summary && (
                  <p className="text-sm text-text-secondary mt-1">
                    {p.summary}
                  </p>
                )}
                {isClassRep && p.status === "pending" && (
                  <div className="flex gap-2 mt-2">
                    <button
                      onClick={() =>
                        acceptEdit.mutate({ proposalUri: p.proposalUri })
                      }
                      className="text-xs px-3 py-1 rounded bg-live text-white hover:bg-live/90"
                    >
                      Accept
                    </button>
                    <button
                      onClick={() =>
                        rejectEdit.mutate({ proposalUri: p.proposalUri })
                      }
                      className="text-xs px-3 py-1 rounded bg-cancelled text-white hover:bg-cancelled/90"
                    >
                      Reject
                    </button>
                  </div>
                )}
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}

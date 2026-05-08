import { useParams, Link } from 'react-router-dom';
import { useSession } from '../hooks/useSessions';
import { useNotes } from '../hooks/useNotes';
import { KeywordHistogram } from '../components/academic/KeywordHistogram';
import { KeywordInput } from '../components/academic/KeywordInput';
import { NoteCard } from '../components/academic/NoteCard';
import { StatusBadge } from '../components/common/StatusBadge';
import { LoadingSpinner } from '../components/common/LoadingSpinner';
import { ErrorMessage } from '../components/common/ErrorMessage';

export default function LiveSession() {
  const { uri: rawUri } = useParams<{ uri: string }>();
  const uri = rawUri ? decodeURIComponent(rawUri) : '';

  const {
    data: session,
    isLoading: sessionLoading,
    isError: sessionError,
    error: sessionErr,
    refetch: retrySession,
  } = useSession(uri);

  const {
    data: notesData,
    isLoading: notesLoading,
  } = useNotes(uri);

  if (!uri) {
    return (
      <div className="max-w-4xl mx-auto px-4 py-8">
        <ErrorMessage title="Missing session" message="No session URI was provided in the URL." />
      </div>
    );
  }

  if (sessionLoading) {
    return (
      <div className="max-w-4xl mx-auto px-4 py-8">
        <LoadingSpinner />
      </div>
    );
  }

  if (sessionError || !session) {
    return (
      <div className="max-w-4xl mx-auto px-4 py-8">
        <ErrorMessage
          title="Failed to load session"
          message={sessionErr instanceof Error ? sessionErr.message : 'The session could not be retrieved.'}
          retry={retrySession}
        />
      </div>
    );
  }

  const isLive = session.status === 'live';

  return (
    <div className="max-w-4xl mx-auto px-4 py-8 space-y-8">
      {/* Header */}
      <div>
        <Link
          to={`/session/${encodeURIComponent(uri)}`}
          className="inline-flex items-center gap-1 text-sm text-academic hover:underline mb-4"
        >
          <span aria-hidden="true">&larr;</span>
          Session details
        </Link>

        <div className="flex items-start justify-between gap-4">
          <div className="min-w-0">
            <h1 className="text-2xl font-bold text-text truncate">
              {session.topic || 'Untitled Session'}
            </h1>
            <p className="text-sm text-text-muted mt-1">
              {new Date(session.scheduledAt).toLocaleString()}
              {session.durationMins > 0 && ` · ${session.durationMins} min`}
            </p>
          </div>

          <div className="flex items-center gap-3 shrink-0">
            {isLive && (
              <span className="inline-flex items-center gap-1.5 text-sm font-semibold text-live">
                <span className="relative flex h-2.5 w-2.5">
                  <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-live opacity-75" />
                  <span className="relative inline-flex rounded-full h-2.5 w-2.5 bg-live" />
                </span>
                Live
              </span>
            )}
            <StatusBadge status={session.status} />
          </div>
        </div>
      </div>

      {/* Keyword Histogram — prominent */}
      <section className="rounded-lg border border-border bg-surface p-6">
        <h2 className="text-lg font-semibold text-text mb-4">Keyword Histogram</h2>
        <KeywordHistogram sessionUri={uri} live={true} />
      </section>

      {/* Keyword Input — shown when window is open */}
      {session.keywordWindowOpen && (
        <section className="rounded-lg border border-academic/30 bg-academic/5 p-6">
          <h2 className="text-lg font-semibold text-academic mb-2">Add Your Keyword</h2>
          <p className="text-sm text-text-muted mb-4">
            The keyword window is open.
            {session.keywordWindowExpiresAt && (
              <> Closes at {new Date(session.keywordWindowExpiresAt).toLocaleTimeString()}.</>
            )}
          </p>
          <KeywordInput sessionUri={uri} />
        </section>
      )}

      {/* Session Notes */}
      <section>
        <h2 className="text-lg font-semibold text-text mb-4">Session Notes</h2>

        {notesLoading && <LoadingSpinner size="sm" />}

        {!notesLoading && notesData?.notes && notesData.notes.length > 0 ? (
          <div className="space-y-3">
            {notesData.notes.map((note) => (
              <NoteCard key={note.uri} note={note} />
            ))}
          </div>
        ) : (
          !notesLoading && (
            <p className="text-sm text-text-muted py-4 text-center">
              No notes have been submitted for this session yet.
            </p>
          )
        )}
      </section>
    </div>
  );
}

import { useState } from 'react';
import { useParams, Link } from 'react-router-dom';
import { useSession } from '../hooks/useSessions';
import { useNotes } from '../hooks/useNotes';
import { StatusBadge } from '../components/common/StatusBadge';
import { NoteCard } from '../components/academic/NoteCard';
import { KeywordHistogram } from '../components/academic/KeywordHistogram';
import { LoadingSpinner } from '../components/common/LoadingSpinner';
import { ErrorMessage } from '../components/common/ErrorMessage';
import type { Note } from '../generated/types';

export default function SessionDetail() {
  const { uri: rawUri } = useParams<{ uri: string }>();
  const uri = rawUri ? decodeURIComponent(rawUri) : '';

  const { data: session, isLoading: sessionLoading, error: sessionError } = useSession(uri);

  const [notesCursor, setNotesCursor] = useState<string | undefined>(undefined);
  const [accumulatedNotes, setAccumulatedNotes] = useState<Note[]>([]);
  const { data: notesData, isLoading: notesLoading } = useNotes(uri, notesCursor);

  const allNotes = notesCursor && accumulatedNotes.length > 0
    ? [...accumulatedNotes, ...(notesData?.notes ?? [])]
    : notesData?.notes ?? [];

  const handleLoadMoreNotes = () => {
    if (notesData?.cursor) {
      setAccumulatedNotes(allNotes);
      setNotesCursor(notesData.cursor);
    }
  };

  // Loading
  if (sessionLoading) {
    return (
      <div className="max-w-3xl mx-auto px-4 py-16">
        <LoadingSpinner size="lg" />
      </div>
    );
  }

  // Error
  if (sessionError || !session) {
    return (
      <div className="max-w-3xl mx-auto px-4 py-16">
        <ErrorMessage
          title="Session not found"
          message={sessionError instanceof Error ? sessionError.message : 'Could not load this session.'}
        />
      </div>
    );
  }

  const scheduledDate = new Date(session.scheduledAt);
  const isLive = session.status === 'live';

  return (
    <div className="max-w-3xl mx-auto px-4 py-8">
      {/* Session Header */}
      <header className="mb-8">
        <div className="flex items-start justify-between gap-4">
          <div className="min-w-0">
            <h1 className="text-2xl font-bold text-text">
              {session.topic || 'Untitled Session'}
            </h1>
            <p className="mt-1 text-text-secondary">
              {scheduledDate.toLocaleDateString('en', {
                weekday: 'long',
                year: 'numeric',
                month: 'long',
                day: 'numeric',
              })}
              {' · '}
              {scheduledDate.toLocaleTimeString('en', {
                hour: '2-digit',
                minute: '2-digit',
              })}
            </p>
          </div>
          <StatusBadge status={session.status} />
        </div>

        <div className="flex flex-wrap items-center gap-4 mt-4 text-sm text-text-muted">
          <span>{session.durationMins} min</span>

          {session.keywordWindowOpen !== undefined && (
            <span className={`px-2 py-0.5 rounded-full text-xs font-medium ${
              session.keywordWindowOpen
                ? 'bg-live/10 text-live'
                : 'bg-surface-alt text-text-muted'
            }`}>
              Keyword window {session.keywordWindowOpen ? 'open' : 'closed'}
            </span>
          )}

          {session.rescheduledTo && (
            <span className="text-rescheduled">
              Rescheduled to {new Date(session.rescheduledTo).toLocaleString()}
            </span>
          )}
        </div>

        {/* Live link */}
        {isLive && (
          <Link
            to={`/session/${encodeURIComponent(session.uri)}/live`}
            className="inline-flex items-center gap-2 mt-5 px-4 py-2 rounded-lg bg-live text-white text-sm font-medium hover:opacity-90 transition"
          >
            <span className="relative flex h-2 w-2">
              <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-white opacity-75" />
              <span className="relative inline-flex rounded-full h-2 w-2 bg-white" />
            </span>
            Join Live Session
          </Link>
        )}
      </header>

      {/* Keyword Histogram */}
      <section className="mb-8">
        <KeywordHistogram sessionUri={uri} live={isLive} />
      </section>

      {/* Notes */}
      <section>
        <h2 className="text-lg font-semibold text-text mb-4">Notes</h2>

        {notesLoading && accumulatedNotes.length === 0 ? (
          <LoadingSpinner size="md" />
        ) : allNotes.length === 0 ? (
          <p className="text-text-muted text-sm py-6 text-center">No notes submitted yet.</p>
        ) : (
          <div className="space-y-3">
            {allNotes.map((note) => (
              <NoteCard key={note.uri} note={note} />
            ))}
          </div>
        )}

        {notesData?.cursor && (
          <div className="flex justify-center mt-6">
            <button
              onClick={handleLoadMoreNotes}
              disabled={notesLoading}
              className="px-5 py-2 rounded-lg border border-border bg-surface text-text text-sm font-medium hover:bg-surface-hover transition disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {notesLoading ? 'Loading…' : 'Load more notes'}
            </button>
          </div>
        )}
      </section>
    </div>
  );
}

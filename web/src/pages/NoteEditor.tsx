import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useCreateNote } from '../hooks/useNotes';
import { ContentRenderer } from '../components/content/ContentRenderer';
import { LoadingSpinner } from '../components/common/LoadingSpinner';
import { ErrorMessage } from '../components/common/ErrorMessage';
import type { NoteFormat } from '../generated/types';

export default function NoteEditor() {
  const navigate = useNavigate();
  const createNote = useCreateNote();

  const [format, setFormat] = useState<NoteFormat>('latex');
  const [sessionUri, setSessionUri] = useState('');
  const [content, setContent] = useState('');
  const [summary, setSummary] = useState('');

  const canSubmit = sessionUri.trim().length > 0 && content.trim().length > 0 && !createNote.isPending;

  function handleSubmit() {
    if (!canSubmit) return;

    createNote.mutate(
      {
        sessionUri: sessionUri.trim(),
        content,
        format,
        summary: summary.trim() || undefined,
      },
      {
        onSuccess: (data) => {
          navigate(`/note/${encodeURIComponent(data.ringRef.cid)}`);
        },
      },
    );
  }

  return (
    <div className="max-w-7xl mx-auto px-4 py-8 space-y-6">
      <h1 className="text-2xl font-bold text-text">New Note</h1>

      {/* Meta controls */}
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
        <div>
          <label htmlFor="session-uri" className="block text-sm font-medium text-text mb-1">
            Session URI
          </label>
          <input
            id="session-uri"
            type="text"
            value={sessionUri}
            onChange={(e) => setSessionUri(e.target.value)}
            placeholder="at://did:plc:.../app.changala.session/..."
            className="w-full rounded-md border border-border bg-surface px-3 py-2 text-sm text-text placeholder:text-text-muted focus:outline-none focus:ring-2 focus:ring-academic/50"
          />
        </div>

        <div>
          <label htmlFor="format" className="block text-sm font-medium text-text mb-1">
            Format
          </label>
          <select
            id="format"
            value={format}
            onChange={(e) => setFormat(e.target.value as NoteFormat)}
            className="w-full rounded-md border border-border bg-surface px-3 py-2 text-sm text-text focus:outline-none focus:ring-2 focus:ring-academic/50"
          >
            <option value="latex">LaTeX</option>
            <option value="plaintext">Plaintext</option>
          </select>
        </div>
      </div>

      <div>
        <label htmlFor="summary" className="block text-sm font-medium text-text mb-1">
          Summary <span className="text-text-muted">(optional)</span>
        </label>
        <input
          id="summary"
          type="text"
          value={summary}
          onChange={(e) => setSummary(e.target.value)}
          placeholder="Brief description of this note"
          className="w-full rounded-md border border-border bg-surface px-3 py-2 text-sm text-text placeholder:text-text-muted focus:outline-none focus:ring-2 focus:ring-academic/50"
        />
      </div>

      {/* Editor + Preview — side-by-side on desktop, stacked on mobile */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
        <div className="flex flex-col">
          <label htmlFor="content" className="block text-sm font-medium text-text mb-1">
            Content
          </label>
          <textarea
            id="content"
            value={content}
            onChange={(e) => setContent(e.target.value)}
            placeholder={format === 'latex' ? '\\section{Introduction}\n...' : 'Start writing...'}
            className="flex-1 min-h-[400px] rounded-md border border-border bg-surface px-4 py-3 font-mono text-sm text-text placeholder:text-text-muted resize-y focus:outline-none focus:ring-2 focus:ring-academic/50"
          />
        </div>

        <div className="flex flex-col">
          <span className="block text-sm font-medium text-text mb-1">
            Preview
          </span>
          <div className="flex-1 min-h-[400px] rounded-md border border-border bg-surface p-4 overflow-auto">
            {content.trim() ? (
              <ContentRenderer format={format} content={content} />
            ) : (
              <p className="text-sm text-text-muted italic">Start typing to see a preview&hellip;</p>
            )}
          </div>
        </div>
      </div>

      {/* Error display */}
      {createNote.isError && (
        <ErrorMessage
          title="Failed to create note"
          message={createNote.error instanceof Error ? createNote.error.message : 'An unexpected error occurred.'}
        />
      )}

      {/* Submit */}
      <div className="flex items-center gap-4">
        <button
          type="button"
          onClick={handleSubmit}
          disabled={!canSubmit}
          className="inline-flex items-center gap-2 rounded-md bg-academic px-5 py-2.5 text-sm font-medium text-white hover:bg-academic/90 disabled:opacity-50 disabled:cursor-not-allowed transition"
        >
          {createNote.isPending && <LoadingSpinner size="sm" />}
          {createNote.isPending ? 'Submitting…' : 'Submit Note'}
        </button>

        {!sessionUri.trim() && content.trim() && (
          <p className="text-xs text-text-muted">Enter a session URI to enable submission.</p>
        )}
      </div>
    </div>
  );
}

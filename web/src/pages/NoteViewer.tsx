import { useParams, Link } from 'react-router-dom';
import { useNoteContent } from '../hooks/useNotes';
import { ContentRenderer } from '../components/content/ContentRenderer';
import { LoadingSpinner } from '../components/common/LoadingSpinner';
import { ErrorMessage } from '../components/common/ErrorMessage';

export default function NoteViewer() {
  const { cid: rawCid } = useParams<{ cid: string }>();
  const cid = rawCid ? decodeURIComponent(rawCid) : '';

  const { data, isLoading, isError, error, refetch } = useNoteContent(cid);

  if (!cid) {
    return (
      <div className="max-w-3xl mx-auto px-4 py-8">
        <ErrorMessage title="Missing note ID" message="No content identifier was provided in the URL." />
      </div>
    );
  }

  if (isLoading) {
    return (
      <div className="max-w-3xl mx-auto px-4 py-8">
        <LoadingSpinner />
      </div>
    );
  }

  if (isError || !data) {
    return (
      <div className="max-w-3xl mx-auto px-4 py-8">
        <ErrorMessage
          title="Failed to load note"
          message={error instanceof Error ? error.message : 'The note could not be retrieved.'}
          retry={refetch}
        />
      </div>
    );
  }

  return (
    <div className="max-w-3xl mx-auto px-4 py-8">
      <Link
        to=".."
        className="inline-flex items-center gap-1 text-sm text-academic hover:underline mb-6"
      >
        <span aria-hidden="true">&larr;</span>
        Back
      </Link>

      <div className="rounded-lg border border-border bg-surface p-6">
        <span className="inline-block text-[10px] uppercase tracking-wider px-2 py-0.5 rounded bg-surface-alt text-text-muted mb-4">
          {data.format}
        </span>

        <ContentRenderer format={data.format} content={data.content} />
      </div>
    </div>
  );
}

import { Link } from 'react-router-dom';
import type { Note } from '../../generated/types';
import { AuthorName } from '../common/AuthorName';
import { VoteButton } from '../common/VoteButton';

interface NoteCardProps {
  note: Note;
}

export function NoteCard({ note }: NoteCardProps) {
  return (
    <div className="flex items-start gap-3 p-3 rounded-lg border border-border bg-surface hover:bg-surface-hover transition">
      <VoteButton subjectUri={note.uri} initialCount={note.voteCount} />
      <Link to={`/note/${encodeURIComponent(note.ringRef.cid)}`} className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <AuthorName did={note.authorDid} />
          <span className="text-xs px-1.5 py-0.5 rounded bg-surface-alt text-text-muted uppercase">{note.format}</span>
          {note.version > 1 && <span className="text-xs text-text-muted">v{note.version}</span>}
        </div>
        {note.summary && <p className="text-sm text-text mt-1 line-clamp-2">{note.summary}</p>}
        <div className="flex items-center gap-2 mt-1.5">
          {note.labels.map((l, i) => (
            <span key={i} className="text-[10px] px-1.5 py-0.5 rounded-full bg-live/10 text-live">{l.val}</span>
          ))}
          <span className="text-xs text-text-muted">{new Date(note.createdAt).toLocaleDateString()}</span>
        </div>
      </Link>
    </div>
  );
}

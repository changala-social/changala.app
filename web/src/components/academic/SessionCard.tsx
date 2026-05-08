import { Link } from 'react-router-dom';
import type { Session } from '../../generated/types';
import { StatusBadge } from '../common/StatusBadge';

interface SessionCardProps {
  session: Session;
}

export function SessionCard({ session }: SessionCardProps) {
  const to = session.status === 'live' ? `/session/${encodeURIComponent(session.uri)}/live` : `/session/${encodeURIComponent(session.uri)}`;
  const date = new Date(session.scheduledAt);

  return (
    <Link
      to={to}
      className="flex items-center gap-4 p-3 rounded-lg border border-border bg-surface hover:bg-surface-hover transition"
    >
      <div className="text-center shrink-0 w-14">
        <div className="text-xs text-text-muted uppercase">{date.toLocaleDateString('en', { month: 'short' })}</div>
        <div className="text-lg font-bold text-text">{date.getDate()}</div>
        <div className="text-xs text-text-muted">{date.toLocaleTimeString('en', { hour: '2-digit', minute: '2-digit' })}</div>
      </div>
      <div className="flex-1 min-w-0">
        <p className="font-medium text-text truncate">{session.topic || 'Untitled Session'}</p>
        <p className="text-xs text-text-muted mt-0.5">{session.durationMins} min</p>
      </div>
      <StatusBadge status={session.status} />
    </Link>
  );
}

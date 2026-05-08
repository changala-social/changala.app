import type { SessionStatus } from '../../generated/types';

const statusConfig: Record<SessionStatus, { bg: string; text: string; label: string; pulse?: boolean }> = {
  scheduled: { bg: 'bg-scheduled/10', text: 'text-scheduled', label: 'Scheduled' },
  live: { bg: 'bg-live/10', text: 'text-live', label: 'Live', pulse: true },
  ended: { bg: 'bg-ended/10', text: 'text-ended', label: 'Ended' },
  cancelled: { bg: 'bg-cancelled/10', text: 'text-cancelled', label: 'Cancelled' },
  rescheduled: { bg: 'bg-rescheduled/10', text: 'text-rescheduled', label: 'Rescheduled' },
};

export function StatusBadge({ status }: { status: SessionStatus }) {
  const config = statusConfig[status] || statusConfig.scheduled;

  return (
    <span className={`inline-flex items-center gap-1.5 px-2 py-0.5 rounded-full text-xs font-medium ${config.bg} ${config.text}`}>
      {config.pulse && (
        <span className="relative flex h-2 w-2">
          <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-live opacity-75" />
          <span className="relative inline-flex rounded-full h-2 w-2 bg-live" />
        </span>
      )}
      {config.label}
    </span>
  );
}

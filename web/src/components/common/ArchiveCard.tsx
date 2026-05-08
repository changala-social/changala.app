interface ArchiveCardProps {
  archiveUri: string;
  courseTitle: string;
  semester: string;
  sessionCount: number;
  sealedAt: string;
  iaUrl?: string;
}

export function ArchiveCard({
  courseTitle,
  semester,
  sessionCount,
  sealedAt,
  iaUrl,
}: ArchiveCardProps) {
  return (
    <div className="p-4 rounded-lg border border-border bg-surface hover:bg-surface-hover transition">
      <h3 className="font-semibold text-text">{courseTitle}</h3>
      <p className="text-sm text-text-secondary mt-0.5">{semester}</p>
      <div className="flex items-center gap-3 mt-2 text-xs text-text-muted">
        <span>{sessionCount} sessions</span>
        <span>Sealed {new Date(sealedAt).toLocaleDateString()}</span>
      </div>
      {iaUrl && (
        <a
          href={iaUrl}
          target="_blank"
          rel="noopener noreferrer"
          className="inline-block mt-2 text-xs text-academic hover:underline"
        >
          View on Internet Archive &rarr;
        </a>
      )}
    </div>
  );
}

interface ErrorMessageProps {
  title?: string;
  message?: string;
  retry?: () => void;
}

export function ErrorMessage({ title = 'Something went wrong', message, retry }: ErrorMessageProps) {
  return (
    <div className="rounded-lg border border-cancelled/20 bg-cancelled/5 p-6 text-center">
      <p className="font-medium text-text">{title}</p>
      {message && <p className="mt-1 text-sm text-text-secondary">{message}</p>}
      {retry && (
        <button onClick={retry} className="mt-3 text-sm text-academic hover:underline">
          Try again
        </button>
      )}
    </div>
  );
}

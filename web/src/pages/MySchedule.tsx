import { useAuth } from '../context/AuthContext';
import { useMyEnrollments } from '../hooks/useCourses';
import { LoadingSpinner } from '../components/common/LoadingSpinner';
import { ErrorMessage } from '../components/common/ErrorMessage';
import { Link } from 'react-router-dom';

export default function MySchedule() {
  const { isAuthenticated, did } = useAuth();
  const { data, isLoading, error } = useMyEnrollments(did || '');

  if (!isAuthenticated) {
    return (
      <div className="max-w-3xl mx-auto px-4 py-16 text-center">
        <h1 className="text-2xl font-bold text-text mb-3">My Schedule</h1>
        <p className="text-text-secondary">Sign in to view your schedule.</p>
      </div>
    );
  }

  if (isLoading) return <div className="max-w-4xl mx-auto px-4 py-16"><LoadingSpinner size="lg" /></div>;
  if (error) return (
    <div className="max-w-4xl mx-auto px-4 py-16">
      <ErrorMessage title="Failed to load enrollments" message={error instanceof Error ? error.message : 'Unknown error'} />
    </div>
  );

  const enrollments = data?.enrollments ?? [];

  return (
    <div className="max-w-5xl mx-auto px-4 py-8">
      <h1 className="text-2xl font-bold text-text mb-2">My Schedule</h1>
      <p className="text-sm text-text-muted mb-8">
        {enrollments.length} enrolled course{enrollments.length !== 1 ? 's' : ''} · DID: <code className="text-xs bg-surface-alt px-1 py-0.5 rounded">{did}</code>
      </p>

      {enrollments.length === 0 ? (
        <div className="text-center py-16 border border-border rounded-lg bg-surface">
          <p className="text-text-muted">No enrollments yet.</p>
          <Link to="/courses" className="mt-2 inline-block text-sm text-academic hover:underline">
            Browse courses →
          </Link>
        </div>
      ) : (
        <div className="space-y-6">
          {/* Slot summary */}
          <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
            {enrollments.map((e) => (
              <div key={e.courseUri} className="p-4 rounded-lg border border-border bg-surface">
                <div className="flex items-start justify-between gap-2">
                  <div className="min-w-0">
                    <p className="text-sm font-semibold text-text truncate">{e.courseCode}</p>
                    <p className="text-xs text-text-muted truncate">{e.courseTitle}</p>
                  </div>
                  {e.slot && (
                    <span className="shrink-0 text-xs font-mono px-2 py-0.5 rounded bg-academic/10 text-academic">
                      {e.slot}
                    </span>
                  )}
                </div>
                <div className="mt-2 text-xs text-text-muted space-y-0.5">
                  <p>Dept: {e.department} · Sem: {e.semester}</p>
                  <p>Enrolled: {new Date(e.enrolledAt).toLocaleDateString()}</p>
                </div>
                <Link
                  to={`/course/${encodeURIComponent(e.courseUri)}`}
                  className="mt-2 inline-block text-xs text-academic hover:underline"
                >
                  View course →
                </Link>
              </div>
            ))}
          </div>

          {/* Raw data table for dev */}
          <details className="border border-border rounded-lg bg-surface">
            <summary className="px-4 py-3 text-sm font-medium text-text cursor-pointer hover:bg-surface-hover">
              Raw Enrollment Data (Dev)
            </summary>
            <div className="overflow-x-auto">
              <table className="w-full text-xs">
                <thead>
                  <tr className="border-b border-border bg-surface-alt">
                    <th className="px-3 py-2 text-left text-text-muted font-medium">Code</th>
                    <th className="px-3 py-2 text-left text-text-muted font-medium">Title</th>
                    <th className="px-3 py-2 text-left text-text-muted font-medium">Slot</th>
                    <th className="px-3 py-2 text-left text-text-muted font-medium">Course URI</th>
                    <th className="px-3 py-2 text-left text-text-muted font-medium">Enrolled At</th>
                  </tr>
                </thead>
                <tbody>
                  {enrollments.map((e) => (
                    <tr key={e.courseUri} className="border-b border-border hover:bg-surface-hover">
                      <td className="px-3 py-2 font-mono text-text">{e.courseCode}</td>
                      <td className="px-3 py-2 text-text">{e.courseTitle}</td>
                      <td className="px-3 py-2 font-mono text-academic">{e.slot || '—'}</td>
                      <td className="px-3 py-2 font-mono text-text-muted max-w-48 truncate">{e.courseUri}</td>
                      <td className="px-3 py-2 text-text-muted">{e.enrolledAt}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </details>
        </div>
      )}
    </div>
  );
}

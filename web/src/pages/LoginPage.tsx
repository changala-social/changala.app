import { useState, useEffect } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';
import { useAuth } from '../context/AuthContext';
import { LoadingSpinner } from '../components/common/LoadingSpinner';
import { ErrorMessage } from '../components/common/ErrorMessage';

export default function LoginPage() {
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();
  const { isAuthenticated, login } = useAuth();

  const [handle, setHandle] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [processing, setProcessing] = useState(false);

  // Already authenticated — redirect immediately
  useEffect(() => {
    if (isAuthenticated) {
      navigate('/dashboard', { replace: true });
    }
  }, [isAuthenticated, navigate]);

  // Handle OAuth callback: token, did, handle arrive as query params
  useEffect(() => {
    const token = searchParams.get('token');
    const did = searchParams.get('did');
    const callbackHandle = searchParams.get('handle');

    if (token && did && callbackHandle) {
      setProcessing(true);
      login(token, did, callbackHandle);
      navigate('/dashboard', { replace: true });
    }
  }, [searchParams, login, navigate]);

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError(null);

    const trimmed = handle.trim();
    if (!trimmed) {
      setError('Please enter your AT Protocol handle.');
      return;
    }

    // Redirect to the atrg OAuth endpoint
    window.location.href = `/auth/login?handle=${encodeURIComponent(trimmed)}`;
  }

  if (processing) {
    return (
      <div className="flex min-h-[60vh] items-center justify-center">
        <div className="text-center space-y-4">
          <LoadingSpinner size="lg" />
          <p className="text-sm text-text-muted">Completing login&hellip;</p>
        </div>
      </div>
    );
  }

  return (
    <div className="flex min-h-[60vh] items-center justify-center px-4">
      <div className="w-full max-w-sm space-y-8">
        {/* Heading */}
        <div className="text-center">
          <h1 className="text-3xl font-bold text-text">Changala</h1>
          <p className="mt-2 text-sm text-text-muted">
            Sign in with your AT Protocol identity
          </p>
        </div>

        {/* Login form */}
        <form onSubmit={handleSubmit} className="space-y-5">
          <div>
            <label htmlFor="handle" className="block text-sm font-medium text-text mb-1">
              AT Protocol Handle
            </label>
            <input
              id="handle"
              type="text"
              value={handle}
              onChange={(e) => setHandle(e.target.value)}
              placeholder="user.bsky.social"
              autoComplete="username"
              autoFocus
              className="w-full rounded-md border border-border bg-surface px-3 py-2 text-sm text-text placeholder:text-text-muted focus:outline-none focus:ring-2 focus:ring-academic/50"
            />
          </div>

          {error && <ErrorMessage title="Login error" message={error} />}

          <button
            type="submit"
            className="w-full rounded-md bg-academic px-4 py-2.5 text-sm font-medium text-white hover:bg-academic/90 transition"
          >
            Login with AT Protocol
          </button>
        </form>

        <p className="text-center text-xs text-text-muted">
          Your identity lives on the AT Protocol — Changala never owns your account.
        </p>
      </div>
    </div>
  );
}

import { useState } from "react";
import { Navigate, useSearchParams } from "react-router-dom";
import { useAuth } from "../context/AuthContext";
import { ErrorMessage } from "../components/common/ErrorMessage";

const RING_URL =
  import.meta.env.VITE_RING_URL || "https://changala.tail477f2f.ts.net";

/**
 * Handles the OAuth callback redirect from /auth/complete.
 * If token params are present, logs in and redirects to /dashboard.
 * Otherwise renders the login form.
 */
export default function LoginPage() {
  const [searchParams] = useSearchParams();
  const { isAuthenticated, login } = useAuth();

  // If /auth/complete redirected here with token params, log in immediately
  const token = searchParams.get("token");
  const did = searchParams.get("did");
  const handleParam = searchParams.get("handle");

  if (token && did && handleParam) {
    // Call login synchronously during render — this is safe because
    // it only writes to localStorage + sets context state, then we
    // redirect via Navigate (no setState needed).
    login(token, did, handleParam);
    return <Navigate to="/dashboard" replace />;
  }

  // Already authenticated
  if (isAuthenticated) {
    return <Navigate to="/dashboard" replace />;
  }

  // Derive error from URL params (set by /auth/complete on failure)
  const errorParam = searchParams.get("error");
  const authError = errorParam
    ? errorParam === "no_session"
      ? "Login failed: no session was created. Please try again."
      : errorParam === "invalid_session"
        ? "Login failed: session expired. Please try again."
        : `Login failed: ${errorParam}`
    : null;

  return <LoginForm authError={authError} />;
}

function LoginForm({ authError }: { authError: string | null }) {
  const [handle, setHandle] = useState("");
  const [submitError, setSubmitError] = useState<string | null>(null);

  const error = submitError || authError;

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setSubmitError(null);

    const trimmed = handle.trim();
    if (!trimmed) {
      setSubmitError("Please enter your AT Protocol handle.");
      return;
    }

    const redirectAfter = `${RING_URL}/auth/complete`;
    window.location.href = `${RING_URL}/auth/login?handle=${encodeURIComponent(trimmed)}&redirect_after=${encodeURIComponent(redirectAfter)}`;
  }

  return (
    <div className="flex min-h-[60vh] items-center justify-center px-4">
      <div className="w-full max-w-sm space-y-8">
        <div className="text-center">
          <h1 className="text-3xl font-bold text-text">Changala</h1>
          <p className="mt-2 text-sm text-text-muted">
            Sign in with your AT Protocol identity
          </p>
        </div>

        <form onSubmit={handleSubmit} className="space-y-5">
          <div>
            <label
              htmlFor="handle"
              className="block text-sm font-medium text-text mb-1"
            >
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
          Your identity lives on the AT Protocol &mdash; Changala never owns
          your account.
        </p>
      </div>
    </div>
  );
}

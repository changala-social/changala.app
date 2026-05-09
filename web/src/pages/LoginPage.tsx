import { useState, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { useAuth } from "../context/AuthContext";
import { LoadingSpinner } from "../components/common/LoadingSpinner";
import { ErrorMessage } from "../components/common/ErrorMessage";

const RING_URL =
  import.meta.env.VITE_RING_URL || "https://changala.tail477f2f.ts.net";

export default function LoginPage() {
  const navigate = useNavigate();
  const { isAuthenticated, login } = useAuth();

  const [handle, setHandle] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [checkingSession, setCheckingSession] = useState(true);

  // Already authenticated via AuthContext — redirect immediately
  useEffect(() => {
    if (isAuthenticated) {
      navigate("/dashboard", { replace: true });
    }
  }, [isAuthenticated, navigate]);

  // On mount: probe GET /auth/session to detect a valid atrg_session cookie.
  // This handles the redirect-back from atrg-auth after successful OAuth.
  useEffect(() => {
    let cancelled = false;

    async function checkSession() {
      try {
        const res = await fetch(`${RING_URL}/auth/session`, {
          credentials: "include",
        });

        if (cancelled) return;

        if (res.ok) {
          const session: { did: string; handle: string; expires_at?: string } =
            await res.json();

          // Store a sentinel token — real auth is cookie-based
          login("cookie-session", session.did, session.handle);
          navigate("/dashboard", { replace: true });
          return;
        }

        // Any non-200 (including 401) means no valid session — show form
      } catch {
        // Network error — Ring unreachable; show form anyway
      }

      if (!cancelled) {
        setCheckingSession(false);
      }
    }

    checkSession();
    return () => {
      cancelled = true;
    };
  }, [login, navigate]);

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError(null);

    const trimmed = handle.trim();
    if (!trimmed) {
      setError("Please enter your AT Protocol handle.");
      return;
    }

    // Redirect browser to atrg-auth OAuth entry point.
    // After the full OAuth dance the browser lands back on this page
    // with an atrg_session cookie set.
    const redirectAfter = `${window.location.origin}/login`;
    window.location.href = `${RING_URL}/auth/login?handle=${encodeURIComponent(trimmed)}&redirect_after=${encodeURIComponent(redirectAfter)}`;
  }

  // Show spinner while we probe the session cookie
  if (checkingSession) {
    return (
      <div className="flex min-h-[60vh] items-center justify-center">
        <div className="text-center space-y-4">
          <LoadingSpinner size="lg" />
          <p className="text-sm text-text-muted">Checking session&hellip;</p>
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
          Your identity lives on the AT Protocol — Changala never owns your
          account.
        </p>
      </div>
    </div>
  );
}

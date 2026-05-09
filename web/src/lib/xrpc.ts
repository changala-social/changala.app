import { clearAuth } from "./auth";

const RING_URL =
  import.meta.env.VITE_RING_URL || "https://changala-ring.tail477f2f.ts.net";
const GLOBALVIEW_URL =
  import.meta.env.VITE_GLOBALVIEW_URL || "https://changala.tail477f2f.ts.net";

function getBaseUrl(nsid: string): string {
  return nsid.startsWith("app.changala.globalview.")
    ? GLOBALVIEW_URL
    : RING_URL;
}

function authHeaders(): Record<string, string> {
  const token = localStorage.getItem("changala_access_token");
  return token ? { Authorization: `Bearer ${token}` } : {};
}

export class XrpcError extends Error {
  error: string;
  status: number;

  constructor(error: string, message: string, status: number) {
    super(message);
    this.name = "XrpcError";
    this.error = error;
    this.status = status;
  }
}

export async function xrpcGet<T>(
  nsid: string,
  params?: Record<string, string>,
): Promise<T> {
  const base = getBaseUrl(nsid);
  const url = new URL(`/xrpc/${nsid}`, base);
  if (params) {
    Object.entries(params).forEach(([k, v]) => {
      if (v !== undefined && v !== null) url.searchParams.set(k, v);
    });
  }
  const res = await fetch(url.toString(), {
    headers: authHeaders(),
    credentials: "include",
  });
  if (!res.ok) {
    const body = await res
      .json()
      .catch(() => ({ error: "UnknownError", message: res.statusText }));
    // Auto-logout on expired/invalid session
    if (res.status === 401) {
      clearExpiredSession();
    }
    throw new XrpcError(
      body.error || "UnknownError",
      body.message || res.statusText,
      res.status,
    );
  }
  return res.json();
}

export async function xrpcPost<T>(nsid: string, body?: unknown): Promise<T> {
  const base = getBaseUrl(nsid);
  const res = await fetch(`${base}/xrpc/${nsid}`, {
    method: "POST",
    headers: { "Content-Type": "application/json", ...authHeaders() },
    body: body ? JSON.stringify(body) : undefined,
    credentials: "include",
  });
  if (!res.ok) {
    const errBody = await res
      .json()
      .catch(() => ({ error: "UnknownError", message: res.statusText }));
    if (res.status === 401) {
      clearExpiredSession();
    }
    throw new XrpcError(
      errBody.error || "UnknownError",
      errBody.message || res.statusText,
      res.status,
    );
  }
  return res.json();
}

/// Clear auth state and redirect to login when session expires.
function clearExpiredSession() {
  const token = localStorage.getItem("changala_access_token");
  if (token) {
    clearAuth();
    // Redirect to login — use window.location to force full page reload
    // so AuthContext re-reads the cleared localStorage.
    window.location.href = "/login";
  }
}

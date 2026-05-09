/**
 * Auth storage helpers.
 *
 * After OAuth login, the real atrg session ID is stored as the token.
 * The xrpc layer sends it as `Authorization: Bearer <session_id>` on
 * every request. The backend's RequireAuth extractor looks up the
 * session ID in the atrg_sessions table.
 *
 * `credentials: 'include'` is also set on fetch calls so the
 * atrg_session cookie is sent for same-origin requests, but for
 * cross-origin (CF Pages → Tailscale Ring) the Bearer header is
 * the primary auth mechanism.
 */
const TOKEN_KEY = "changala_access_token";
const DID_KEY = "changala_did";
const HANDLE_KEY = "changala_handle";

export function getStoredToken(): string | null {
  return localStorage.getItem(TOKEN_KEY);
}

export function getStoredDid(): string | null {
  return localStorage.getItem(DID_KEY);
}

export function getStoredHandle(): string | null {
  return localStorage.getItem(HANDLE_KEY);
}

export function storeAuth(token: string, did: string, handle: string): void {
  localStorage.setItem(TOKEN_KEY, token);
  localStorage.setItem(DID_KEY, did);
  localStorage.setItem(HANDLE_KEY, handle);
}

export function clearAuth(): void {
  localStorage.removeItem(TOKEN_KEY);
  localStorage.removeItem(DID_KEY);
  localStorage.removeItem(HANDLE_KEY);
}

export function isAuthenticated(): boolean {
  return !!getStoredToken();
}

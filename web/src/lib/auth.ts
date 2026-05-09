/**
 * Auth storage helpers.
 *
 * With atrg-auth v0.1.3 the real session is managed via an HttpOnly
 * `atrg_session` cookie — the browser sends it automatically on every
 * request to the Ring URL (fetch must use `credentials: 'include'`).
 *
 * The token stored here is the sentinel value `"cookie-session"` which
 * lets AuthContext know the user is logged in without exposing a real
 * Bearer token to JavaScript. The stored `did` and `handle` are used
 * purely for UI display.
 *
 * If a real Bearer token is stored (e.g. for API testing), the xrpc
 * layer will send it as an Authorization header alongside the cookie.
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

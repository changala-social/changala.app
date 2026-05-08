const TOKEN_KEY = 'changala_access_token';
const DID_KEY = 'changala_did';
const HANDLE_KEY = 'changala_handle';

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

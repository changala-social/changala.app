import {
  createContext,
  useContext,
  useState,
  useCallback,
  type ReactNode,
} from "react";
import {
  storeAuth,
  clearAuth,
  getStoredToken,
  getStoredDid,
  getStoredHandle,
} from "../lib/auth";

interface AuthState {
  token: string | null;
  did: string | null;
  handle: string | null;
}

interface AuthContextValue extends AuthState {
  isAuthenticated: boolean;
  login: (token: string, did: string, handle: string) => void;
  logout: () => void;
}

const AuthContext = createContext<AuthContextValue | undefined>(undefined);

export function AuthProvider({ children }: { children: ReactNode }) {
  const [auth, setAuth] = useState<AuthState>(() => ({
    token: getStoredToken(),
    did: getStoredDid(),
    handle: getStoredHandle(),
  }));

  const login = useCallback((token: string, did: string, handle: string) => {
    storeAuth(token, did, handle);
    setAuth({ token, did, handle });
  }, []);

  const logout = useCallback(() => {
    clearAuth();
    setAuth({ token: null, did: null, handle: null });
  }, []);

  return (
    <AuthContext.Provider
      value={{ ...auth, isAuthenticated: !!auth.token, login, logout }}
    >
      {children}
    </AuthContext.Provider>
  );
}

// eslint-disable-next-line react-refresh/only-export-components
export function useAuth(): AuthContextValue {
  const ctx = useContext(AuthContext);
  if (!ctx) throw new Error("useAuth must be used within AuthProvider");
  return ctx;
}

import { Link, useNavigate } from "react-router-dom";
import { ModeToggle } from "./ModeToggle";
import { NotificationBell } from "../common/NotificationBell";
import { useAuth } from "../../context/AuthContext";
import { useState } from "react";
import { SearchBar } from "../common/SearchBar";

export function Header() {
  const { isAuthenticated, handle, logout } = useAuth();
  const navigate = useNavigate();
  const [menuOpen, setMenuOpen] = useState(false);

  return (
    <header className="sticky top-0 z-50 bg-surface/80 backdrop-blur border-b border-border">
      <div className="max-w-6xl mx-auto px-4 h-14 flex items-center gap-3">
        <Link to="/" className="font-bold text-lg text-text shrink-0">
          Changala
        </Link>

        <div className="flex-1 max-w-md hidden sm:block">
          <SearchBar />
        </div>

        <ModeToggle />

        <div className="flex items-center gap-2 ml-auto">
          {isAuthenticated && <NotificationBell />}
          {isAuthenticated ? (
            <div className="relative">
              <button
                onClick={() => setMenuOpen(!menuOpen)}
                className="text-sm px-3 py-1.5 rounded-lg bg-surface-alt hover:bg-surface-hover text-text-secondary transition"
              >
                @{handle || "me"}
              </button>
              {menuOpen && (
                <div className="absolute right-0 top-full mt-1 bg-surface border border-border rounded-lg shadow-lg py-1 min-w-[160px]">
                  <Link
                    to="/dashboard"
                    className="block px-4 py-2 text-sm hover:bg-surface-hover text-text"
                    onClick={() => setMenuOpen(false)}
                  >
                    Dashboard
                  </Link>
                  <Link
                    to="/schedule"
                    className="block px-4 py-2 text-sm hover:bg-surface-hover text-text"
                    onClick={() => setMenuOpen(false)}
                  >
                    My Schedule
                  </Link>
                  <Link
                    to={`/profile/${handle}`}
                    className="block px-4 py-2 text-sm hover:bg-surface-hover text-text"
                    onClick={() => setMenuOpen(false)}
                  >
                    Profile
                  </Link>
                  <button
                    onClick={() => {
                      logout();
                      setMenuOpen(false);
                      navigate("/");
                    }}
                    className="block w-full text-left px-4 py-2 text-sm hover:bg-surface-hover text-cancelled"
                  >
                    Logout
                  </button>
                </div>
              )}
            </div>
          ) : (
            <Link
              to="/login"
              className="text-sm px-3 py-1.5 rounded-lg bg-academic text-white hover:bg-academic/90 transition"
            >
              Login
            </Link>
          )}
        </div>
      </div>
      <div className="sm:hidden px-4 pb-2">
        <SearchBar />
      </div>
    </header>
  );
}

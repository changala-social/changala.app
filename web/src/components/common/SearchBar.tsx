import { useState } from 'react';
import { useNavigate } from 'react-router-dom';

export function SearchBar() {
  const [query, setQuery] = useState('');
  const navigate = useNavigate();

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (query.trim()) {
      navigate(`/search?q=${encodeURIComponent(query.trim())}`);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="w-full">
      <input
        type="search"
        value={query}
        onChange={(e) => setQuery(e.target.value)}
        placeholder="Search courses, notes, brain nodes..."
        className="w-full px-3 py-1.5 text-sm rounded-lg bg-surface-alt border border-border text-text placeholder:text-text-muted focus:outline-none focus:ring-2 focus:ring-academic/50 focus:border-academic transition"
      />
    </form>
  );
}

import { Link } from 'react-router-dom';
import { useTrendingKeywords } from '../hooks/useKeywords';
import { useTrendingBrainTags } from '../hooks/useBrain';
import { useMode } from '../context/ModeContext';
import { LoadingSpinner } from '../components/common/LoadingSpinner';

export default function LandingPage() {
  const { mode } = useMode();
  const { data: keywordsData, isLoading: keywordsLoading } = useTrendingKeywords();
  const { data: brainTagsData, isLoading: brainTagsLoading } = useTrendingBrainTags();

  const showBrainTags = mode === 'brain' || mode === 'all';

  return (
    <div className="min-h-screen flex flex-col">
      {/* Hero */}
      <section className="flex flex-col items-center justify-center px-4 pt-24 pb-16 text-center">
        <h1 className="text-5xl font-bold text-text tracking-tight sm:text-6xl">
          Changala
        </h1>
        <p className="mt-4 max-w-xl text-lg text-text-secondary">
          Preserve knowledge. Make it social. Give it to anyone who seeks.
        </p>

        <nav className="mt-8 flex flex-wrap items-center justify-center gap-3">
          <Link
            to="/courses"
            className="px-5 py-2.5 rounded-lg bg-academic text-white font-medium text-sm hover:opacity-90 transition"
          >
            Browse Courses
          </Link>
          <Link
            to="/brain"
            className="px-5 py-2.5 rounded-lg border border-border bg-surface text-text font-medium text-sm hover:bg-surface-hover transition"
          >
            Explore Brain
          </Link>
          <Link
            to="/search"
            className="px-5 py-2.5 rounded-lg border border-border bg-surface text-text font-medium text-sm hover:bg-surface-hover transition"
          >
            Search
          </Link>
        </nav>
      </section>

      {/* Trending Keywords */}
      <section className="max-w-3xl w-full mx-auto px-4 pb-12">
        <h2 className="text-lg font-semibold text-text mb-4">Trending Keywords</h2>
        {keywordsLoading ? (
          <LoadingSpinner size="sm" />
        ) : keywordsData && keywordsData.keywords.length > 0 ? (
          <div className="flex flex-wrap gap-2">
            {keywordsData.keywords.map((kw) => (
              <Link
                key={`${kw.sessionUri}-${kw.text}`}
                to={`/session/${encodeURIComponent(kw.sessionUri)}`}
                className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-full border border-border bg-surface text-sm text-text hover:bg-surface-hover hover:border-academic/40 transition"
              >
                <span>{kw.text}</span>
                <span className="text-xs text-text-muted">({kw.count})</span>
              </Link>
            ))}
          </div>
        ) : (
          <p className="text-sm text-text-muted">No trending keywords right now.</p>
        )}
        {keywordsData?.computedAt && (
          <p className="mt-2 text-xs text-text-muted">
            Updated {new Date(keywordsData.computedAt).toLocaleString()}
          </p>
        )}
      </section>

      {/* Trending Brain Tags — only in brain or all mode */}
      {showBrainTags && (
        <section className="max-w-3xl w-full mx-auto px-4 pb-16">
          <h2 className="text-lg font-semibold text-text mb-4">Trending Brain Tags</h2>
          {brainTagsLoading ? (
            <LoadingSpinner size="sm" />
          ) : brainTagsData && brainTagsData.tags.length > 0 ? (
            <div className="flex flex-wrap gap-2">
              {brainTagsData.tags.map((t) => (
                <Link
                  key={t.tag}
                  to={`/brain?tag=${encodeURIComponent(t.tag)}`}
                  className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-full border border-border bg-surface text-sm text-text hover:bg-surface-hover hover:border-academic/40 transition"
                >
                  <span>#{t.tag}</span>
                  <span className="text-xs text-text-muted">({t.count})</span>
                </Link>
              ))}
            </div>
          ) : (
            <p className="text-sm text-text-muted">No trending brain tags right now.</p>
          )}
          {brainTagsData?.computedAt && (
            <p className="mt-2 text-xs text-text-muted">
              Updated {new Date(brainTagsData.computedAt).toLocaleString()}
            </p>
          )}
        </section>
      )}
    </div>
  );
}

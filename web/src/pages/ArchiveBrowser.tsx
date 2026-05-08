import { useState, type FormEvent } from 'react';
import { useGlobalArchiveFeed, useSearchArchive } from '../hooks/useArchive';
import { ArchiveCard } from '../components/common/ArchiveCard';
import { LoadingSpinner } from '../components/common/LoadingSpinner';
import { ErrorMessage } from '../components/common/ErrorMessage';

export default function ArchiveBrowser() {
  const [searchInput, setSearchInput] = useState('');
  const [searchQuery, setSearchQuery] = useState('');
  const [feedCursor, setFeedCursor] = useState<string | undefined>(undefined);
  const [searchCursor, setSearchCursor] = useState<string | undefined>(undefined);

  // Accumulated items for pagination
  const [accFeedItems, setAccFeedItems] = useState<typeof feedItems>([]);
  const [accSearchResults, setAccSearchResults] = useState<typeof searchResults>([]);

  const isSearching = searchQuery.length > 0;

  const {
    data: feedData,
    isLoading: feedLoading,
    error: feedError,
    refetch: feedRefetch,
  } = useGlobalArchiveFeed(feedCursor);

  const {
    data: searchData,
    isLoading: searchLoading,
    error: searchError,
    refetch: searchRefetch,
  } = useSearchArchive(searchQuery, searchCursor);

  const feedItems = feedData?.items ?? [];
  const searchResults = searchData?.results ?? [];

  const allFeedItems = feedCursor && accFeedItems.length > 0
    ? [...accFeedItems, ...feedItems]
    : feedItems;

  const allSearchResults = searchCursor && accSearchResults.length > 0
    ? [...accSearchResults, ...searchResults]
    : searchResults;

  const isLoading = isSearching ? searchLoading : feedLoading;
  const error = isSearching ? searchError : feedError;
  const nextCursor = isSearching ? searchData?.cursor : feedData?.cursor;

  const handleSearch = (e: FormEvent) => {
    e.preventDefault();
    const trimmed = searchInput.trim();
    setSearchQuery(trimmed);
    setSearchCursor(undefined);
    setAccSearchResults([]);
  };

  const handleClearSearch = () => {
    setSearchInput('');
    setSearchQuery('');
    setSearchCursor(undefined);
    setAccSearchResults([]);
  };

  const handleLoadMore = () => {
    if (!nextCursor) return;
    if (isSearching) {
      setAccSearchResults(allSearchResults);
      setSearchCursor(nextCursor);
    } else {
      setAccFeedItems(allFeedItems);
      setFeedCursor(nextCursor);
    }
  };

  return (
    <div className="max-w-5xl mx-auto px-4 py-8">
      <h1 className="text-2xl font-bold text-text mb-6">Archive Browser</h1>

      {/* Search input */}
      <form onSubmit={handleSearch} className="mb-8">
        <div className="flex gap-2">
          <input
            type="text"
            value={searchInput}
            onChange={(e) => setSearchInput(e.target.value)}
            placeholder="Search archives by course, semester…"
            className="flex-1 px-4 py-2.5 rounded-lg border border-border bg-surface text-text placeholder:text-text-muted text-sm focus:outline-none focus:ring-2 focus:ring-academic/40"
          />
          <button
            type="submit"
            className="px-5 py-2.5 rounded-lg bg-academic text-white text-sm font-medium hover:bg-academic/90 transition"
          >
            Search
          </button>
          {isSearching && (
            <button
              type="button"
              onClick={handleClearSearch}
              className="px-4 py-2.5 rounded-lg border border-border bg-surface text-text text-sm font-medium hover:bg-surface-hover transition"
            >
              Clear
            </button>
          )}
        </div>
      </form>

      {/* Results header */}
      {isSearching && searchData && (
        <p className="text-sm text-text-secondary mb-4">
          {searchData.hitsTotal} {searchData.hitsTotal === 1 ? 'result' : 'results'} for &ldquo;{searchQuery}&rdquo;
        </p>
      )}

      {/* Content */}
      {isLoading && (isSearching ? accSearchResults.length === 0 : accFeedItems.length === 0) ? (
        <LoadingSpinner size="lg" />
      ) : error ? (
        <ErrorMessage
          title={isSearching ? 'Search failed' : 'Failed to load archives'}
          message={error instanceof Error ? error.message : 'An unexpected error occurred.'}
          retry={() => isSearching ? searchRefetch() : feedRefetch()}
        />
      ) : (isSearching ? allSearchResults : allFeedItems).length === 0 ? (
        <div className="text-center py-16">
          <p className="text-text-muted">
            {isSearching ? 'No archives match your search.' : 'No sealed archives yet.'}
          </p>
        </div>
      ) : (
        <>
          <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
            {isSearching
              ? allSearchResults.map((archive) => (
                  <ArchiveCard
                    key={archive.archiveUri}
                    archiveUri={archive.archiveUri}
                    courseTitle={`${archive.courseUri}`}
                    semester={archive.semester}
                    sessionCount={archive.sessionCount}
                    sealedAt={archive.sealedAt}
                    iaUrl={archive.internetArchiveUrl}
                  />
                ))
              : allFeedItems.map((item) => (
                  <ArchiveCard
                    key={item.archiveUri}
                    archiveUri={item.archiveUri}
                    courseTitle={item.courseTitle}
                    semester={item.semester}
                    sessionCount={item.sessionCount}
                    sealedAt={item.sealedAt}
                    iaUrl={item.iaUrl}
                  />
                ))}
          </div>

          {/* Load more */}
          {nextCursor && (
            <div className="flex justify-center mt-8">
              <button
                onClick={handleLoadMore}
                disabled={isLoading}
                className="px-5 py-2 rounded-lg border border-border bg-surface text-text text-sm font-medium hover:bg-surface-hover transition disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {isLoading ? 'Loading…' : 'Load more'}
              </button>
            </div>
          )}
        </>
      )}
    </div>
  );
}

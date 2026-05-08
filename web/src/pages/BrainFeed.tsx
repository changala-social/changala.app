import { useState } from 'react';
import { useBrainFeed, useTrendingBrainTags } from '../hooks/useBrain';
import { BrainNodeCard } from '../components/brain/BrainNodeCard';
import { LoadingSpinner } from '../components/common/LoadingSpinner';
import { ErrorMessage } from '../components/common/ErrorMessage';

export default function BrainFeed() {
  const [selectedTag, setSelectedTag] = useState<string | null>(null);
  const [cursor, setCursor] = useState<string | undefined>(undefined);

  const {
    data: feedData,
    isLoading: feedLoading,
    isError: feedError,
    error: feedErr,
    refetch: refetchFeed,
  } = useBrainFeed(cursor, selectedTag ?? undefined);

  const {
    data: trendingData,
    isLoading: trendingLoading,
  } = useTrendingBrainTags();

  const handleTagClick = (tag: string) => {
    setCursor(undefined);
    setSelectedTag((prev) => (prev === tag ? null : tag));
  };

  return (
    <div className="max-w-4xl mx-auto px-4 py-8">
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-2xl font-bold text-text">Brain Feed</h1>
        <a
          href="/brain/new"
          className="inline-flex items-center gap-1.5 px-4 py-2 rounded-lg bg-brain text-white text-sm font-medium hover:bg-brain/90 transition"
        >
          + New Node
        </a>
      </div>

      {/* Trending tag pills */}
      <div className="mb-6">
        <h2 className="text-xs font-semibold uppercase tracking-wider text-text-muted mb-2">
          Trending Tags
        </h2>
        <div className="flex flex-wrap gap-2">
          {trendingLoading && (
            <span className="text-xs text-text-muted">Loading tags…</span>
          )}
          {trendingData?.tags.map(({ tag, count }) => (
            <button
              key={tag}
              onClick={() => handleTagClick(tag)}
              className={`text-sm px-3 py-1 rounded-full border transition ${
                selectedTag === tag
                  ? 'bg-brain text-white border-brain'
                  : 'bg-surface border-border text-text-secondary hover:border-brain hover:text-brain'
              }`}
            >
              {tag}
              <span className="ml-1.5 text-xs opacity-70">{count}</span>
            </button>
          ))}
          {selectedTag && (
            <button
              onClick={() => {
                setSelectedTag(null);
                setCursor(undefined);
              }}
              className="text-sm px-3 py-1 rounded-full border border-border text-text-muted hover:text-text transition"
            >
              Clear filter ×
            </button>
          )}
        </div>
      </div>

      {/* Feed */}
      {feedLoading && !feedData && <LoadingSpinner />}

      {feedError && (
        <ErrorMessage
          title="Failed to load brain feed"
          message={feedErr instanceof Error ? feedErr.message : 'Could not fetch brain nodes.'}
          retry={refetchFeed}
        />
      )}

      {feedData && (
        <>
          {feedData.nodes.length === 0 ? (
            <div className="text-center py-16 text-text-muted">
              <p className="text-lg">No brain nodes found.</p>
              {selectedTag && (
                <p className="text-sm mt-1">
                  Try clearing the tag filter or exploring a different tag.
                </p>
              )}
            </div>
          ) : (
            <div className="space-y-3">
              {feedData.nodes.map((node) => (
                <BrainNodeCard key={node.uri} node={node} />
              ))}
            </div>
          )}

          {/* Load more */}
          {feedData.cursor && (
            <div className="mt-8 text-center">
              <button
                onClick={() => setCursor(feedData.cursor)}
                className="px-6 py-2 rounded-lg border border-border bg-surface text-text-secondary hover:bg-surface-hover hover:text-text transition text-sm font-medium"
              >
                Load more
              </button>
            </div>
          )}
        </>
      )}
    </div>
  );
}

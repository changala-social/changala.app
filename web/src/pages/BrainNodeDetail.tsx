import { useParams, Link } from 'react-router-dom';
import { useNodeContent } from '../hooks/useBrain';
import { useXrpcQuery } from '../hooks/useXrpc';
import { useAuth } from '../context/AuthContext';
import type { BrainNode, GetBrainFeedResponse } from '../generated/types';
import { ContentRenderer } from '../components/content/ContentRenderer';
import { BacklinkList } from '../components/brain/BacklinkList';
import { MiniGraph } from '../components/brain/MiniGraph';
import { VoteButton } from '../components/common/VoteButton';
import { AuthorName } from '../components/common/AuthorName';
import { LoadingSpinner } from '../components/common/LoadingSpinner';
import { ErrorMessage } from '../components/common/ErrorMessage';

export default function BrainNodeDetail() {
  const { uri: rawUri } = useParams<{ uri: string }>();
  const uri = rawUri ? decodeURIComponent(rawUri) : '';
  const { did: authedDid } = useAuth();

  // Fetch node metadata via the brain feed endpoint filtered to this node
  const {
    data: metaData,
    isLoading: metaLoading,
    isError: metaError,
    error: metaErr,
    refetch: refetchMeta,
  } = useXrpcQuery<GetBrainFeedResponse>(
    'app.changala.globalview.getBrainFeed',
    { nodeUri: uri },
    { enabled: !!uri }
  );

  // Fetch raw content from the Ring
  const {
    data: contentData,
    isLoading: contentLoading,
    isError: contentError,
    error: contentErr,
    refetch: refetchContent,
  } = useNodeContent(uri);

  const node: BrainNode | undefined = metaData?.nodes?.[0];
  const isAuthor = !!(authedDid && node && node.authorDid === authedDid);

  if (!uri) {
    return (
      <div className="max-w-3xl mx-auto px-4 py-8">
        <ErrorMessage title="Missing node URI" message="No brain node URI was provided in the URL." />
      </div>
    );
  }

  if (metaLoading || contentLoading) {
    return (
      <div className="max-w-3xl mx-auto px-4 py-8">
        <LoadingSpinner />
      </div>
    );
  }

  if (metaError || contentError) {
    const message =
      (metaErr instanceof Error ? metaErr.message : '') ||
      (contentErr instanceof Error ? contentErr.message : '') ||
      'Could not load this brain node.';
    return (
      <div className="max-w-3xl mx-auto px-4 py-8">
        <ErrorMessage
          title="Failed to load brain node"
          message={message}
          retry={() => { refetchMeta(); refetchContent(); }}
        />
      </div>
    );
  }

  return (
    <div className="max-w-3xl mx-auto px-4 py-8">
      {/* Navigation */}
      <Link
        to="/brain"
        className="inline-flex items-center gap-1 text-sm text-brain hover:underline mb-6"
      >
        <span aria-hidden="true">&larr;</span>
        Brain Feed
      </Link>

      {/* Header */}
      <div className="rounded-lg border border-border bg-surface p-6 mb-6">
        <div className="flex items-start justify-between gap-4 mb-4">
          <div className="flex-1 min-w-0">
            <h1 className="text-2xl font-bold text-text">
              {node?.title ?? 'Brain Node'}
            </h1>
            {node && (
              <div className="flex items-center gap-3 mt-2 text-sm text-text-muted">
                <AuthorName did={node.authorDid} />
                <span>v{node.version}</span>
                <span>{new Date(node.createdAt).toLocaleDateString()}</span>
              </div>
            )}
          </div>

          {node && (
            <div className="flex items-center gap-2 shrink-0">
              <VoteButton subjectUri={node.uri} initialCount={node.voteCount} />
            </div>
          )}
        </div>

        {/* Tags */}
        {node && node.tags.length > 0 && (
          <div className="flex flex-wrap gap-1.5 mb-4">
            {node.tags.map((tag) => (
              <Link
                key={tag}
                to={`/brain?tag=${encodeURIComponent(tag)}`}
                className="text-xs px-2 py-0.5 rounded-full bg-brain/10 text-brain hover:bg-brain/20 transition"
              >
                {tag}
              </Link>
            ))}
          </div>
        )}

        {/* Format badge + academic ref */}
        <div className="flex items-center gap-2 mb-4">
          {node && (
            <span className="text-[10px] uppercase tracking-wider px-2 py-0.5 rounded bg-surface-alt text-text-muted">
              {node.format}
            </span>
          )}
          {node?.academicRef && (
            <Link
              to={`/session/${encodeURIComponent(node.academicRef)}`}
              className="text-xs px-2 py-0.5 rounded bg-academic/10 text-academic hover:bg-academic/20 transition"
            >
              Linked to academic session →
            </Link>
          )}
        </div>

        {/* Node summary */}
        {node?.summary && (
          <p className="text-sm text-text-secondary italic border-l-2 border-brain/30 pl-3 mb-4">
            {node.summary}
          </p>
        )}

        {/* Content */}
        {contentData && (
          <ContentRenderer format={contentData.format} content={contentData.content} />
        )}
      </div>

      {/* Actions */}
      <div className="flex items-center gap-3 mb-6">
        <Link
          to={`/graph/${encodeURIComponent(uri)}`}
          className="px-4 py-2 rounded-lg border border-border bg-surface text-text-secondary hover:bg-surface-hover hover:text-text text-sm font-medium transition"
        >
          View full graph
        </Link>
        {isAuthor && (
          <Link
            to={`/brain/${encodeURIComponent(uri)}/edit`}
            className="px-4 py-2 rounded-lg border border-brain bg-brain/10 text-brain hover:bg-brain/20 text-sm font-medium transition"
          >
            Edit
          </Link>
        )}
      </div>

      {/* Backlinks */}
      <div className="mb-6">
        <h2 className="text-lg font-semibold text-text mb-3">Backlinks</h2>
        <BacklinkList nodeUri={uri} />
      </div>

      {/* Mini graph */}
      <div>
        <h2 className="text-lg font-semibold text-text mb-3">Graph Neighbourhood</h2>
        <div className="rounded-lg border border-border bg-surface p-4">
          <MiniGraph nodeUri={uri} />
        </div>
      </div>
    </div>
  );
}

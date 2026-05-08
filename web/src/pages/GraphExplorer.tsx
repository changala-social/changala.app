import { useState } from 'react';
import { useParams, Link } from 'react-router-dom';
import { useNodeGraph } from '../hooks/useGraph';
import { GraphView } from '../components/brain/GraphView';
import { LoadingSpinner } from '../components/common/LoadingSpinner';
import { ErrorMessage } from '../components/common/ErrorMessage';

const MIN_DEPTH = 1;
const MAX_DEPTH = 3;
const DEFAULT_DEPTH = 2;

export default function GraphExplorer() {
  const { uri: rawUri } = useParams<{ uri: string }>();
  const uri = rawUri ? decodeURIComponent(rawUri) : '';
  const [depth, setDepth] = useState(DEFAULT_DEPTH);

  const {
    data: graphData,
    isLoading,
    isError,
    error,
    refetch,
  } = useNodeGraph(uri, depth);

  // Find the center node's title from the graph data
  const centerNode = graphData?.nodes.find((n) => n.uri === uri);

  if (!uri) {
    return (
      <div className="max-w-5xl mx-auto px-4 py-8">
        <ErrorMessage title="Missing node URI" message="No brain node URI was provided for graph exploration." />
      </div>
    );
  }

  return (
    <div className="flex flex-col h-screen">
      {/* Top bar */}
      <div className="flex items-center justify-between px-4 py-3 border-b border-border bg-surface shrink-0">
        <div className="flex items-center gap-4 min-w-0">
          <Link
            to={`/brain/${encodeURIComponent(uri)}`}
            className="inline-flex items-center gap-1 text-sm text-brain hover:underline shrink-0"
          >
            <span aria-hidden="true">&larr;</span>
            Back to node
          </Link>

          {centerNode && (
            <div className="min-w-0">
              <h1 className="text-lg font-semibold text-text truncate">
                {centerNode.title}
              </h1>
              {centerNode.summary && (
                <p className="text-xs text-text-muted truncate">{centerNode.summary}</p>
              )}
            </div>
          )}
        </div>

        {/* Depth control */}
        <div className="flex items-center gap-3 shrink-0">
          <label htmlFor="depth-slider" className="text-sm text-text-secondary">
            Depth: <span className="font-semibold text-text">{depth}</span>
          </label>
          <input
            id="depth-slider"
            type="range"
            min={MIN_DEPTH}
            max={MAX_DEPTH}
            step={1}
            value={depth}
            onChange={(e) => setDepth(Number(e.target.value))}
            className="w-24 accent-brain"
          />
          {graphData && (
            <span className="text-xs text-text-muted">
              {graphData.nodes.length} nodes · {graphData.edges.length} edges
            </span>
          )}
        </div>
      </div>

      {/* Graph area */}
      <div className="flex-1 relative bg-surface-alt">
        {isLoading && (
          <div className="absolute inset-0 flex items-center justify-center">
            <LoadingSpinner />
          </div>
        )}

        {isError && (
          <div className="absolute inset-0 flex items-center justify-center p-8">
            <ErrorMessage
              title="Failed to load graph"
              message={error instanceof Error ? error.message : 'Could not fetch the node graph.'}
              retry={refetch}
            />
          </div>
        )}

        {graphData && (
          <GraphView
            data={graphData}
            centerUri={uri}
            width={typeof window !== 'undefined' ? window.innerWidth : 1200}
            height={typeof window !== 'undefined' ? window.innerHeight - 64 : 800}
          />
        )}
      </div>
    </div>
  );
}

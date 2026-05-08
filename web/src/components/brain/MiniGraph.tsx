import { useNodeGraph } from '../../hooks/useGraph';
import { GraphView } from './GraphView';
import { Link } from 'react-router-dom';
import { LoadingSpinner } from '../common/LoadingSpinner';

interface MiniGraphProps {
  nodeUri: string;
}

export function MiniGraph({ nodeUri }: MiniGraphProps) {
  const { data, isLoading } = useNodeGraph(nodeUri, 1);

  if (isLoading) return <LoadingSpinner size="sm" />;
  if (!data || !data.nodes.length) return null;

  return (
    <div className="rounded-lg border border-border bg-surface p-4">
      <div className="flex items-center justify-between mb-2">
        <h3 className="text-sm font-semibold text-text">Graph</h3>
        <Link to={`/graph/${encodeURIComponent(nodeUri)}`} className="text-xs text-brain hover:underline">
          Expand &rarr;
        </Link>
      </div>
      <div className="h-48">
        <GraphView data={data} centerUri={nodeUri} width={400} height={192} />
      </div>
    </div>
  );
}

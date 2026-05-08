import { useXrpcQuery } from './useXrpc';
import type { GetNodeGraphResponse, GetNeighboursResponse } from '../generated/types';

export function useNodeGraph(nodeUri: string, depth: number = 2) {
  return useXrpcQuery<GetNodeGraphResponse>(
    'app.changala.globalview.getNodeGraph',
    { nodeUri, depth: String(depth) },
    { enabled: !!nodeUri }
  );
}

export function useNeighbours(nodeUri: string, maxDistance: number = 2) {
  return useXrpcQuery<GetNeighboursResponse>(
    'app.changala.globalview.getNeighbours',
    { nodeUri, maxDistance: String(maxDistance) },
    { enabled: !!nodeUri }
  );
}

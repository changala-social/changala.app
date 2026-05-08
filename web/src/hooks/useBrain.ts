import { useXrpcQuery, useXrpcMutation } from "./useXrpc";
import type {
  GetBrainFeedResponse,
  GetNodeContentResponse,
  GetBacklinksResponse,
  CreateNodeResponse,
  CreateLinkResponse,
  GetTrendingBrainTagsResponse,
  SearchBrainNodesResponse,
} from "../generated/types";

export function useBrainFeed(cursor?: string, tag?: string) {
  const params: Record<string, string> = {};
  if (cursor) params.cursor = cursor;
  if (tag) params.tag = tag;
  return useXrpcQuery<GetBrainFeedResponse>(
    "app.changala.globalview.getBrainFeed",
    params,
  );
}

export function useNodeContent(cid: string, ringDid: string = "") {
  return useXrpcQuery<GetNodeContentResponse>(
    "app.changala.ring.getNodeContent",
    { cid, ringDid },
    {
      enabled: !!cid,
    },
  );
}

export function useBacklinks(uri: string, cursor?: string) {
  const params: Record<string, string> = { nodeUri: uri };
  if (cursor) params.cursor = cursor;
  return useXrpcQuery<GetBacklinksResponse>(
    "app.changala.globalview.getBacklinks",
    params,
    {
      enabled: !!uri,
    },
  );
}

export function useTrendingBrainTags() {
  return useXrpcQuery<GetTrendingBrainTagsResponse>(
    "app.changala.globalview.getTrendingBrainTags",
  );
}

export function useCreateNode() {
  return useXrpcMutation<
    CreateNodeResponse,
    {
      title: string;
      content: string;
      format: string;
      tags?: string[];
      academicRef?: string;
      summary?: string;
    }
  >("app.changala.ring.createNode");
}

export function useVersionNode() {
  return useXrpcMutation<
    CreateNodeResponse,
    {
      nodeUri: string;
      title?: string;
      content: string;
      format: string;
      tags?: string[];
      summary?: string;
    }
  >("app.changala.ring.versionNode");
}

export function useCreateLink() {
  return useXrpcMutation<
    CreateLinkResponse,
    {
      fromUri: string;
      toUri: string;
      label?: string;
    }
  >("app.changala.ring.createLink");
}

export function useDeleteLink() {
  return useXrpcMutation<void, { linkUri: string }>(
    "app.changala.ring.deleteLink",
  );
}

export function useSearchBrainNodes(query: string, cursor?: string) {
  const params: Record<string, string> = { q: query };
  if (cursor) params.cursor = cursor;
  return useXrpcQuery<SearchBrainNodesResponse>(
    "app.changala.globalview.searchBrainNodes",
    params,
    {
      enabled: query.length > 0,
    },
  );
}

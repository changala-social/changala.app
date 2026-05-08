import { useXrpcQuery, useXrpcMutation } from './useXrpc';
import type {
  GetGlobalArchiveFeedResponse, SearchArchiveResponse,
  Archive, InitiateArchiveResponse, SealArchiveResponse,
} from '../generated/types';

export function useGlobalArchiveFeed(cursor?: string) {
  const params: Record<string, string> = {};
  if (cursor) params.cursor = cursor;
  return useXrpcQuery<GetGlobalArchiveFeedResponse>('app.changala.globalview.getGlobalArchiveFeed', params);
}

export function useSearchArchive(query: string, cursor?: string) {
  const params: Record<string, string> = { q: query };
  if (cursor) params.cursor = cursor;
  return useXrpcQuery<SearchArchiveResponse>('app.changala.globalview.searchArchive', params, {
    enabled: query.length > 0,
  });
}

export function useArchive(archiveUri: string) {
  return useXrpcQuery<Archive>('app.changala.ring.getArchive', { archiveUri }, {
    enabled: !!archiveUri,
  });
}

export function useInitiateArchive() {
  return useXrpcMutation<InitiateArchiveResponse, { courseUri: string; semester: string }>(
    'app.changala.ring.initiateArchive'
  );
}

export function useSealArchive() {
  return useXrpcMutation<SealArchiveResponse, { courseUri: string; semester: string }>(
    'app.changala.ring.sealArchive'
  );
}

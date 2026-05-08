import { useXrpcQuery, useXrpcMutation } from './useXrpc';
import type {
  GetKeywordHistogramResponse, AddKeywordResponse,
  GetTrendingKeywordsResponse,
} from '../generated/types';

export function useKeywordHistogram(sessionUri: string, live: boolean = false) {
  return useXrpcQuery<GetKeywordHistogramResponse>(
    'app.changala.globalview.getKeywordHistogram',
    { sessionUri },
    {
      enabled: !!sessionUri,
      refetchInterval: live ? 5000 : false,
    }
  );
}

export function useAddKeyword() {
  return useXrpcMutation<AddKeywordResponse, { sessionUri: string; text: string }>(
    'app.changala.ring.addKeyword'
  );
}

export function useTrendingKeywords() {
  return useXrpcQuery<GetTrendingKeywordsResponse>('app.changala.globalview.getTrendingKeywords');
}

import { useQuery, useMutation, type UseQueryOptions, type UseMutationOptions } from '@tanstack/react-query';
import { xrpcGet, xrpcPost, type XrpcError } from '../lib/xrpc';

export function useXrpcQuery<T>(
  nsid: string,
  params?: Record<string, string>,
  options?: Omit<UseQueryOptions<T, XrpcError>, 'queryKey' | 'queryFn'>
) {
  return useQuery<T, XrpcError>({
    queryKey: [nsid, params],
    queryFn: () => xrpcGet<T>(nsid, params),
    ...options,
  });
}

export function useXrpcMutation<TData, TVariables = unknown>(
  nsid: string,
  options?: Omit<UseMutationOptions<TData, XrpcError, TVariables>, 'mutationFn'>
) {
  return useMutation<TData, XrpcError, TVariables>({
    mutationFn: (variables) => xrpcPost<TData>(nsid, variables),
    ...options,
  });
}

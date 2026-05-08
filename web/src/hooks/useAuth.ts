import { useXrpcQuery } from './useXrpc';
import { useAuth } from '../context/AuthContext';
import type { GetMembershipsResponse, GetRoleResponse, GetFollowedEnrollmentsResponse } from '../generated/types';

export function useMemberships(did: string) {
  return useXrpcQuery<GetMembershipsResponse>('app.changala.ring.getMemberships', { did }, {
    enabled: !!did,
  });
}

export function useRole(did: string) {
  return useXrpcQuery<GetRoleResponse>('app.changala.ring.getRole', { did }, {
    enabled: !!did,
  });
}

export function useFollowedEnrollments() {
  return useXrpcQuery<GetFollowedEnrollmentsResponse>('app.changala.globalview.getFollowedEnrollments');
}

export { useAuth };

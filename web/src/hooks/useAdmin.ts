import { useXrpcQuery, useXrpcMutation } from './useXrpc';
import type { AuditLogResponse, ListApiKeysResponse, ListBansResponse } from '../generated/types';

export function useAuditLog() {
  return useXrpcQuery<AuditLogResponse>('app.changala.ring.getAuditLog');
}

export function useListBans() {
  return useXrpcQuery<ListBansResponse>('app.changala.ring.listBans');
}

export function useBanDid() {
  return useXrpcMutation<void, { targetDid: string; reason?: string; ttlSeconds?: number }>(
    'app.changala.ring.banDid'
  );
}

export function useLiftBan() {
  return useXrpcMutation<void, { targetDid: string }>('app.changala.ring.liftBan');
}

export function useListApiKeys() {
  return useXrpcQuery<ListApiKeysResponse>('app.changala.ring.listApiKeys');
}

export function useCreateApiKey() {
  return useXrpcMutation<
    { key: string; id: string; name: string; prefix: string },
    { name: string; scopes?: string[]; expiresInDays?: number }
  >('app.changala.ring.createApiKey');
}

export function useRevokeApiKey() {
  return useXrpcMutation<void, { keyId: string }>('app.changala.ring.revokeApiKey');
}

export function usePromoteRole() {
  return useXrpcMutation<void, { targetDid: string; role: string }>(
    'app.changala.ring.promoteRole'
  );
}

export function useDemoteRole() {
  return useXrpcMutation<void, { targetDid: string }>(
    'app.changala.ring.demoteRole'
  );
}

import { useXrpcQuery, useXrpcMutation } from './useXrpc';

export interface VerifyEmailResponse {
  status: string;
  membershipUri: string | null;
}

export function useVerifyEmail() {
  return useXrpcMutation<VerifyEmailResponse, {
    did: string;
    email: string;
    otp?: string;
  }>('app.changala.ring.verifyEmail');
}

export function useProvisionAdmin() {
  return useXrpcMutation<{ did: string; role: string; provisionedAt: string }, {
    did: string;
    secret: string;
    institutionDomain?: string;
  }>('app.changala.ring.provisionAdmin');
}

export function usePromoteRole() {
  return useXrpcMutation<{ targetDid: string; newRole: string; promotedBy: string; promotedAt: string }, {
    targetDid: string;
    role: string;
  }>('app.changala.ring.promoteRole');
}

export function useDemoteRole() {
  return useXrpcMutation<{ targetDid: string; newRole: string; demotedBy: string; demotedAt: string }, {
    targetDid: string;
    role: string;
  }>('app.changala.ring.demoteRole');
}

export function useAuditLog(cursor?: string) {
  const params: Record<string, string> = {};
  if (cursor) params.cursor = cursor;
  return useXrpcQuery<{
    entries: { actorDid: string; action: string; targetDid: string | null; details: unknown; createdAt: string }[];
    cursor: string | null;
  }>('app.changala.ring.getAuditLog', params);
}

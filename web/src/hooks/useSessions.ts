import { useXrpcQuery, useXrpcMutation } from './useXrpc';
import type { Session, ListSessionsResponse } from '../generated/types';

export function useSessions(courseUri: string, cursor?: string) {
  const params: Record<string, string> = { courseUri };
  if (cursor) params.cursor = cursor;
  return useXrpcQuery<ListSessionsResponse>('app.changala.ring.listSessions', params, {
    enabled: !!courseUri,
  });
}

export function useSession(uri: string) {
  return useXrpcQuery<Session>('app.changala.ring.getSession', { uri }, { enabled: !!uri });
}

export function useOpenSession() {
  return useXrpcMutation<Session, { sessionUri: string }>('app.changala.ring.openSession');
}

export function useCloseSession() {
  return useXrpcMutation<{ session: Session; keywordWindowExpiresAt: string }, { sessionUri: string }>(
    'app.changala.ring.closeSession'
  );
}

export function useCancelSession() {
  return useXrpcMutation<void, { sessionUri: string; reason?: string }>('app.changala.ring.cancelSession');
}

export function useRescheduleSession() {
  return useXrpcMutation<void, { sessionUri: string; rescheduledTo: string }>('app.changala.ring.rescheduleSession');
}

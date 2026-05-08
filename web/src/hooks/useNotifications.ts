import { useXrpcQuery, useXrpcMutation } from './useXrpc';
import type { GetNotificationsResponse } from '../generated/types';

export function useNotifications(cursor?: string) {
  const params: Record<string, string> = {};
  if (cursor) params.cursor = cursor;
  return useXrpcQuery<GetNotificationsResponse>('app.changala.globalview.getNotifications', params);
}

export function useMarkNotificationRead() {
  return useXrpcMutation<void, { notificationId: string }>('app.changala.globalview.markNotificationRead');
}

export function useMarkAllRead() {
  return useXrpcMutation<void, void>('app.changala.globalview.markAllRead');
}

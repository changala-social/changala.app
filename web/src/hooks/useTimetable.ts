import { useXrpcMutation } from './useXrpc';
import type { ProvisionSessionsResponse, LoadSlotsResponse, LoadCalendarResponse } from '../generated/types';

export function useProvisionSessions() {
  return useXrpcMutation<
    ProvisionSessionsResponse,
    { courseUri: string; slot: string; semester: string }
  >('app.changala.ring.provisionSessions');
}

export function useLoadSlots() {
  return useXrpcMutation<
    LoadSlotsResponse,
    { semester: string; slots: Record<string, { schedule: { day: string; start: string; end: string }[] }> }
  >('app.changala.ring.loadSlots');
}

export function useLoadCalendar() {
  return useXrpcMutation<
    LoadCalendarResponse,
    {
      semester: string;
      phases: { name: string; label: string; type: string; start: string; end: string }[];
      holidays: { date: string; name: string }[];
    }
  >('app.changala.ring.loadCalendar');
}

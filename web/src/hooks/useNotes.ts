import { useXrpcQuery, useXrpcMutation } from "./useXrpc";
import type {
  GetNotesResponse,
  GetNoteContentResponse,
  GetNoteHistoryResponse,
  CreateNoteResponse,
  RegisterVoteResponse,
  GetCollectiveNoteResponse,
  ListEditProposalsResponse,
  SearchNotesResponse,
} from "../generated/types";

export function useNotes(sessionUri: string, cursor?: string) {
  const params: Record<string, string> = { sessionUri };
  if (cursor) params.cursor = cursor;
  return useXrpcQuery<GetNotesResponse>(
    "app.changala.globalview.getNotes",
    params,
    {
      enabled: !!sessionUri,
    },
  );
}

export function useNoteContent(cid: string, ringDid: string = "") {
  return useXrpcQuery<GetNoteContentResponse>(
    "app.changala.ring.getNoteContent",
    { cid, ringDid },
    {
      enabled: !!cid,
    },
  );
}

export function useNoteHistory(noteUri: string) {
  return useXrpcQuery<GetNoteHistoryResponse>(
    "app.changala.ring.getNoteHistory",
    { noteUri },
    {
      enabled: !!noteUri,
    },
  );
}

export function useCreateNote() {
  return useXrpcMutation<
    CreateNoteResponse,
    {
      sessionUri: string;
      content: string;
      format: string;
      summary?: string;
    }
  >("app.changala.ring.createNote");
}

export function useRegisterVote() {
  return useXrpcMutation<RegisterVoteResponse, { subjectUri: string }>(
    "app.changala.ring.registerVote",
  );
}

export function useCollectiveNote(sessionUri: string) {
  return useXrpcQuery<GetCollectiveNoteResponse>(
    "app.changala.ring.getCollectiveNote",
    { sessionUri },
    {
      enabled: !!sessionUri,
    },
  );
}

export function useEditProposals(sessionUri: string, cursor?: string) {
  const params: Record<string, string> = { sessionUri };
  if (cursor) params.cursor = cursor;
  return useXrpcQuery<ListEditProposalsResponse>(
    "app.changala.ring.listEditProposals",
    params,
    {
      enabled: !!sessionUri,
    },
  );
}

export function useSearchNotes(query: string, cursor?: string) {
  const params: Record<string, string> = { q: query };
  if (cursor) params.cursor = cursor;
  return useXrpcQuery<SearchNotesResponse>(
    "app.changala.globalview.searchNotes",
    params,
    {
      enabled: query.length > 0,
    },
  );
}

export function useProposeEdit() {
  return useXrpcMutation<
    { proposalUri: string; diffRingRef: { ringDid: string; cid: string } },
    {
      sessionUri: string;
      diff: string;
      summary?: string;
    }
  >("app.changala.ring.proposeEdit");
}

export function useAcceptEdit() {
  return useXrpcMutation<
    {
      collectiveNoteUri: string;
      newCollectiveRingRef: { ringDid: string; cid: string };
    },
    {
      proposalUri: string;
    }
  >("app.changala.ring.acceptEdit");
}

export function useRejectEdit() {
  return useXrpcMutation<void, { proposalUri: string }>(
    "app.changala.ring.rejectEdit",
  );
}

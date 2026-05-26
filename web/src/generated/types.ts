// Auto-generated from lexicons — DO NOT EDIT

// ── Shared Defs ──────────────────────────────────────────────

export interface RingRef {
  ringDid: string;
  cid: string;
}

export type Role = "student" | "classRep" | "admin" | "faculty";
export type SessionStatus =
  | "scheduled"
  | "live"
  | "ended"
  | "cancelled"
  | "rescheduled";
export type NoteFormat = "latex" | "plaintext";
export type BrainNodeFormat = "markdown" | "plaintext" | "latex" | "html";
export type Visibility = "world" | "institution" | "course";
export type LabelVal = "endorsed" | "communityVerified" | "archived";
export type NotificationType =
  | "sessionOpened"
  | "sessionCancelled"
  | "sessionRescheduled"
  | "keywordWindowClosing"
  | "noteVoted"
  | "editProposed"
  | "editAccepted"
  | "archiveInitiated"
  | "labelApplied"
  | "brainNodeLinked";
export type ProposalStatus = "pending" | "accepted" | "rejected";
export type Mode = "academic" | "brain" | "all";

export interface LabelSignal {
  val: LabelVal;
  srcDid: string;
  createdAt: string;
}

export interface Notification {
  id: string;
  recipientDid: string;
  reason: NotificationType;
  subjectUri: string;
  read: boolean;
  createdAt: string;
}

// ── Record Types ─────────────────────────────────────────────

export interface Membership {
  institutionDid: string;
  institutionDomain: string;
  role: Role;
  verifiedAt: string;
  verifiedEmail?: string;
}

export interface Course {
  uri: string;
  title: string;
  code: string;
  department: string;
  semester: string;
  visibility: Visibility;
  description?: string;
  createdBy: string;
  classRepDid?: string;
  enrolledCount?: number;
  createdAt: string;
}

export interface Session {
  uri: string;
  courseUri: string;
  scheduledAt: string;
  durationMins: number;
  status: SessionStatus;
  createdBy: string;
  topic?: string;
  slot?: string;
  openedAt?: string;
  closedAt?: string;
  keywordWindowExpiresAt?: string;
  keywordWindowOpen?: boolean;
  rescheduledTo?: string;
  createdAt: string;
}

export interface Keyword {
  sessionUri: string;
  text: string;
  createdAt: string;
}

export interface Note {
  uri: string;
  authorDid: string;
  sessionUri: string;
  format: NoteFormat;
  ringRef: RingRef;
  version: number;
  voteCount: number;
  labels: LabelSignal[];
  summary?: string;
  createdAt: string;
}

export interface Vote {
  subjectUri: string;
  createdAt: string;
}

export interface Label {
  subjectUri: string;
  val: LabelVal;
  neg?: boolean;
  createdAt: string;
}

export interface CollectiveNoteProposal {
  proposalUri: string;
  sessionUri: string;
  proposerDid: string;
  diffRingRef: RingRef;
  status: ProposalStatus;
  summary?: string;
  resolvedAt?: string;
  resolvedBy?: string;
  createdAt: string;
}

export interface BrainNode {
  uri: string;
  authorDid: string;
  title: string;
  format: BrainNodeFormat;
  ringRef: RingRef;
  tags: string[];
  academicRef?: string;
  version: number;
  voteCount: number;
  summary?: string;
  createdAt: string;
}

export interface BrainLink {
  fromUri: string;
  toUri: string;
  label?: string;
  createdAt: string;
}

export interface Archive {
  archiveUri: string;
  courseUri: string;
  semester: string;
  sealedBy: string;
  bundleRef: RingRef;
  internetArchiveUrl?: string;
  sessionCount: number;
  noteCount: number;
  sealedAt: string;
}

// ── API Response Types ───────────────────────────────────────

export interface PaginatedResponse {
  cursor?: string;
}

export interface ListCoursesResponse {
  courses: Course[];
  cursor?: string;
}

export interface ListSessionsResponse {
  sessions: Session[];
  cursor?: string;
}

export interface GetNotesResponse {
  notes: Note[];
  cursor?: string;
}

export interface GetNoteContentResponse {
  format: string;
  content: string;
}

export interface GetNoteHistoryResponse {
  sessionUri: string;
  authorDid: string;
  versions: {
    uri: string;
    ringRef: RingRef;
    version: number;
    parentNoteUri?: string;
    summary?: string;
    createdAt: string;
  }[];
}

export interface CreateNoteResponse {
  ringRef: RingRef;
  noteTemplate: {
    uri: string;
    sessionUri: string;
    format: string;
    ringRef: RingRef;
    version: number;
    summary?: string;
    createdAt: string;
  };
}

export interface GetKeywordHistogramResponse {
  sessionUri: string;
  entries: { text: string; count: number }[];
  totalSubmissions: number;
  windowOpen: boolean;
  windowExpiresAt?: string;
}

export interface AddKeywordResponse {
  keywordUri: string;
  windowExpiresAt?: string;
}

export interface GetCourseFeedResponse {
  feed: {
    eventType: string;
    actorDid: string;
    subjectUri: string;
    sessionUri?: string;
    createdAt: string;
  }[];
  cursor?: string;
}

export interface GetSocialFeedResponse {
  feed: {
    eventType: string;
    actorDid: string;
    subjectUri: string;
    courseUri?: string;
    sessionUri?: string;
    createdAt: string;
  }[];
  cursor?: string;
}

export interface GetBrainFeedResponse {
  nodes: BrainNode[];
  cursor?: string;
}

export interface GetTrendingKeywordsResponse {
  keywords: {
    text: string;
    count: number;
    sessionUri: string;
    courseUri: string;
  }[];
  computedAt: string;
}

export interface GetTrendingBrainTagsResponse {
  tags: { tag: string; count: number; sampleNodeUri: string }[];
  computedAt: string;
}

export interface GetFollowedEnrollmentsResponse {
  enrollments: {
    courseUri: string;
    courseTitle: string;
    institutionDid: string;
    followedCount: number;
  }[];
}

export interface GetGlobalArchiveFeedResponse {
  items: {
    archiveUri: string;
    courseUri: string;
    courseTitle: string;
    semester: string;
    institutionDid: string;
    iaUrl?: string;
    sessionCount: number;
    sealedAt: string;
  }[];
  cursor?: string;
}

export interface SearchCoursesResponse {
  courses: {
    uri: string;
    title: string;
    code: string;
    department: string;
    semester: string;
    visibility: string;
    enrolledCount: number;
    institutionDid: string;
  }[];
  hitsTotal: number;
  cursor?: string;
}

export interface SearchNotesResponse {
  notes: Note[];
  hitsTotal: number;
  cursor?: string;
}

export interface SearchBrainNodesResponse {
  nodes: BrainNode[];
  hitsTotal: number;
  cursor?: string;
}

export interface SearchArchiveResponse {
  results: Archive[];
  hitsTotal: number;
  cursor?: string;
}

export interface GetNodeGraphResponse {
  nodes: {
    uri: string;
    title: string;
    authorDid: string;
    tags: string[];
    summary?: string;
  }[];
  edges: { fromUri: string; toUri: string; label?: string }[];
}

export interface GetBacklinksResponse {
  backlinks: {
    fromUri: string;
    fromTitle: string;
    fromAuthorDid: string;
    label?: string;
    createdAt: string;
  }[];
  cursor?: string;
}

export interface GetNeighboursResponse {
  neighbours: {
    uri: string;
    title: string;
    authorDid: string;
    tags: string[];
    distance: number;
    summary?: string;
  }[];
}

export interface GetNodeContentResponse {
  format: string;
  content: string;
}

export interface CreateNodeResponse {
  ringRef: RingRef;
  nodeTemplate: {
    uri: string;
    title: string;
    format: string;
    ringRef: RingRef;
    tags: string[];
    academicRef?: string;
    version: number;
    summary?: string;
    createdAt: string;
  };
}

export interface CreateLinkResponse {
  linkUri: string;
  createdAt: string;
}

export interface GetNotificationsResponse {
  notifications: Notification[];
  unreadCount: number;
  cursor?: string;
}

export interface GetMembershipsResponse {
  memberships: Membership[];
}

export interface GetRoleResponse {
  role: Role | null;
}

export interface RegisterVoteResponse {
  subjectUri: string;
  totalVotes: number;
}

export interface GetCollectiveNoteResponse {
  sessionUri: string;
  ringRef: RingRef;
  contributorDids: string[];
  version: number;
  updatedAt: string;
}

export interface ListEditProposalsResponse {
  proposals: CollectiveNoteProposal[];
  cursor?: string;
}

export interface GetEnrollmentsResponse {
  courseUri: string;
  dids: string[];
  total: number;
  cursor?: string;
}

export interface ListBansResponse {
  bans: {
    targetDid: string;
    reason?: string;
    permanent: boolean;
    expiresAt?: string;
    bannedAt: string;
  }[];
}

export interface IsBannedResponse {
  banned: boolean;
  expiresAt?: string;
}

export interface InitiateArchiveResponse {
  courseUri: string;
  semester: string;
  initiatedAt: string;
  sessionCount: number;
  noteCount: number;
}

export interface SealArchiveResponse {
  archiveUri: string;
  bundleRef: RingRef;
  sealedAt: string;
  sessionCount: number;
  noteCount: number;
}

export interface Enrollment {
  courseUri: string;
  did: string;
  slot?: string;
  enrolledAt: string;
}

export interface MyEnrollmentsResponse {
  enrollments: (Enrollment & {
    courseTitle: string;
    courseCode: string;
    department: string;
    semester: string;
  })[];
}

export interface ProvisionSessionsResponse {
  courseUri: string;
  slot: string;
  sessionsCreated: number;
  firstSession?: string;
  lastSession?: string;
}

export interface LoadSlotsResponse {
  semester: string;
  slotsLoaded: number;
  occurrencesLoaded: number;
}

export interface LoadCalendarResponse {
  semester: string;
  phasesLoaded: number;
  holidaysLoaded: number;
}

export interface AuditLogResponse {
  entries: {
    action: string;
    actorDid: string;
    targetDid?: string;
    details?: string;
    createdAt: string;
  }[];
}

export interface ListApiKeysResponse {
  keys: {
    id: string;
    name: string;
    prefix: string;
    scopes: string[];
    createdAt: string;
    expiresAt?: string;
    lastUsedAt?: string;
  }[];
}

import { useState, type FormEvent } from "react";
import { useXrpcMutation, useXrpcQuery } from "../hooks/useXrpc";
import { useInitiateArchive, useSealArchive } from "../hooks/useArchive";
import {
  usePromoteRole,
  useDemoteRole,
  useAuditLog,
} from "../hooks/useIdentity";
import { useMemberships, useRole } from "../hooks/useAuth";
import { useAuth } from "../context/AuthContext";
import { LoadingSpinner } from "../components/common/LoadingSpinner";
import { ErrorMessage } from "../components/common/ErrorMessage";
import type { ListBansResponse, Visibility } from "../generated/types";

type AdminTab = "courses" | "moderation" | "archive" | "roles" | "apikeys";

const TAB_LABELS: Record<AdminTab, string> = {
  courses: "Course Management",
  moderation: "Moderation",
  archive: "Archive",
  roles: "Roles",
  apikeys: "API Keys",
};

// ── Sub-components ───────────────────────────────────────────

function CreateCourseForm() {
  const [title, setTitle] = useState("");
  const [code, setCode] = useState("");
  const [department, setDepartment] = useState("");
  const [semester, setSemester] = useState("");
  const [visibility, setVisibility] = useState<Visibility>("institution");
  const [description, setDescription] = useState("");

  const createCourse = useXrpcMutation<
    { courseUri: string },
    {
      title: string;
      code: string;
      department: string;
      semester: string;
      visibility?: Visibility;
      description?: string;
    }
  >("app.changala.ring.createCourse");

  const handleSubmit = (e: FormEvent) => {
    e.preventDefault();
    createCourse.mutate({
      title: title.trim(),
      code: code.trim(),
      department: department.trim(),
      semester: semester.trim(),
      visibility,
      description: description.trim() || undefined,
    });
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <h3 className="text-base font-semibold text-text">Create Course</h3>
      <div className="grid gap-4 sm:grid-cols-2">
        <div>
          <label
            htmlFor="course-title"
            className="block text-sm font-medium text-text-secondary mb-1"
          >
            Title *
          </label>
          <input
            id="course-title"
            type="text"
            required
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            className="w-full px-3 py-2 rounded-lg border border-border bg-surface text-text text-sm focus:outline-none focus:ring-2 focus:ring-academic/40"
            placeholder="Data Structures & Algorithms"
          />
        </div>
        <div>
          <label
            htmlFor="course-code"
            className="block text-sm font-medium text-text-secondary mb-1"
          >
            Code *
          </label>
          <input
            id="course-code"
            type="text"
            required
            value={code}
            onChange={(e) => setCode(e.target.value)}
            className="w-full px-3 py-2 rounded-lg border border-border bg-surface text-text text-sm focus:outline-none focus:ring-2 focus:ring-academic/40"
            placeholder="CS301"
          />
        </div>
        <div>
          <label
            htmlFor="course-dept"
            className="block text-sm font-medium text-text-secondary mb-1"
          >
            Department *
          </label>
          <input
            id="course-dept"
            type="text"
            required
            value={department}
            onChange={(e) => setDepartment(e.target.value)}
            className="w-full px-3 py-2 rounded-lg border border-border bg-surface text-text text-sm focus:outline-none focus:ring-2 focus:ring-academic/40"
            placeholder="Computer Science"
          />
        </div>
        <div>
          <label
            htmlFor="course-semester"
            className="block text-sm font-medium text-text-secondary mb-1"
          >
            Semester *
          </label>
          <input
            id="course-semester"
            type="text"
            required
            value={semester}
            onChange={(e) => setSemester(e.target.value)}
            className="w-full px-3 py-2 rounded-lg border border-border bg-surface text-text text-sm focus:outline-none focus:ring-2 focus:ring-academic/40"
            placeholder="Spring 2026"
          />
        </div>
        <div>
          <label
            htmlFor="course-visibility"
            className="block text-sm font-medium text-text-secondary mb-1"
          >
            Visibility
          </label>
          <select
            id="course-visibility"
            value={visibility}
            onChange={(e) => setVisibility(e.target.value as Visibility)}
            className="w-full px-3 py-2 rounded-lg border border-border bg-surface text-text text-sm focus:outline-none focus:ring-2 focus:ring-academic/40"
          >
            <option value="world">World Public</option>
            <option value="institution">Institution Public</option>
            <option value="course">Course-Enrolled Only</option>
          </select>
        </div>
        <div className="sm:col-span-2">
          <label
            htmlFor="course-desc"
            className="block text-sm font-medium text-text-secondary mb-1"
          >
            Description
          </label>
          <textarea
            id="course-desc"
            rows={2}
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            className="w-full px-3 py-2 rounded-lg border border-border bg-surface text-text text-sm focus:outline-none focus:ring-2 focus:ring-academic/40 resize-y"
            placeholder="Optional course description…"
          />
        </div>
      </div>

      {createCourse.error && (
        <p className="text-sm text-cancelled">
          {createCourse.error instanceof Error
            ? createCourse.error.message
            : "Failed to create course."}
        </p>
      )}
      {createCourse.isSuccess && (
        <p className="text-sm text-academic">Course created successfully.</p>
      )}

      <button
        type="submit"
        disabled={createCourse.isPending}
        className="px-5 py-2 rounded-lg bg-academic text-white text-sm font-medium hover:bg-academic/90 transition disabled:opacity-50 disabled:cursor-not-allowed"
      >
        {createCourse.isPending ? "Creating…" : "Create Course"}
      </button>
    </form>
  );
}

function AssignClassRepForm() {
  const [courseUri, setCourseUri] = useState("");
  const [classRepDid, setClassRepDid] = useState("");

  const assign = useXrpcMutation<
    void,
    { courseUri: string; classRepDid: string }
  >("app.changala.ring.assignClassRep");

  const handleSubmit = (e: FormEvent) => {
    e.preventDefault();
    assign.mutate({
      courseUri: courseUri.trim(),
      classRepDid: classRepDid.trim(),
    });
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <h3 className="text-base font-semibold text-text">Assign Class Rep</h3>
      <div className="grid gap-4 sm:grid-cols-2">
        <div>
          <label
            htmlFor="rep-course"
            className="block text-sm font-medium text-text-secondary mb-1"
          >
            Course URI *
          </label>
          <input
            id="rep-course"
            type="text"
            required
            value={courseUri}
            onChange={(e) => setCourseUri(e.target.value)}
            className="w-full px-3 py-2 rounded-lg border border-border bg-surface text-text text-sm focus:outline-none focus:ring-2 focus:ring-academic/40"
            placeholder="at://did:plc:…/app.changala.course/…"
          />
        </div>
        <div>
          <label
            htmlFor="rep-did"
            className="block text-sm font-medium text-text-secondary mb-1"
          >
            Class Rep DID *
          </label>
          <input
            id="rep-did"
            type="text"
            required
            value={classRepDid}
            onChange={(e) => setClassRepDid(e.target.value)}
            className="w-full px-3 py-2 rounded-lg border border-border bg-surface text-text text-sm focus:outline-none focus:ring-2 focus:ring-academic/40"
            placeholder="did:plc:…"
          />
        </div>
      </div>

      {assign.error && (
        <p className="text-sm text-cancelled">
          {assign.error instanceof Error
            ? assign.error.message
            : "Failed to assign class rep."}
        </p>
      )}
      {assign.isSuccess && (
        <p className="text-sm text-academic">
          Class rep assigned successfully.
        </p>
      )}

      <button
        type="submit"
        disabled={assign.isPending}
        className="px-5 py-2 rounded-lg bg-academic text-white text-sm font-medium hover:bg-academic/90 transition disabled:opacity-50 disabled:cursor-not-allowed"
      >
        {assign.isPending ? "Assigning…" : "Assign"}
      </button>
    </form>
  );
}

function BanSection() {
  const [targetDid, setTargetDid] = useState("");
  const [reason, setReason] = useState("");
  const [ttl, setTtl] = useState("");

  const ban = useXrpcMutation<
    void,
    { targetDid: string; reason?: string; ttlSeconds?: number }
  >("app.changala.ring.banDid");
  const liftBan = useXrpcMutation<void, { targetDid: string }>(
    "app.changala.ring.liftBan",
  );

  const {
    data: bansData,
    isLoading: bansLoading,
    error: bansError,
    refetch: bansRefetch,
  } = useXrpcQuery<ListBansResponse>("app.changala.ring.listBans");

  const bans = bansData?.bans ?? [];

  const handleBan = (e: FormEvent) => {
    e.preventDefault();
    const ttlNum = ttl ? parseInt(ttl, 10) : undefined;
    ban.mutate(
      {
        targetDid: targetDid.trim(),
        reason: reason.trim() || undefined,
        ttlSeconds: ttlNum && ttlNum > 0 ? ttlNum : undefined,
      },
      { onSuccess: () => bansRefetch() },
    );
  };

  const handleLift = (did: string) => {
    liftBan.mutate({ targetDid: did }, { onSuccess: () => bansRefetch() });
  };

  return (
    <div className="space-y-6">
      {/* Ban form */}
      <form onSubmit={handleBan} className="space-y-4">
        <h3 className="text-base font-semibold text-text">Ban DID</h3>
        <div className="grid gap-4 sm:grid-cols-3">
          <div>
            <label
              htmlFor="ban-did"
              className="block text-sm font-medium text-text-secondary mb-1"
            >
              Target DID *
            </label>
            <input
              id="ban-did"
              type="text"
              required
              value={targetDid}
              onChange={(e) => setTargetDid(e.target.value)}
              className="w-full px-3 py-2 rounded-lg border border-border bg-surface text-text text-sm focus:outline-none focus:ring-2 focus:ring-cancelled/40"
              placeholder="did:plc:…"
            />
          </div>
          <div>
            <label
              htmlFor="ban-reason"
              className="block text-sm font-medium text-text-secondary mb-1"
            >
              Reason
            </label>
            <input
              id="ban-reason"
              type="text"
              value={reason}
              onChange={(e) => setReason(e.target.value)}
              className="w-full px-3 py-2 rounded-lg border border-border bg-surface text-text text-sm focus:outline-none focus:ring-2 focus:ring-cancelled/40"
              placeholder="Optional reason"
            />
          </div>
          <div>
            <label
              htmlFor="ban-ttl"
              className="block text-sm font-medium text-text-secondary mb-1"
            >
              TTL (seconds)
            </label>
            <input
              id="ban-ttl"
              type="number"
              min="0"
              value={ttl}
              onChange={(e) => setTtl(e.target.value)}
              className="w-full px-3 py-2 rounded-lg border border-border bg-surface text-text text-sm focus:outline-none focus:ring-2 focus:ring-cancelled/40"
              placeholder="0 = permanent"
            />
          </div>
        </div>

        {ban.error && (
          <p className="text-sm text-cancelled">
            {ban.error instanceof Error
              ? ban.error.message
              : "Failed to ban DID."}
          </p>
        )}
        {ban.isSuccess && (
          <p className="text-sm text-academic">DID banned successfully.</p>
        )}

        <button
          type="submit"
          disabled={ban.isPending}
          className="px-5 py-2 rounded-lg bg-cancelled text-white text-sm font-medium hover:bg-cancelled/90 transition disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {ban.isPending ? "Banning…" : "Ban DID"}
        </button>
      </form>

      {/* Active bans list */}
      <div>
        <h3 className="text-base font-semibold text-text mb-3">Active Bans</h3>
        {bansLoading ? (
          <LoadingSpinner size="sm" />
        ) : bansError ? (
          <ErrorMessage
            title="Failed to load bans"
            message={
              bansError instanceof Error ? bansError.message : "Unknown error"
            }
            retry={() => bansRefetch()}
          />
        ) : bans.length === 0 ? (
          <p className="text-sm text-text-muted py-4">No active bans.</p>
        ) : (
          <div className="space-y-2">
            {bans.map((b) => (
              <div
                key={b.targetDid}
                className="flex items-center justify-between p-3 rounded-lg border border-border bg-surface"
              >
                <div className="min-w-0 flex-1">
                  <p className="text-sm text-text font-mono truncate">
                    {b.targetDid}
                  </p>
                  <div className="flex gap-3 text-xs text-text-muted mt-0.5">
                    {b.reason && <span>Reason: {b.reason}</span>}
                    <span>
                      {b.permanent
                        ? "Permanent"
                        : `Expires ${new Date(b.expiresAt!).toLocaleString()}`}
                    </span>
                    <span>
                      Banned {new Date(b.bannedAt).toLocaleDateString()}
                    </span>
                  </div>
                </div>
                <button
                  onClick={() => handleLift(b.targetDid)}
                  disabled={liftBan.isPending}
                  className="shrink-0 ml-3 px-3 py-1 rounded border border-border text-xs font-medium text-text hover:bg-surface-hover transition disabled:opacity-50"
                >
                  Lift Ban
                </button>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

function ArchiveSection() {
  const [initCourseUri, setInitCourseUri] = useState("");
  const [initSemester, setInitSemester] = useState("");
  const [sealCourseUri, setSealCourseUri] = useState("");
  const [sealSemester, setSealSemester] = useState("");

  const initiate = useInitiateArchive();
  const seal = useSealArchive();

  const handleInitiate = (e: FormEvent) => {
    e.preventDefault();
    initiate.mutate({
      courseUri: initCourseUri.trim(),
      semester: initSemester.trim(),
    });
  };

  const handleSeal = (e: FormEvent) => {
    e.preventDefault();
    seal.mutate({
      courseUri: sealCourseUri.trim(),
      semester: sealSemester.trim(),
    });
  };

  return (
    <div className="space-y-8">
      {/* Initiate archive */}
      <form onSubmit={handleInitiate} className="space-y-4">
        <h3 className="text-base font-semibold text-text">Initiate Archive</h3>
        <div className="grid gap-4 sm:grid-cols-2">
          <div>
            <label
              htmlFor="init-course"
              className="block text-sm font-medium text-text-secondary mb-1"
            >
              Course URI *
            </label>
            <input
              id="init-course"
              type="text"
              required
              value={initCourseUri}
              onChange={(e) => setInitCourseUri(e.target.value)}
              className="w-full px-3 py-2 rounded-lg border border-border bg-surface text-text text-sm focus:outline-none focus:ring-2 focus:ring-academic/40"
              placeholder="at://did:plc:…/app.changala.course/…"
            />
          </div>
          <div>
            <label
              htmlFor="init-semester"
              className="block text-sm font-medium text-text-secondary mb-1"
            >
              Semester *
            </label>
            <input
              id="init-semester"
              type="text"
              required
              value={initSemester}
              onChange={(e) => setInitSemester(e.target.value)}
              className="w-full px-3 py-2 rounded-lg border border-border bg-surface text-text text-sm focus:outline-none focus:ring-2 focus:ring-academic/40"
              placeholder="Spring 2026"
            />
          </div>
        </div>

        {initiate.error && (
          <p className="text-sm text-cancelled">
            {initiate.error instanceof Error
              ? initiate.error.message
              : "Failed to initiate archive."}
          </p>
        )}
        {initiate.isSuccess && (
          <p className="text-sm text-academic">
            Archive initiated — {initiate.data.sessionCount} sessions,{" "}
            {initiate.data.noteCount} notes.
          </p>
        )}

        <button
          type="submit"
          disabled={initiate.isPending}
          className="px-5 py-2 rounded-lg bg-academic text-white text-sm font-medium hover:bg-academic/90 transition disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {initiate.isPending ? "Initiating…" : "Initiate Archive"}
        </button>
      </form>

      {/* Seal archive */}
      <form onSubmit={handleSeal} className="space-y-4">
        <h3 className="text-base font-semibold text-text">Seal Archive</h3>
        <div className="grid gap-4 sm:grid-cols-2">
          <div>
            <label
              htmlFor="seal-course"
              className="block text-sm font-medium text-text-secondary mb-1"
            >
              Course URI *
            </label>
            <input
              id="seal-course"
              type="text"
              required
              value={sealCourseUri}
              onChange={(e) => setSealCourseUri(e.target.value)}
              className="w-full px-3 py-2 rounded-lg border border-border bg-surface text-text text-sm focus:outline-none focus:ring-2 focus:ring-academic/40"
              placeholder="at://did:plc:…/app.changala.course/…"
            />
          </div>
          <div>
            <label
              htmlFor="seal-semester"
              className="block text-sm font-medium text-text-secondary mb-1"
            >
              Semester *
            </label>
            <input
              id="seal-semester"
              type="text"
              required
              value={sealSemester}
              onChange={(e) => setSealSemester(e.target.value)}
              className="w-full px-3 py-2 rounded-lg border border-border bg-surface text-text text-sm focus:outline-none focus:ring-2 focus:ring-academic/40"
              placeholder="Spring 2026"
            />
          </div>
        </div>

        {seal.error && (
          <p className="text-sm text-cancelled">
            {seal.error instanceof Error
              ? seal.error.message
              : "Failed to seal archive."}
          </p>
        )}
        {seal.isSuccess && (
          <p className="text-sm text-academic">
            Archive sealed — {seal.data.sessionCount} sessions,{" "}
            {seal.data.noteCount} notes.
          </p>
        )}

        <button
          type="submit"
          disabled={seal.isPending}
          className="px-5 py-2 rounded-lg bg-academic text-white text-sm font-medium hover:bg-academic/90 transition disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {seal.isPending ? "Sealing…" : "Seal Archive"}
        </button>
      </form>
    </div>
  );
}

// ── Role Management ──────────────────────────────────────────

function RoleManagementSection() {
  const [targetDid, setTargetDid] = useState("");
  const [lookedUpDid, setLookedUpDid] = useState("");

  const promote = usePromoteRole();
  const demote = useDemoteRole();

  const { data: membershipData, isLoading: membershipLoading } =
    useMemberships(lookedUpDid);

  const {
    data: roleData,
    isLoading: roleLoading,
    refetch: roleRefetch,
  } = useRole(lookedUpDid);

  const {
    data: auditData,
    isLoading: auditLoading,
    error: auditError,
    refetch: auditRefetch,
  } = useAuditLog();

  const handleLookup = (e: FormEvent) => {
    e.preventDefault();
    if (!targetDid.trim()) return;
    setLookedUpDid(targetDid.trim());
  };

  const handlePromote = (role: string) => {
    if (!lookedUpDid) return;
    promote.mutate(
      { targetDid: lookedUpDid, role },
      {
        onSuccess: () => {
          roleRefetch();
          auditRefetch();
        },
      },
    );
  };

  const handleDemote = (role: string) => {
    if (!lookedUpDid) return;
    demote.mutate(
      { targetDid: lookedUpDid, role },
      {
        onSuccess: () => {
          roleRefetch();
          auditRefetch();
        },
      },
    );
  };

  const memberships = membershipData?.memberships ?? [];
  const currentRole = roleData?.role ?? null;
  const auditEntries = auditData?.entries ?? [];

  return (
    <div className="space-y-8">
      {/* Lookup user */}
      <div>
        <h3 className="text-base font-semibold text-text mb-4">
          Role Management
        </h3>
        <form onSubmit={handleLookup} className="flex gap-3">
          <input
            type="text"
            value={targetDid}
            onChange={(e) => setTargetDid(e.target.value)}
            placeholder="did:plc:abc123..."
            className="flex-1 rounded-md border border-border bg-surface-alt px-3 py-2 text-sm text-text placeholder:text-text-muted focus:outline-none focus:ring-2 focus:ring-academic/50"
          />
          <button
            type="submit"
            disabled={!targetDid.trim()}
            className="rounded-md bg-academic px-4 py-2 text-sm font-medium text-white hover:bg-academic/90 disabled:opacity-50 transition"
          >
            Look Up
          </button>
        </form>
      </div>

      {/* User details */}
      {lookedUpDid && (
        <div className="rounded-lg border border-border bg-surface p-4 space-y-4">
          <div>
            <p className="text-xs text-text-muted">DID</p>
            <p className="text-sm font-mono text-text break-all">
              {lookedUpDid}
            </p>
          </div>

          {/* Current role */}
          <div>
            <p className="text-xs text-text-muted mb-1">Current Role</p>
            {roleLoading ? (
              <LoadingSpinner size="sm" />
            ) : currentRole ? (
              <span className="inline-block rounded-full bg-academic/10 text-academic px-3 py-0.5 text-sm font-medium">
                {currentRole}
              </span>
            ) : (
              <span className="text-sm text-text-muted">No role assigned</span>
            )}
          </div>

          {/* Memberships */}
          <div>
            <p className="text-xs text-text-muted mb-1">Memberships</p>
            {membershipLoading ? (
              <LoadingSpinner size="sm" />
            ) : memberships.length === 0 ? (
              <span className="text-sm text-text-muted">No memberships</span>
            ) : (
              <div className="flex flex-wrap gap-2">
                {memberships.map((m, i) => (
                  <span
                    key={i}
                    className="inline-block rounded-full bg-live/10 text-live px-3 py-0.5 text-xs font-medium"
                  >
                    {m.institutionDomain}
                  </span>
                ))}
              </div>
            )}
          </div>

          {/* Promote / Demote actions */}
          <div>
            <p className="text-xs text-text-muted mb-2">Actions</p>
            <div className="flex flex-wrap gap-2">
              <button
                onClick={() => handlePromote("classRep")}
                disabled={promote.isPending || currentRole === "classRep"}
                className="rounded-md border border-academic bg-academic/10 px-3 py-1.5 text-xs font-medium text-academic hover:bg-academic/20 disabled:opacity-50 transition"
              >
                {promote.isPending ? "..." : "Promote to Class Rep"}
              </button>
              <button
                onClick={() => handlePromote("admin")}
                disabled={promote.isPending || currentRole === "admin"}
                className="rounded-md border border-academic bg-academic/10 px-3 py-1.5 text-xs font-medium text-academic hover:bg-academic/20 disabled:opacity-50 transition"
              >
                {promote.isPending ? "..." : "Promote to Admin"}
              </button>
              <button
                onClick={() => handleDemote("student")}
                disabled={
                  demote.isPending || currentRole === "student" || !currentRole
                }
                className="rounded-md border border-cancelled bg-cancelled/10 px-3 py-1.5 text-xs font-medium text-cancelled hover:bg-cancelled/20 disabled:opacity-50 transition"
              >
                {demote.isPending ? "..." : "Demote to Student"}
              </button>
            </div>
            {promote.isError && (
              <ErrorMessage
                title="Promotion failed"
                message={promote.error?.message}
              />
            )}
            {demote.isError && (
              <ErrorMessage
                title="Demotion failed"
                message={demote.error?.message}
              />
            )}
            {promote.isSuccess && (
              <p className="mt-2 text-xs text-live">
                Role updated successfully.
              </p>
            )}
            {demote.isSuccess && (
              <p className="mt-2 text-xs text-live">
                Role updated successfully.
              </p>
            )}
          </div>
        </div>
      )}

      {/* Audit Log */}
      <div>
        <h3 className="text-base font-semibold text-text mb-4">Audit Log</h3>
        {auditLoading ? (
          <LoadingSpinner size="md" />
        ) : auditError ? (
          <ErrorMessage
            title="Failed to load audit log"
            message={
              auditError instanceof Error ? auditError.message : "Unknown error"
            }
            retry={() => auditRefetch()}
          />
        ) : auditEntries.length === 0 ? (
          <div className="text-center py-8 rounded-lg border border-border bg-surface">
            <p className="text-sm text-text-muted">No audit entries yet.</p>
          </div>
        ) : (
          <div className="space-y-2">
            {auditEntries.map((entry, i) => (
              <div
                key={i}
                className="flex items-start gap-3 p-3 rounded-lg border border-border bg-surface"
              >
                <div className="flex-1 min-w-0">
                  <p className="text-sm text-text font-medium">
                    {entry.action}
                  </p>
                  <p className="text-xs text-text-muted mt-0.5 font-mono truncate">
                    by {entry.actorDid}
                  </p>
                  {entry.targetDid && (
                    <p className="text-xs text-text-muted font-mono truncate">
                      target: {entry.targetDid}
                    </p>
                  )}
                </div>
                <span className="shrink-0 text-xs text-text-muted">
                  {new Date(entry.createdAt).toLocaleString()}
                </span>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

// ── API Keys Section ─────────────────────────────────────────────

function ApiKeysSection() {
  const [name, setName] = useState("");
  const [scopes, setScopes] = useState("admin:*");
  const [expiryDays, setExpiryDays] = useState("");
  const [newKey, setNewKey] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);

  const createKey = useXrpcMutation<
    {
      key: string;
      prefix: string;
      name: string;
      scopes: string[];
      expiresAt: string | null;
    },
    { name: string; scopes: string[]; expires_in_days?: number }
  >("app.changala.ring.createApiKey");

  const revokeKey = useXrpcMutation<
    { revoked: boolean; prefix: string },
    { prefix: string }
  >("app.changala.ring.revokeApiKey");

  const {
    data: keysData,
    isLoading: keysLoading,
    refetch: refetchKeys,
  } = useXrpcQuery<{
    keys: {
      prefix: string;
      did: string;
      name: string;
      scopes: string[];
      expiresAt: string | null;
      createdAt: string;
      lastUsedAt: string | null;
    }[];
  }>("app.changala.ring.listApiKeys");

  const handleCreate = (e: FormEvent) => {
    e.preventDefault();
    if (!name.trim()) return;
    const scopeList = scopes
      .split(",")
      .map((s) => s.trim())
      .filter(Boolean);
    const body: { name: string; scopes: string[]; expires_in_days?: number } = {
      name: name.trim(),
      scopes: scopeList,
    };
    if (expiryDays.trim()) {
      body.expires_in_days = parseInt(expiryDays, 10);
    }
    createKey.mutate(body, {
      onSuccess: (data) => {
        setNewKey(data.key);
        setCopied(false);
        setName("");
        refetchKeys();
      },
    });
  };

  const handleCopy = () => {
    if (newKey) {
      navigator.clipboard.writeText(newKey);
      setCopied(true);
      setTimeout(() => setCopied(false), 3000);
    }
  };

  const handleRevoke = (prefix: string) => {
    if (!confirm(`Revoke API key ${prefix}...?`)) return;
    revokeKey.mutate({ prefix }, { onSuccess: () => refetchKeys() });
  };

  const keys = keysData?.keys ?? [];

  return (
    <div className="space-y-6">
      {/* Create new key */}
      <div className="rounded-lg border border-border bg-surface p-6">
        <h2 className="text-lg font-semibold text-text mb-4">Create API Key</h2>
        <form onSubmit={handleCreate} className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-text mb-1">
              Key Name
            </label>
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="MCP Admin Key"
              className="w-full rounded-md border border-border bg-surface-alt px-3 py-2 text-sm text-text"
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-text mb-1">
              Scopes (comma-separated)
            </label>
            <input
              type="text"
              value={scopes}
              onChange={(e) => setScopes(e.target.value)}
              placeholder="admin:*"
              className="w-full rounded-md border border-border bg-surface-alt px-3 py-2 text-sm text-text"
            />
            <p className="text-xs text-text-muted mt-1">
              Available: admin:*, admin:courses, admin:sessions,
              admin:moderation, admin:roles, write:notes, write:brain, read:*
            </p>
          </div>
          <div>
            <label className="block text-sm font-medium text-text mb-1">
              Expires in (days, optional)
            </label>
            <input
              type="number"
              value={expiryDays}
              onChange={(e) => setExpiryDays(e.target.value)}
              placeholder="Leave empty for no expiry"
              className="w-full rounded-md border border-border bg-surface-alt px-3 py-2 text-sm text-text"
            />
          </div>
          {createKey.error && (
            <ErrorMessage message={createKey.error.message} />
          )}
          <button
            type="submit"
            disabled={!name.trim() || createKey.isPending}
            className="rounded-md bg-academic px-4 py-2 text-sm font-medium text-white hover:bg-academic/90 disabled:opacity-50 transition"
          >
            {createKey.isPending ? "Creating..." : "Create Key"}
          </button>
        </form>
      </div>

      {/* Newly created key (show once) */}
      {newKey && (
        <div className="rounded-lg border-2 border-live bg-live/5 p-6">
          <h3 className="text-sm font-semibold text-live mb-2">
            ⚠ Copy this key now — it won't be shown again
          </h3>
          <div className="flex items-center gap-2">
            <code className="flex-1 bg-surface rounded px-3 py-2 text-sm font-mono text-text break-all select-all">
              {newKey}
            </code>
            <button
              onClick={handleCopy}
              className="shrink-0 rounded-md bg-live px-3 py-2 text-sm font-medium text-white hover:bg-live/90 transition"
            >
              {copied ? "✓ Copied" : "Copy"}
            </button>
          </div>
          <button
            onClick={() => setNewKey(null)}
            className="mt-3 text-xs text-text-muted hover:text-text"
          >
            Dismiss
          </button>
        </div>
      )}

      {/* Existing keys */}
      <div className="rounded-lg border border-border bg-surface p-6">
        <h2 className="text-lg font-semibold text-text mb-4">Active Keys</h2>
        {keysLoading ? (
          <LoadingSpinner size="sm" />
        ) : keys.length === 0 ? (
          <p className="text-sm text-text-muted">No API keys yet.</p>
        ) : (
          <div className="space-y-3">
            {keys.map((k) => (
              <div
                key={k.prefix}
                className="flex items-center justify-between p-3 rounded border border-border bg-surface-alt"
              >
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2">
                    <code className="text-sm font-mono text-text">
                      {k.prefix}...
                    </code>
                    <span className="text-sm font-medium text-text">
                      {k.name}
                    </span>
                  </div>
                  <div className="flex items-center gap-3 mt-1 text-xs text-text-muted">
                    <span>
                      Scopes:{" "}
                      {Array.isArray(k.scopes)
                        ? k.scopes.join(", ")
                        : String(k.scopes)}
                    </span>
                    <span>
                      Created {new Date(k.createdAt).toLocaleDateString()}
                    </span>
                    {k.lastUsedAt && (
                      <span>
                        Last used {new Date(k.lastUsedAt).toLocaleDateString()}
                      </span>
                    )}
                    {k.expiresAt && (
                      <span className="text-rescheduled">
                        Expires {new Date(k.expiresAt).toLocaleDateString()}
                      </span>
                    )}
                  </div>
                </div>
                <button
                  onClick={() => handleRevoke(k.prefix)}
                  className="shrink-0 text-xs px-3 py-1 rounded bg-cancelled/10 text-cancelled hover:bg-cancelled/20 transition"
                >
                  Revoke
                </button>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

// ── Main AdminPanel ──────────────────────────────────────────────

export default function AdminPanel() {
  const { isAuthenticated, did } = useAuth();
  const { data: roleData, isLoading: roleLoading } = useRole(did || "");
  const [activeTab, setActiveTab] = useState<AdminTab>("courses");

  if (!isAuthenticated) {
    return (
      <div className="max-w-3xl mx-auto px-4 py-16 text-center">
        <h1 className="text-2xl font-bold text-text mb-3">Admin Panel</h1>
        <p className="text-text-secondary">
          You must be signed in to access this panel.
        </p>
      </div>
    );
  }

  if (roleLoading) {
    return (
      <div className="max-w-3xl mx-auto px-4 py-16">
        <LoadingSpinner />
      </div>
    );
  }

  if (roleData?.role !== "admin") {
    return (
      <div className="max-w-3xl mx-auto px-4 py-16 text-center">
        <h1 className="text-2xl font-bold text-text mb-3">Access Denied</h1>
        <p className="text-text-secondary">
          This panel is restricted to administrators.
        </p>
      </div>
    );
  }

  return (
    <div className="max-w-4xl mx-auto px-4 py-8">
      <h1 className="text-2xl font-bold text-text mb-6">Admin Panel</h1>

      {/* Tabs */}
      <div className="flex gap-1 border-b border-border mb-8">
        {(Object.keys(TAB_LABELS) as AdminTab[]).map((tab) => (
          <button
            key={tab}
            onClick={() => setActiveTab(tab)}
            className={`px-4 py-2 text-sm font-medium transition border-b-2 -mb-px ${
              activeTab === tab
                ? "border-academic text-academic"
                : "border-transparent text-text-muted hover:text-text hover:border-border"
            }`}
          >
            {TAB_LABELS[tab]}
          </button>
        ))}
      </div>

      {/* Tab content */}
      {activeTab === "courses" && (
        <div className="space-y-10">
          <CreateCourseForm />
          <hr className="border-border" />
          <AssignClassRepForm />
        </div>
      )}

      {activeTab === "moderation" && <BanSection />}

      {activeTab === "archive" && <ArchiveSection />}

      {activeTab === "roles" && <RoleManagementSection />}

      {activeTab === "apikeys" && <ApiKeysSection />}
    </div>
  );
}

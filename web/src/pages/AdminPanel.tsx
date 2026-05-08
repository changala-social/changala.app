import { useState, type FormEvent } from "react";
import { useXrpcMutation, useXrpcQuery } from "../hooks/useXrpc";
import { useInitiateArchive, useSealArchive } from "../hooks/useArchive";
import { useAuth } from "../context/AuthContext";
import { LoadingSpinner } from "../components/common/LoadingSpinner";
import { ErrorMessage } from "../components/common/ErrorMessage";
import type { ListBansResponse, Visibility } from "../generated/types";

type AdminTab = "courses" | "moderation" | "archive";

const TAB_LABELS: Record<AdminTab, string> = {
  courses: "Course Management",
  moderation: "Moderation",
  archive: "Archive",
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

// ── Main AdminPanel ──────────────────────────────────────────

export default function AdminPanel() {
  const { isAuthenticated } = useAuth();
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
    </div>
  );
}

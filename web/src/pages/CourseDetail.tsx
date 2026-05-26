import { useState } from "react";
import { useParams } from "react-router-dom";
import { useCourse, useEnrollStudent } from "../hooks/useCourses";
import { useSessions } from "../hooks/useSessions";
import { useAuth } from "../context/AuthContext";
import { SessionCard } from "../components/academic/SessionCard";
import { LoadingSpinner } from "../components/common/LoadingSpinner";
import { ErrorMessage } from "../components/common/ErrorMessage";
import type { Session } from "../generated/types";

export default function CourseDetail() {
  const { uri: rawUri } = useParams<{ uri: string }>();
  const uri = rawUri ? decodeURIComponent(rawUri) : "";

  const { isAuthenticated, did } = useAuth();
  const {
    data: course,
    isLoading: courseLoading,
    error: courseError,
  } = useCourse(uri);

  const [sessionCursor, setSessionCursor] = useState<string | undefined>(
    undefined,
  );
  const [accumulatedSessions, setAccumulatedSessions] = useState<Session[]>([]);
  const { data: sessionsData, isLoading: sessionsLoading } = useSessions(
    uri,
    sessionCursor,
  );

  const [enrollSlot, setEnrollSlot] = useState("");
  const enrollMutation = useEnrollStudent();

  const allSessions =
    sessionCursor && accumulatedSessions.length > 0
      ? [...accumulatedSessions, ...(sessionsData?.sessions ?? [])]
      : (sessionsData?.sessions ?? []);

  const handleLoadMoreSessions = () => {
    if (sessionsData?.cursor) {
      setAccumulatedSessions(allSessions);
      setSessionCursor(sessionsData.cursor);
    }
  };

  const handleEnroll = () => {
    if (!did) return;
    enrollMutation.mutate({
      courseUri: uri,
      ...(enrollSlot ? { slot: enrollSlot } : {}),
    });
  };

  // Loading state
  if (courseLoading) {
    return (
      <div className="max-w-3xl mx-auto px-4 py-16">
        <LoadingSpinner size="lg" />
      </div>
    );
  }

  // Error state
  if (courseError || !course) {
    return (
      <div className="max-w-3xl mx-auto px-4 py-16">
        <ErrorMessage
          title="Course not found"
          message={
            courseError instanceof Error
              ? courseError.message
              : "Could not load this course."
          }
        />
      </div>
    );
  }

  return (
    <div className="max-w-3xl mx-auto px-4 py-8">
      {/* Course Header */}
      <header className="mb-8">
        <div className="flex items-start justify-between gap-4">
          <div className="min-w-0">
            <h1 className="text-2xl font-bold text-text">{course.title}</h1>
            <p className="mt-1 text-text-secondary">
              {course.code} &middot; {course.department}
            </p>
          </div>
          <span className="shrink-0 text-sm px-3 py-1 rounded-full bg-surface-alt text-text-muted">
            {course.semester}
          </span>
        </div>

        {course.description && (
          <p className="mt-4 text-text-secondary leading-relaxed">
            {course.description}
          </p>
        )}

        <div className="flex flex-wrap items-center gap-4 mt-5 text-sm text-text-muted">
          {course.enrolledCount !== undefined && (
            <span>{course.enrolledCount} enrolled</span>
          )}
          {course.classRepDid && (
            <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full bg-academic/10 text-academic text-xs font-medium">
              Class Rep assigned
            </span>
          )}
        </div>

        {/* Enroll button */}
        {isAuthenticated && (
          <div className="mt-5 flex items-center gap-3">
            <input
              type="text"
              value={enrollSlot}
              onChange={(e) => setEnrollSlot(e.target.value)}
              placeholder="Slot (e.g. B1, optional)"
              className="px-3 py-2 rounded-lg border border-border bg-surface text-text text-sm w-36 focus:outline-none focus:ring-2 focus:ring-academic/40 placeholder:text-text-muted"
            />
            <button
              onClick={handleEnroll}
              disabled={enrollMutation.isPending}
              className="px-5 py-2 rounded-lg bg-academic text-white text-sm font-medium hover:opacity-90 transition disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {enrollMutation.isPending
                ? "Enrolling…"
                : enrollMutation.isSuccess
                  ? "Enrolled ✓"
                  : "Enroll"}
            </button>
          </div>
        )}
        {enrollMutation.isError && (
          <p className="mt-2 text-sm text-cancelled">
            {enrollMutation.error instanceof Error
              ? enrollMutation.error.message
              : "Failed to enroll."}
          </p>
        )}
      </header>

      {/* Sessions */}
      <section>
        <h2 className="text-lg font-semibold text-text mb-4">Sessions</h2>

        {sessionsLoading && accumulatedSessions.length === 0 ? (
          <LoadingSpinner size="md" />
        ) : allSessions.length === 0 ? (
          <p className="text-text-muted text-sm py-6 text-center">
            No sessions yet.
          </p>
        ) : (
          <div className="space-y-3">
            {allSessions.map((session) => (
              <SessionCard key={session.uri} session={session} />
            ))}
          </div>
        )}

        {sessionsData?.cursor && (
          <div className="flex justify-center mt-6">
            <button
              onClick={handleLoadMoreSessions}
              disabled={sessionsLoading}
              className="px-5 py-2 rounded-lg border border-border bg-surface text-text text-sm font-medium hover:bg-surface-hover transition disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {sessionsLoading ? "Loading…" : "Load more sessions"}
            </button>
          </div>
        )}
      </section>
    </div>
  );
}

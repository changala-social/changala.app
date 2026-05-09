import { Link } from "react-router-dom";
import { useAuth } from "../context/AuthContext";
import {
  useNotifications,
  useMarkNotificationRead,
  useMarkAllRead,
} from "../hooks/useNotifications";
import { useFollowedEnrollments, useMemberships } from "../hooks/useAuth";
import { EmailVerification } from "../components/common/EmailVerification";
import { LoadingSpinner } from "../components/common/LoadingSpinner";
import { ErrorMessage } from "../components/common/ErrorMessage";
import type { Notification, NotificationType } from "../generated/types";

const REASON_LABELS: Record<NotificationType, string> = {
  sessionOpened: "Session opened",
  sessionCancelled: "Session cancelled",
  sessionRescheduled: "Session rescheduled",
  keywordWindowClosing: "Keyword window closing",
  noteVoted: "Vote on your note",
  editProposed: "Edit proposed",
  editAccepted: "Edit accepted",
  archiveInitiated: "Archive initiated",
  labelApplied: "Label applied",
  brainNodeLinked: "Brain node linked",
};

const HIGH_PRIORITY: Set<NotificationType> = new Set([
  "sessionCancelled",
  "sessionRescheduled",
]);

export default function Dashboard() {
  const { isAuthenticated, did } = useAuth();

  const { data: membershipData } = useMemberships(did || "");
  const hasMembership = (membershipData?.memberships?.length ?? 0) > 0;

  const {
    data: notifData,
    isLoading: notifLoading,
    error: notifError,
    refetch: notifRefetch,
  } = useNotifications();

  const {
    data: enrollData,
    isLoading: enrollLoading,
    error: enrollError,
  } = useFollowedEnrollments();

  const markRead = useMarkNotificationRead();
  const markAllRead = useMarkAllRead();

  const notifications = notifData?.notifications ?? [];
  const unreadCount = notifData?.unreadCount ?? 0;
  const enrollments = enrollData?.enrollments ?? [];

  const handleMarkRead = (notificationId: string) => {
    markRead.mutate({ notificationId });
  };

  const handleMarkAllRead = () => {
    markAllRead.mutate(undefined);
  };

  if (!isAuthenticated) {
    return (
      <div className="max-w-3xl mx-auto px-4 py-16 text-center">
        <h1 className="text-2xl font-bold text-text mb-3">
          Welcome to Changala
        </h1>
        <p className="text-text-secondary mb-6">
          Sign in to see your dashboard.
        </p>
      </div>
    );
  }

  return (
    <div className="max-w-4xl mx-auto px-4 py-8">
      <h1 className="text-2xl font-bold text-text mb-8">Dashboard</h1>

      {isAuthenticated && !hasMembership && (
        <div className="mb-6">
          <EmailVerification onVerified={() => window.location.reload()} />
        </div>
      )}

      <div className="grid gap-8 lg:grid-cols-3">
        {/* Main column — Notifications */}
        <div className="lg:col-span-2 space-y-6">
          {/* Quick actions */}
          <div className="flex flex-wrap gap-3">
            <Link
              to="/note/new"
              className="px-4 py-2 rounded-lg bg-academic text-white text-sm font-medium hover:bg-academic/90 transition"
            >
              + New Note
            </Link>
            <Link
              to="/brain/new"
              className="px-4 py-2 rounded-lg bg-brain text-white text-sm font-medium hover:bg-brain/90 transition"
            >
              + New Brain Node
            </Link>
          </div>

          {/* Notifications */}
          <div>
            <div className="flex items-center justify-between mb-4">
              <h2 className="text-lg font-semibold text-text">
                Notifications
                {unreadCount > 0 && (
                  <span className="ml-2 text-xs px-2 py-0.5 rounded-full bg-academic/10 text-academic font-medium">
                    {unreadCount} unread
                  </span>
                )}
              </h2>
              {unreadCount > 0 && (
                <button
                  onClick={handleMarkAllRead}
                  disabled={markAllRead.isPending}
                  className="text-xs text-academic hover:underline disabled:opacity-50"
                >
                  Mark all read
                </button>
              )}
            </div>

            {notifLoading ? (
              <LoadingSpinner size="md" />
            ) : notifError ? (
              <ErrorMessage
                title="Failed to load notifications"
                message={
                  notifError instanceof Error
                    ? notifError.message
                    : "Unknown error"
                }
                retry={() => notifRefetch()}
              />
            ) : notifications.length === 0 ? (
              <div className="text-center py-10 rounded-lg border border-border bg-surface">
                <p className="text-text-muted">No notifications yet.</p>
              </div>
            ) : (
              <div className="space-y-2">
                {notifications.map((n: Notification) => (
                  <div
                    key={n.id}
                    className={`flex items-start gap-3 p-3 rounded-lg border transition ${
                      n.read
                        ? "border-border bg-surface"
                        : "border-academic/20 bg-academic/5"
                    }`}
                  >
                    {/* Priority indicator */}
                    {HIGH_PRIORITY.has(n.reason) && (
                      <span className="mt-0.5 w-2 h-2 rounded-full bg-cancelled shrink-0" />
                    )}

                    <div className="flex-1 min-w-0">
                      <p className="text-sm text-text font-medium">
                        {REASON_LABELS[n.reason] ?? n.reason}
                      </p>
                      <p className="text-xs text-text-muted mt-0.5 truncate">
                        {n.subjectUri}
                      </p>
                      <p className="text-xs text-text-muted mt-0.5">
                        {new Date(n.createdAt).toLocaleString()}
                      </p>
                    </div>

                    {!n.read && (
                      <button
                        onClick={() => handleMarkRead(n.id)}
                        disabled={markRead.isPending}
                        className="shrink-0 text-xs text-academic hover:underline disabled:opacity-50"
                      >
                        Mark read
                      </button>
                    )}
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>

        {/* Sidebar */}
        <div className="space-y-6">
          {/* Followed enrollments */}
          <div>
            <h2 className="text-lg font-semibold text-text mb-4">
              People You Follow
            </h2>
            {enrollLoading ? (
              <LoadingSpinner size="sm" />
            ) : enrollError ? (
              <ErrorMessage
                title="Failed to load suggestions"
                message={
                  enrollError instanceof Error
                    ? enrollError.message
                    : "Unknown error"
                }
              />
            ) : enrollments.length === 0 ? (
              <div className="text-center py-8 rounded-lg border border-border bg-surface">
                <p className="text-sm text-text-muted">
                  No enrollment suggestions yet.
                </p>
                <p className="text-xs text-text-muted mt-1">
                  Follow more people to see what they&apos;re studying.
                </p>
              </div>
            ) : (
              <div className="space-y-2">
                {enrollments.map((e) => (
                  <div
                    key={e.courseUri}
                    className="p-3 rounded-lg border border-border bg-surface hover:bg-surface-hover transition"
                  >
                    <p className="text-sm font-medium text-text">
                      {e.courseTitle}
                    </p>
                    <p className="text-xs text-text-muted mt-0.5">
                      {e.followedCount}{" "}
                      {e.followedCount === 1 ? "person" : "people"} you follow
                      enrolled
                    </p>
                  </div>
                ))}
              </div>
            )}
          </div>

          {/* Profile link */}
          {did && (
            <Link
              to={`/profile/${encodeURIComponent(did)}`}
              className="block text-center text-sm text-academic hover:underline"
            >
              View your profile &rarr;
            </Link>
          )}
        </div>
      </div>
    </div>
  );
}

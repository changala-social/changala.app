import { useState } from 'react';
import { useNotifications, useMarkNotificationRead, useMarkAllRead } from '../../hooks/useNotifications';
import { Link } from 'react-router-dom';

export function NotificationBell() {
  const [open, setOpen] = useState(false);
  const { data } = useNotifications();
  const markRead = useMarkNotificationRead();
  const markAll = useMarkAllRead();

  const unread = data?.unreadCount ?? 0;

  return (
    <div className="relative">
      <button
        onClick={() => setOpen(!open)}
        className="relative p-2 rounded-lg hover:bg-surface-hover transition"
      >
        <svg className="w-5 h-5 text-text-secondary" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
          <path strokeLinecap="round" strokeLinejoin="round" d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9" />
        </svg>
        {unread > 0 && (
          <span className="absolute -top-0.5 -right-0.5 w-4 h-4 bg-cancelled text-white text-[10px] font-bold rounded-full flex items-center justify-center">
            {unread > 9 ? '9+' : unread}
          </span>
        )}
      </button>
      {open && (
        <div className="absolute right-0 top-full mt-1 bg-surface border border-border rounded-lg shadow-lg w-80 max-h-96 overflow-y-auto">
          <div className="flex items-center justify-between px-4 py-2 border-b border-border">
            <span className="text-sm font-medium text-text">Notifications</span>
            {unread > 0 && (
              <button
                onClick={() => markAll.mutate(undefined)}
                className="text-xs text-academic hover:underline"
              >
                Mark all read
              </button>
            )}
          </div>
          {data?.notifications?.length ? (
            data.notifications.slice(0, 10).map((n) => (
              <Link
                key={n.id}
                to={`/note/${encodeURIComponent(n.subjectUri)}`}
                onClick={() => {
                  if (!n.read) markRead.mutate({ notificationId: n.id });
                  setOpen(false);
                }}
                className={`block px-4 py-3 border-b border-border last:border-0 hover:bg-surface-hover transition ${!n.read ? 'bg-academic/5' : ''}`}
              >
                <p className="text-sm text-text">{n.reason.replace(/([A-Z])/g, ' $1').trim()}</p>
                <p className="text-xs text-text-muted mt-0.5">{new Date(n.createdAt).toLocaleDateString()}</p>
              </Link>
            ))
          ) : (
            <p className="px-4 py-6 text-sm text-text-muted text-center">No notifications yet</p>
          )}
        </div>
      )}
    </div>
  );
}

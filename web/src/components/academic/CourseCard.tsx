import { Link } from 'react-router-dom';
import type { Course } from '../../generated/types';

interface CourseCardProps {
  course: Course;
}

export function CourseCard({ course }: CourseCardProps) {
  return (
    <Link
      to={`/course/${encodeURIComponent(course.uri)}`}
      className="block p-4 rounded-lg border border-border bg-surface hover:bg-surface-hover transition group"
    >
      <div className="flex items-start justify-between gap-2">
        <div className="flex-1 min-w-0">
          <h3 className="font-semibold text-text group-hover:text-academic transition truncate">
            {course.title}
          </h3>
          <p className="text-sm text-text-secondary mt-0.5">
            {course.code} &middot; {course.department}
          </p>
        </div>
        <span className="text-xs px-2 py-0.5 rounded-full bg-surface-alt text-text-muted shrink-0">
          {course.semester}
        </span>
      </div>
      {course.description && (
        <p className="text-sm text-text-muted mt-2 line-clamp-2">{course.description}</p>
      )}
      <div className="flex items-center gap-3 mt-3 text-xs text-text-muted">
        {course.enrolledCount !== undefined && (
          <span>{course.enrolledCount} enrolled</span>
        )}
        {course.classRepDid && (
          <span className="px-1.5 py-0.5 rounded bg-academic/10 text-academic">Class Rep assigned</span>
        )}
      </div>
    </Link>
  );
}

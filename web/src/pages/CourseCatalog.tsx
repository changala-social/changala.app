import { useState } from 'react';
import { useCourses } from '../hooks/useCourses';
import { CourseCard } from '../components/academic/CourseCard';
import { LoadingSpinner } from '../components/common/LoadingSpinner';
import { ErrorMessage } from '../components/common/ErrorMessage';
import type { Course } from '../generated/types';

export default function CourseCatalog() {
  const [semester, setSemester] = useState('');
  const [department, setDepartment] = useState('');
  const [cursor, setCursor] = useState<string | undefined>(undefined);
  const [accumulated, setAccumulated] = useState<Course[]>([]);

  const { data, isLoading, error } = useCourses(
    semester || undefined,
    department || undefined,
    cursor
  );

  // Merge newly-fetched courses into the accumulated list whenever data changes
  const allCourses = cursor && accumulated.length > 0
    ? [...accumulated, ...(data?.courses ?? [])]
    : data?.courses ?? [];

  const handleLoadMore = () => {
    if (data?.cursor) {
      setAccumulated(allCourses);
      setCursor(data.cursor);
    }
  };

  const handleFilterChange = (
    setter: (v: string) => void,
    value: string
  ) => {
    setter(value);
    setCursor(undefined);
    setAccumulated([]);
  };

  return (
    <div className="max-w-5xl mx-auto px-4 py-8">
      <h1 className="text-2xl font-bold text-text mb-6">Course Catalog</h1>

      {/* Filters */}
      <div className="flex flex-col sm:flex-row gap-3 mb-8">
        <div className="flex-1">
          <label htmlFor="semester-filter" className="block text-sm font-medium text-text-secondary mb-1">
            Semester
          </label>
          <input
            id="semester-filter"
            type="text"
            placeholder="e.g. Spring 2026"
            value={semester}
            onChange={(e) => handleFilterChange(setSemester, e.target.value)}
            className="w-full px-3 py-2 rounded-lg border border-border bg-surface text-text placeholder:text-text-muted text-sm focus:outline-none focus:ring-2 focus:ring-academic/40"
          />
        </div>
        <div className="flex-1">
          <label htmlFor="department-filter" className="block text-sm font-medium text-text-secondary mb-1">
            Department
          </label>
          <input
            id="department-filter"
            type="text"
            placeholder="e.g. Computer Science"
            value={department}
            onChange={(e) => handleFilterChange(setDepartment, e.target.value)}
            className="w-full px-3 py-2 rounded-lg border border-border bg-surface text-text placeholder:text-text-muted text-sm focus:outline-none focus:ring-2 focus:ring-academic/40"
          />
        </div>
      </div>

      {/* Content */}
      {isLoading && accumulated.length === 0 ? (
        <LoadingSpinner size="lg" />
      ) : error ? (
        <ErrorMessage
          title="Failed to load courses"
          message={error instanceof Error ? error.message : 'An unexpected error occurred.'}
          retry={() => {
            setCursor(undefined);
            setAccumulated([]);
          }}
        />
      ) : allCourses.length === 0 ? (
        <div className="text-center py-16">
          <p className="text-text-muted">No courses found.</p>
          {(semester || department) && (
            <p className="text-sm text-text-muted mt-1">Try adjusting your filters.</p>
          )}
        </div>
      ) : (
        <>
          <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
            {allCourses.map((course) => (
              <CourseCard key={course.uri} course={course} />
            ))}
          </div>

          {/* Load more */}
          {data?.cursor && (
            <div className="flex justify-center mt-8">
              <button
                onClick={handleLoadMore}
                disabled={isLoading}
                className="px-5 py-2 rounded-lg border border-border bg-surface text-text text-sm font-medium hover:bg-surface-hover transition disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {isLoading ? 'Loading…' : 'Load more'}
              </button>
            </div>
          )}
        </>
      )}
    </div>
  );
}

import { useState, useEffect, type FormEvent } from "react";
import { useSearchParams } from "react-router-dom";
import { useSearchCourses } from "../hooks/useCourses";
import { useSearchNotes } from "../hooks/useNotes";
import { useSearchBrainNodes } from "../hooks/useBrain";
import { useMode } from "../context/ModeContext";
import { CourseCard } from "../components/academic/CourseCard";
import { NoteCard } from "../components/academic/NoteCard";
import { BrainNodeCard } from "../components/brain/BrainNodeCard";
import { LoadingSpinner } from "../components/common/LoadingSpinner";
import { ErrorMessage } from "../components/common/ErrorMessage";
import type { Course, Note, BrainNode } from "../generated/types";

type SearchTab = "courses" | "notes" | "brain";

const TAB_LABELS: Record<SearchTab, string> = {
  courses: "Courses",
  notes: "Notes",
  brain: "Brain Nodes",
};

function getVisibleTabs(mode: string): SearchTab[] {
  if (mode === "academic") return ["courses", "notes"];
  if (mode === "brain") return ["brain"];
  return ["courses", "notes", "brain"];
}

export default function SearchPage() {
  const [searchParams, setSearchParams] = useSearchParams();
  const { mode } = useMode();
  const queryFromUrl = searchParams.get("q") ?? "";

  const visibleTabs = getVisibleTabs(mode);
  const [activeTab, setActiveTab] = useState<SearchTab>(visibleTabs[0]);
  const [inputValue, setInputValue] = useState(queryFromUrl);
  const [query, setQuery] = useState(queryFromUrl);
  const [cursor, setCursor] = useState<string | undefined>(undefined);

  // Accumulated results for pagination
  const [accCourses, setAccCourses] = useState<Course[]>([]);
  const [accNotes, setAccNotes] = useState<Note[]>([]);
  const [accNodes, setAccNodes] = useState<BrainNode[]>([]);

  // Reset tab when mode changes and current tab is no longer visible
  useEffect(() => {
    if (!visibleTabs.includes(activeTab)) {
      setActiveTab(visibleTabs[0]);
    }
  }, [mode]); // eslint-disable-line react-hooks/exhaustive-deps

  const courseResult = useSearchCourses(
    activeTab === "courses" ? query : "",
    activeTab === "courses" ? cursor : undefined,
  );
  const noteResult = useSearchNotes(
    activeTab === "notes" ? query : "",
    activeTab === "notes" ? cursor : undefined,
  );
  const brainResult = useSearchBrainNodes(
    activeTab === "brain" ? query : "",
    activeTab === "brain" ? cursor : undefined,
  );

  const activeResult =
    activeTab === "courses"
      ? courseResult
      : activeTab === "notes"
        ? noteResult
        : brainResult;

  const isLoading = activeResult.isLoading;
  const error = activeResult.error;

  // Derive display lists
  // Cast search hits to Course — the CourseCard only reads fields present in the
  // search response (uri, title, code, department, semester, enrolledCount, etc.)
  const rawCourses = (courseResult.data?.courses ?? []) as unknown as Course[];
  const courses =
    cursor && accCourses.length > 0
      ? [...accCourses, ...rawCourses]
      : rawCourses;
  const notes =
    cursor && accNotes.length > 0
      ? [...accNotes, ...(noteResult.data?.notes ?? [])]
      : (noteResult.data?.notes ?? []);
  const nodes =
    cursor && accNodes.length > 0
      ? [...accNodes, ...(brainResult.data?.nodes ?? [])]
      : (brainResult.data?.nodes ?? []);

  const hitsTotal =
    activeTab === "courses"
      ? (courseResult.data?.hitsTotal ?? 0)
      : activeTab === "notes"
        ? (noteResult.data?.hitsTotal ?? 0)
        : (brainResult.data?.hitsTotal ?? 0);

  const nextCursor =
    activeTab === "courses"
      ? courseResult.data?.cursor
      : activeTab === "notes"
        ? noteResult.data?.cursor
        : brainResult.data?.cursor;

  const handleSubmit = (e: FormEvent) => {
    e.preventDefault();
    const trimmed = inputValue.trim();
    setQuery(trimmed);
    setCursor(undefined);
    setAccCourses([]);
    setAccNotes([]);
    setAccNodes([]);
    if (trimmed) {
      setSearchParams({ q: trimmed });
    } else {
      setSearchParams({});
    }
  };

  const handleTabChange = (tab: SearchTab) => {
    setActiveTab(tab);
    setCursor(undefined);
    setAccCourses([]);
    setAccNotes([]);
    setAccNodes([]);
  };

  const handleLoadMore = () => {
    if (!nextCursor) return;
    if (activeTab === "courses") setAccCourses(courses as Course[]);
    if (activeTab === "notes") setAccNotes(notes);
    if (activeTab === "brain") setAccNodes(nodes);
    setCursor(nextCursor);
  };

  return (
    <div className="max-w-5xl mx-auto px-4 py-8">
      <h1 className="text-2xl font-bold text-text mb-6">Search</h1>

      {/* Search input */}
      <form onSubmit={handleSubmit} className="mb-6">
        <div className="flex gap-2">
          <input
            type="text"
            value={inputValue}
            onChange={(e) => setInputValue(e.target.value)}
            placeholder="Search courses, notes, brain nodes…"
            className="flex-1 px-4 py-2.5 rounded-lg border border-border bg-surface text-text placeholder:text-text-muted text-sm focus:outline-none focus:ring-2 focus:ring-academic/40"
          />
          <button
            type="submit"
            className="px-5 py-2.5 rounded-lg bg-academic text-white text-sm font-medium hover:bg-academic/90 transition"
          >
            Search
          </button>
        </div>
      </form>

      {/* Tabs */}
      <div className="flex gap-1 border-b border-border mb-6">
        {visibleTabs.map((tab) => (
          <button
            key={tab}
            onClick={() => handleTabChange(tab)}
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

      {/* Results */}
      {!query ? (
        <div className="text-center py-16">
          <p className="text-text-muted">
            Enter a search query to get started.
          </p>
        </div>
      ) : isLoading && !cursor ? (
        <LoadingSpinner size="lg" />
      ) : error ? (
        <ErrorMessage
          title="Search failed"
          message={
            error instanceof Error
              ? error.message
              : "An unexpected error occurred."
          }
          retry={() => activeResult.refetch()}
        />
      ) : (
        <>
          {/* Hit count */}
          <p className="text-sm text-text-secondary mb-4">
            {hitsTotal} {hitsTotal === 1 ? "result" : "results"} for &ldquo;
            {query}&rdquo;
          </p>

          {/* Course results */}
          {activeTab === "courses" &&
            (courses.length === 0 ? (
              <p className="text-center py-12 text-text-muted">
                No courses found.
              </p>
            ) : (
              <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
                {courses.map((course) => (
                  <CourseCard key={course.uri} course={course} />
                ))}
              </div>
            ))}

          {/* Note results */}
          {activeTab === "notes" &&
            (notes.length === 0 ? (
              <p className="text-center py-12 text-text-muted">
                No notes found.
              </p>
            ) : (
              <div className="grid gap-4 sm:grid-cols-2">
                {notes.map((note) => (
                  <NoteCard key={note.uri} note={note} />
                ))}
              </div>
            ))}

          {/* Brain node results */}
          {activeTab === "brain" &&
            (nodes.length === 0 ? (
              <p className="text-center py-12 text-text-muted">
                No brain nodes found.
              </p>
            ) : (
              <div className="grid gap-4 sm:grid-cols-2">
                {nodes.map((node) => (
                  <BrainNodeCard key={node.uri} node={node} />
                ))}
              </div>
            ))}

          {/* Load more */}
          {nextCursor && (
            <div className="flex justify-center mt-8">
              <button
                onClick={handleLoadMore}
                disabled={isLoading}
                className="px-5 py-2 rounded-lg border border-border bg-surface text-text text-sm font-medium hover:bg-surface-hover transition disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {isLoading ? "Loading…" : "Load more"}
              </button>
            </div>
          )}
        </>
      )}
    </div>
  );
}

import { useState } from "react";
import { useParams, useNavigate } from "react-router-dom";
import {
  useNodeContent,
  useCreateNode,
  useVersionNode,
} from "../hooks/useBrain";
import { useXrpcQuery } from "../hooks/useXrpc";
import type { BrainNodeFormat, GetBrainFeedResponse } from "../generated/types";
import { ContentRenderer } from "../components/content/ContentRenderer";
import { LoadingSpinner } from "../components/common/LoadingSpinner";
import { ErrorMessage } from "../components/common/ErrorMessage";

const FORMAT_OPTIONS: { value: BrainNodeFormat; label: string }[] = [
  { value: "markdown", label: "Markdown" },
  { value: "plaintext", label: "Plain Text" },
  { value: "latex", label: "LaTeX" },
  { value: "html", label: "HTML" },
];

interface FormValues {
  title: string;
  format: BrainNodeFormat;
  content: string;
  tags: string;
  academicRef: string;
  summary: string;
}

const EMPTY_FORM: FormValues = {
  title: "",
  format: "markdown",
  content: "",
  tags: "",
  academicRef: "",
  summary: "",
};

export default function BrainNodeEditor() {
  const { uri: rawUri } = useParams<{ uri: string }>();
  const editUri = rawUri ? decodeURIComponent(rawUri) : "";
  const isEditMode = !!editUri;

  // Fetch existing metadata for edit mode
  const { data: metaData, isLoading: metaLoading } =
    useXrpcQuery<GetBrainFeedResponse>(
      "app.changala.globalview.getBrainFeed",
      { nodeUri: editUri },
      { enabled: isEditMode },
    );

  const existingNode = metaData?.nodes?.[0];

  // Fetch existing content for edit mode (need cid + ringDid from metadata)
  const {
    data: existingContent,
    isLoading: contentLoading,
    isError: contentError,
    error: contentErr,
  } = useNodeContent(
    existingNode?.ringRef?.cid || "",
    existingNode?.ringRef?.ringDid || "",
  );

  // Loading state for edit mode
  if (isEditMode && (contentLoading || metaLoading)) {
    return (
      <div className="max-w-4xl mx-auto px-4 py-8">
        <LoadingSpinner />
      </div>
    );
  }

  if (isEditMode && contentError) {
    return (
      <div className="max-w-4xl mx-auto px-4 py-8">
        <ErrorMessage
          title="Failed to load node"
          message={
            contentErr instanceof Error
              ? contentErr.message
              : "Could not load existing content for editing."
          }
        />
      </div>
    );
  }

  // Build initial values from fetched data (guaranteed loaded at this point in edit mode)
  let initialValues = EMPTY_FORM;
  if (isEditMode) {
    const node = existingNode;
    initialValues = {
      title: node?.title ?? "",
      format:
        node?.format ??
        (existingContent?.format as BrainNodeFormat) ??
        "markdown",
      content: existingContent?.content ?? "",
      tags: node?.tags.join(", ") ?? "",
      academicRef: node?.academicRef ?? "",
      summary: node?.summary ?? "",
    };
  }

  return (
    <NodeForm
      key={editUri}
      editUri={editUri}
      isEditMode={isEditMode}
      initialValues={initialValues}
    />
  );
}

function NodeForm({
  editUri,
  isEditMode,
  initialValues,
}: {
  editUri: string;
  isEditMode: boolean;
  initialValues: FormValues;
}) {
  const navigate = useNavigate();

  const [title, setTitle] = useState(initialValues.title);
  const [format, setFormat] = useState<BrainNodeFormat>(initialValues.format);
  const [content, setContent] = useState(initialValues.content);
  const [tagsInput, setTagsInput] = useState(initialValues.tags);
  const [academicRef, setAcademicRef] = useState(initialValues.academicRef);
  const [summary, setSummary] = useState(initialValues.summary);
  const [showPreview, setShowPreview] = useState(false);

  const createNode = useCreateNode();
  const versionNode = useVersionNode();

  const parseTags = (): string[] =>
    tagsInput
      .split(",")
      .map((t) => t.trim())
      .filter((t) => t.length > 0);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();

    const tags = parseTags();

    if (isEditMode) {
      versionNode.mutate(
        {
          nodeUri: editUri,
          title: title || undefined,
          content,
          format,
          tags: tags.length > 0 ? tags : undefined,
          summary: summary || undefined,
        },
        {
          onSuccess: (res) => {
            navigate(`/brain/${encodeURIComponent(res.nodeTemplate.uri)}`);
          },
        },
      );
    } else {
      createNode.mutate(
        {
          title,
          content,
          format,
          tags: tags.length > 0 ? tags : undefined,
          academicRef: academicRef || undefined,
          summary: summary || undefined,
        },
        {
          onSuccess: (res) => {
            navigate(`/brain/${encodeURIComponent(res.nodeTemplate.uri)}`);
          },
        },
      );
    }
  };

  const isSubmitting = createNode.isPending || versionNode.isPending;
  const submitError = createNode.error || versionNode.error;

  return (
    <div className="max-w-4xl mx-auto px-4 py-8">
      <h1 className="text-2xl font-bold text-text mb-6">
        {isEditMode ? "Edit Brain Node" : "New Brain Node"}
      </h1>

      <form onSubmit={handleSubmit} className="space-y-6">
        {/* Title */}
        <div>
          <label
            htmlFor="title"
            className="block text-sm font-medium text-text mb-1"
          >
            Title
          </label>
          <input
            id="title"
            type="text"
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            required={!isEditMode}
            placeholder="What is this node about?"
            className="w-full px-3 py-2 rounded-lg border border-border bg-surface text-text placeholder-text-muted focus:outline-none focus:ring-2 focus:ring-brain/50 focus:border-brain"
          />
        </div>

        {/* Format selector */}
        <div>
          <label
            htmlFor="format"
            className="block text-sm font-medium text-text mb-1"
          >
            Format
          </label>
          <select
            id="format"
            value={format}
            onChange={(e) => setFormat(e.target.value as BrainNodeFormat)}
            className="w-full px-3 py-2 rounded-lg border border-border bg-surface text-text focus:outline-none focus:ring-2 focus:ring-brain/50 focus:border-brain"
          >
            {FORMAT_OPTIONS.map((opt) => (
              <option key={opt.value} value={opt.value}>
                {opt.label}
              </option>
            ))}
          </select>
        </div>

        {/* Content + Preview */}
        <div>
          <div className="flex items-center justify-between mb-1">
            <label
              htmlFor="content"
              className="block text-sm font-medium text-text"
            >
              Content
            </label>
            <button
              type="button"
              onClick={() => setShowPreview((p) => !p)}
              className="text-xs px-2 py-1 rounded border border-border text-text-secondary hover:text-text hover:bg-surface-hover transition"
            >
              {showPreview ? "Hide Preview" : "Show Preview"}
            </button>
          </div>

          <div className={showPreview ? "grid grid-cols-2 gap-4" : ""}>
            <textarea
              id="content"
              value={content}
              onChange={(e) => setContent(e.target.value)}
              required
              rows={16}
              placeholder="Write your thoughts…"
              className="w-full px-3 py-2 rounded-lg border border-border bg-surface text-text font-mono text-sm placeholder-text-muted resize-y focus:outline-none focus:ring-2 focus:ring-brain/50 focus:border-brain"
            />

            {showPreview && (
              <div className="rounded-lg border border-border bg-surface p-4 overflow-auto max-h-104">
                {content ? (
                  <ContentRenderer format={format} content={content} />
                ) : (
                  <p className="text-text-muted text-sm italic">
                    Nothing to preview yet.
                  </p>
                )}
              </div>
            )}
          </div>
        </div>

        {/* Tags */}
        <div>
          <label
            htmlFor="tags"
            className="block text-sm font-medium text-text mb-1"
          >
            Tags
          </label>
          <input
            id="tags"
            type="text"
            value={tagsInput}
            onChange={(e) => setTagsInput(e.target.value)}
            placeholder="physics, book-notes, idea (comma-separated)"
            className="w-full px-3 py-2 rounded-lg border border-border bg-surface text-text placeholder-text-muted focus:outline-none focus:ring-2 focus:ring-brain/50 focus:border-brain"
          />
          {parseTags().length > 0 && (
            <div className="flex flex-wrap gap-1.5 mt-2">
              {parseTags().map((tag) => (
                <span
                  key={tag}
                  className="text-xs px-2 py-0.5 rounded-full bg-brain/10 text-brain"
                >
                  {tag}
                </span>
              ))}
            </div>
          )}
        </div>

        {/* Academic ref — only for new nodes */}
        {!isEditMode && (
          <div>
            <label
              htmlFor="academicRef"
              className="block text-sm font-medium text-text mb-1"
            >
              Academic Reference
              <span className="text-text-muted font-normal ml-1">
                (optional AT URI)
              </span>
            </label>
            <input
              id="academicRef"
              type="text"
              value={academicRef}
              onChange={(e) => setAcademicRef(e.target.value)}
              placeholder="at://did:plc:…/app.changala.session/…"
              className="w-full px-3 py-2 rounded-lg border border-border bg-surface text-text font-mono text-sm placeholder-text-muted focus:outline-none focus:ring-2 focus:ring-brain/50 focus:border-brain"
            />
          </div>
        )}

        {/* Summary */}
        <div>
          <label
            htmlFor="summary"
            className="block text-sm font-medium text-text mb-1"
          >
            Summary
            <span className="text-text-muted font-normal ml-1">
              (optional, shown in graph tooltips)
            </span>
          </label>
          <input
            id="summary"
            type="text"
            value={summary}
            onChange={(e) => setSummary(e.target.value)}
            placeholder="A short description of this node"
            className="w-full px-3 py-2 rounded-lg border border-border bg-surface text-text placeholder-text-muted focus:outline-none focus:ring-2 focus:ring-brain/50 focus:border-brain"
          />
        </div>

        {/* Submit error */}
        {submitError && (
          <ErrorMessage
            title="Failed to save"
            message={
              submitError instanceof Error
                ? submitError.message
                : "An unexpected error occurred."
            }
          />
        )}

        {/* Actions */}
        <div className="flex items-center gap-3 pt-2">
          <button
            type="submit"
            disabled={isSubmitting}
            className="px-6 py-2 rounded-lg bg-brain text-white font-medium text-sm hover:bg-brain/90 disabled:opacity-50 disabled:cursor-not-allowed transition"
          >
            {isSubmitting
              ? "Saving…"
              : isEditMode
                ? "Save New Version"
                : "Create Node"}
          </button>
          <button
            type="button"
            onClick={() => navigate(-1)}
            className="px-6 py-2 rounded-lg border border-border bg-surface text-text-secondary hover:bg-surface-hover hover:text-text text-sm font-medium transition"
          >
            Cancel
          </button>
        </div>
      </form>
    </div>
  );
}

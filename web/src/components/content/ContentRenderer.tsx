import { LatexRenderer } from './LatexRenderer';
import { MarkdownRenderer } from './MarkdownRenderer';

interface ContentRendererProps {
  format: string;
  content: string;
}

export function ContentRenderer({ format, content }: ContentRendererProps) {
  switch (format) {
    case 'latex':
      return <LatexRenderer content={content} />;
    case 'markdown':
      return <MarkdownRenderer content={content} />;
    case 'html':
      return (
        <div
          className="prose max-w-none dark:prose-invert"
          dangerouslySetInnerHTML={{ __html: content }}
        />
      );
    case 'plaintext':
    default:
      return <pre className="whitespace-pre-wrap text-text-secondary font-mono text-sm">{content}</pre>;
  }
}

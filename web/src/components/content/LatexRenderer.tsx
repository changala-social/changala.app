import katex from 'katex';
import 'katex/dist/katex.min.css';

interface LatexRendererProps {
  content: string;
}

export function LatexRenderer({ content }: LatexRendererProps) {
  // Split content by display math ($$...$$) and inline math ($...$)
  const blocks = content.split(/(\$\$[\s\S]*?\$\$|\$[^$\n]+?\$)/);

  const rendered = blocks.map((block, i) => {
    if (block.startsWith('$$') && block.endsWith('$$')) {
      const tex = block.slice(2, -2);
      try {
        const html = katex.renderToString(tex, { throwOnError: false, displayMode: true });
        return <div key={i} className="my-4 overflow-x-auto" dangerouslySetInnerHTML={{ __html: html }} />;
      } catch {
        return <pre key={i} className="text-cancelled text-sm my-2">{tex}</pre>;
      }
    }
    if (block.startsWith('$') && block.endsWith('$') && block.length > 2) {
      const tex = block.slice(1, -1);
      try {
        const html = katex.renderToString(tex, { throwOnError: false, displayMode: false });
        return <span key={i} dangerouslySetInnerHTML={{ __html: html }} />;
      } catch {
        return <code key={i} className="text-cancelled text-sm">{tex}</code>;
      }
    }
    return <span key={i}>{block}</span>;
  });

  return <div className="prose max-w-none dark:prose-invert">{rendered}</div>;
}

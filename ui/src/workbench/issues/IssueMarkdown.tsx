import type { ReactNode } from "react";

type Block =
  | { kind: "paragraph"; lines: string[] }
  | { kind: "heading"; level: number; text: string }
  | { kind: "list"; ordered: boolean; items: string[] }
  | { kind: "code"; language: string | null; code: string };

function parseBlocks(markdown: string): Block[] {
  const lines = markdown.replace(/\r\n/g, "\n").split("\n");
  const blocks: Block[] = [];
  let index = 0;

  while (index < lines.length) {
    const line = lines[index];

    if (line.trim().startsWith("```")) {
      const language = line.trim().slice(3).trim() || null;
      const codeLines: string[] = [];
      index += 1;
      while (index < lines.length && !lines[index].trim().startsWith("```")) {
        codeLines.push(lines[index]);
        index += 1;
      }
      if (index < lines.length) {
        index += 1;
      }
      blocks.push({ kind: "code", language, code: codeLines.join("\n") });
      continue;
    }

    const heading = line.match(/^(#{1,6})\s+(.+)$/);
    if (heading) {
      blocks.push({ kind: "heading", level: heading[1].length, text: heading[2] });
      index += 1;
      continue;
    }

    const unordered = line.match(/^\s*[-*+]\s+(.+)$/);
    const ordered = line.match(/^\s*\d+\.\s+(.+)$/);
    if (unordered || ordered) {
      const items: string[] = [];
      const isOrdered = Boolean(ordered);
      while (index < lines.length) {
        const current = lines[index];
        const bullet = current.match(/^\s*[-*+]\s+(.+)$/);
        const number = current.match(/^\s*\d+\.\s+(.+)$/);
        if (isOrdered && number) {
          items.push(number[1]);
          index += 1;
          continue;
        }
        if (!isOrdered && bullet) {
          items.push(bullet[1]);
          index += 1;
          continue;
        }
        break;
      }
      blocks.push({ kind: "list", ordered: isOrdered, items });
      continue;
    }

    if (line.trim() === "") {
      index += 1;
      continue;
    }

    const paragraph: string[] = [];
    while (index < lines.length && lines[index].trim() !== "") {
      const current = lines[index];
      if (
        current.trim().startsWith("```") ||
        /^(#{1,6})\s+/.test(current) ||
        /^\s*[-*+]\s+/.test(current) ||
        /^\s*\d+\.\s+/.test(current)
      ) {
        break;
      }
      paragraph.push(current);
      index += 1;
    }
    blocks.push({ kind: "paragraph", lines: paragraph });
  }

  return blocks;
}

function renderInline(text: string, keyPrefix: string): ReactNode[] {
  const nodes: ReactNode[] = [];
  const pattern =
    /(`[^`]+`|\*\*[^*]+\*\*|\*[^*]+\*|__[^_]+__|_[^_]+_|\[[^\]]+\]\([^)]+\))/g;
  let lastIndex = 0;
  let match: RegExpExecArray | null;
  let part = 0;

  while ((match = pattern.exec(text)) !== null) {
    if (match.index > lastIndex) {
      nodes.push(text.slice(lastIndex, match.index));
    }
    const token = match[0];
    const key = `${keyPrefix}-${part}`;
    part += 1;

    if (token.startsWith("`") && token.endsWith("`")) {
      nodes.push(
        <code
          key={key}
          className="rounded bg-nest-muted/15 px-1 py-0.5 font-mono text-[0.9em] text-nest-foreground"
        >
          {token.slice(1, -1)}
        </code>,
      );
    } else if (
      (token.startsWith("**") && token.endsWith("**")) ||
      (token.startsWith("__") && token.endsWith("__"))
    ) {
      nodes.push(
        <strong key={key} className="font-semibold text-nest-foreground">
          {renderInline(token.slice(2, -2), `${key}-inner`)}
        </strong>,
      );
    } else if (
      (token.startsWith("*") && token.endsWith("*")) ||
      (token.startsWith("_") && token.endsWith("_"))
    ) {
      nodes.push(
        <em key={key} className="italic">
          {renderInline(token.slice(1, -1), `${key}-inner`)}
        </em>,
      );
    } else {
      const link = token.match(/^\[([^\]]+)\]\(([^)]+)\)$/);
      if (link) {
        nodes.push(
          <a
            key={key}
            href={link[2]}
            className="text-nest-accent hover:underline"
            target="_blank"
            rel="noreferrer"
          >
            {link[1]}
          </a>,
        );
      } else {
        nodes.push(token);
      }
    }
    lastIndex = pattern.lastIndex;
  }

  if (lastIndex < text.length) {
    nodes.push(text.slice(lastIndex));
  }
  return nodes;
}

type IssueMarkdownProps = {
  markdown: string;
  className?: string;
};

/** Lightweight GitHub-style markdown rendering for issue bodies. */
export function IssueMarkdown({ markdown, className }: IssueMarkdownProps) {
  const blocks = parseBlocks(markdown);

  if (!markdown.trim()) {
    return <p className="text-sm italic text-nest-muted">No description provided.</p>;
  }

  return (
    <div className={["prose-nest space-y-3 text-sm leading-relaxed text-nest-foreground", className].filter(Boolean).join(" ")}>
      {blocks.map((block, index) => {
        if (block.kind === "heading") {
          const Tag = (`h${Math.min(block.level, 6)}` as "h1");
          const size =
            block.level === 1
              ? "text-xl font-semibold"
              : block.level === 2
                ? "text-lg font-semibold"
                : "text-base font-semibold";
          return (
            <Tag key={index} className={`${size} text-nest-foreground`}>
              {renderInline(block.text, `h-${index}`)}
            </Tag>
          );
        }
        if (block.kind === "code") {
          return (
            <pre
              key={index}
              className="overflow-x-auto rounded-nest-md border border-nest-border bg-nest-surface p-3 font-mono text-[13px]"
            >
              <code>{block.code}</code>
            </pre>
          );
        }
        if (block.kind === "list") {
          const ListTag = block.ordered ? "ol" : "ul";
          return (
            <ListTag
              key={index}
              className={block.ordered ? "list-decimal space-y-1 pl-6" : "list-disc space-y-1 pl-6"}
            >
              {block.items.map((item, itemIndex) => (
                <li key={itemIndex}>{renderInline(item, `li-${index}-${itemIndex}`)}</li>
              ))}
            </ListTag>
          );
        }
        return (
          <p key={index} className="whitespace-pre-wrap">
            {block.lines.map((line, lineIndex) => (
              <span key={lineIndex}>
                {lineIndex > 0 ? <br /> : null}
                {renderInline(line, `p-${index}-${lineIndex}`)}
              </span>
            ))}
          </p>
        );
      })}
    </div>
  );
}

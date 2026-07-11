import { IssueMarkdown } from "./issues/IssueMarkdown";

type DocViewProps = {
  content: string;
};

/** Read-only Markdown viewer for a Help doc tab. */
export function DocView({ content }: DocViewProps) {
  return (
    <div className="h-full min-h-0 overflow-auto bg-nest-background">
      <div className="mx-auto max-w-4xl px-6 py-6">
        <IssueMarkdown markdown={content} />
      </div>
    </div>
  );
}

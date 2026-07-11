import {
  createContext,
  useCallback,
  useContext,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import { formatIpcError } from "../../lib/agent";
import {
  githubIssueComment,
  githubIssueCreate,
  githubIssueView,
  githubLabelList,
  githubMilestoneList,
  type GitHubLabel,
  type GitHubMilestone,
} from "../../lib/github";
import { useToast } from "../../shell";
import { CommentModal } from "./CommentModal";
import { MetadataListModal } from "./MetadataListModal";
import { NewIssueModal } from "./NewIssueModal";
import { useWorkbench } from "../state";

export type IssuesActions = {
  openNewIssue: () => void;
  openNewComment: (issueNumber?: number) => void;
  openManageLabels: () => void;
  openManageMilestones: () => void;
};

type IssuesContextValue = IssuesActions & {
  refreshToken: number;
};

const IssuesActionsContext = createContext<IssuesContextValue | null>(null);

export function IssuesModalsProvider({ children }: { children: ReactNode }) {
  const toast = useToast();
  const { openIssue } = useWorkbench();
  const [refreshToken, setRefreshToken] = useState(0);
  const [commentOpen, setCommentOpen] = useState(false);
  const [commentIssue, setCommentIssue] = useState<number | undefined>(undefined);
  const [newIssueOpen, setNewIssueOpen] = useState(false);
  const [labelsOpen, setLabelsOpen] = useState(false);
  const [milestonesOpen, setMilestonesOpen] = useState(false);
  const [labels, setLabels] = useState<GitHubLabel[]>([]);
  const [milestones, setMilestones] = useState<GitHubMilestone[]>([]);
  const [metadataLoading, setMetadataLoading] = useState(false);

  const openNewComment = useCallback((issueNumber?: number) => {
    setCommentIssue(issueNumber);
    setCommentOpen(true);
  }, []);

  const openNewIssue = useCallback(() => {
    setNewIssueOpen(true);
  }, []);

  const openManageLabels = useCallback(async () => {
    setLabelsOpen(true);
    setMetadataLoading(true);
    try {
      setLabels(await githubLabelList());
    } catch (error) {
      toast.error(formatIpcError(error));
      setLabelsOpen(false);
    } finally {
      setMetadataLoading(false);
    }
  }, [toast]);

  const openManageMilestones = useCallback(async () => {
    setMilestonesOpen(true);
    setMetadataLoading(true);
    try {
      setMilestones(await githubMilestoneList());
    } catch (error) {
      toast.error(formatIpcError(error));
      setMilestonesOpen(false);
    } finally {
      setMetadataLoading(false);
    }
  }, [toast]);

  const bumpRefresh = useCallback(() => {
    setRefreshToken((token) => token + 1);
  }, []);

  const actions = useMemo<IssuesContextValue>(
    () => ({
      openNewIssue,
      openNewComment,
      openManageLabels,
      openManageMilestones,
      refreshToken,
    }),
    [openManageLabels, openManageMilestones, openNewComment, openNewIssue, refreshToken],
  );

  return (
    <IssuesActionsContext.Provider value={actions}>
      {children}
      <CommentModal
        open={commentOpen}
        issueNumber={commentIssue}
        onCancel={() => setCommentOpen(false)}
        onSubmit={async (number, body) => {
          await githubIssueComment(number, body);
          toast.success(`Comment posted on #${number}`);
          setCommentOpen(false);
          bumpRefresh();
        }}
      />
      <NewIssueModal
        open={newIssueOpen}
        onCancel={() => setNewIssueOpen(false)}
        onSubmit={async (title, body) => {
          const created = await githubIssueCreate(title, body);
          const issue = await githubIssueView(created.number);
          openIssue(issue);
          toast.success(`Created issue #${created.number}`);
          setNewIssueOpen(false);
          bumpRefresh();
        }}
      />
      <MetadataListModal
        open={labelsOpen}
        title="Labels"
        loading={metadataLoading}
        emptyMessage="No labels in this repository."
        items={labels.map((label) => ({
          id: label.name,
          title: label.name,
          subtitle: label.description ?? undefined,
          color: label.color ?? undefined,
        }))}
        onClose={() => setLabelsOpen(false)}
      />
      <MetadataListModal
        open={milestonesOpen}
        title="Milestones"
        loading={metadataLoading}
        emptyMessage="No open milestones."
        items={milestones.map((milestone) => ({
          id: String(milestone.number),
          title: milestone.title,
          subtitle: milestone.dueOn ? `Due ${milestone.dueOn}` : milestone.description ?? undefined,
        }))}
        onClose={() => setMilestonesOpen(false)}
      />
    </IssuesActionsContext.Provider>
  );
}

export function useIssuesActions(): IssuesActions {
  const value = useContext(IssuesActionsContext);
  if (!value) {
    throw new Error("useIssuesActions must be used within IssuesModalsProvider");
  }
  return value;
}

export function useIssuesRefreshToken(): number {
  const value = useContext(IssuesActionsContext);
  if (!value) {
    throw new Error("useIssuesRefreshToken must be used within IssuesModalsProvider");
  }
  return value.refreshToken;
}

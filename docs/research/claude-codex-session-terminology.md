# Claude Code and Codex session terminology

This is a glossary of how Claude Code and OpenAI Codex name their own session concepts. Its purpose is to let adhd-ranch reuse each tool's terms when it starts tracking sessions from them. It records findings only and makes no recommendation.

Sources and versions, checked 2026-09-14:

- Claude Code docs at `https://code.claude.com/docs/en/<page>` (the `.md` twin of each page was read). The local install is Claude Code 2.1.270 (the `version` field in `~/.claude/sessions/*.json`).
- Codex docs. `https://developers.openai.com/codex/*` now returns HTTP 308 to `https://learn.chatgpt.com/docs/*`, and the Codex `llms.txt` at developers.openai.com lists only learn.chatgpt.com pages. This doc cites the learn.chatgpt.com URLs.
- Codex source in `openai/codex` at commit [`60e3576`](https://github.com/openai/codex/tree/60e35765c3e43e152bf5b382a38a0628efd70842). The local install is `codex-cli 0.154.0`; the local rollout files were written by 0.153.4.
- The local machine was inspected read-only. Only directory layout, file-name patterns, JSON keys, and enum values were recorded, never content. Local-only facts are marked **observed**. Both vendors say their on-disk formats are internal, so an observed format can change in any release.

## Claude Code

### Unit of one run: session (a.k.a. conversation)

- **Session**: "A session is a saved conversation tied to a project directory." [Manage sessions](https://code.claude.com/docs/en/sessions)
- The docs use "session" and "conversation" interchangeably: "Each Claude Code conversation is a session tied to your current directory." [How Claude Code works](https://code.claude.com/docs/en/how-claude-code-works#work-across-branches)
- **Main thread**: the top-level conversation, as opposed to its subagents. Example: "When an agent runs as the main thread with `claude --agent`". [Subagents](https://code.claude.com/docs/en/sub-agents)
- **Session kinds**:
  - **interactive session**: tied to a terminal.
  - **background session** (also "background agent"; `/status` shows `background job · attached` / `background job · unattended`): hosted by the **supervisor**.
  - `claude -p` / **Agent SDK** sessions: non-interactive.
  - Sources: [Agent view](https://code.claude.com/docs/en/agent-view#how-background-sessions-are-hosted), [Commands](https://code.claude.com/docs/en/commands).

### A single exchange: prompt, turn, message

Claude Code has no single term that matches Codex's "turn".

- **Prompt / interaction**: OTel ties everything triggered by one user prompt to `prompt.id` ("UUID v4 identifier linking all events produced while processing a single user prompt"). Its root span is `claude_code.interaction`. Hooks also receive `prompt_id`. [Monitoring](https://code.claude.com/docs/en/monitoring-usage#event-correlation-attributes), [Hooks](https://code.claude.com/docs/en/hooks)
- **Turn (CLI prose)** is used loosely for "Claude working on one prompt":
  - `StopFailure`: "When the turn ends due to an API error".
  - Agent view `working`: "A turn is running".
  - Agent view `done`: "The last turn finished what you asked for".
  - Sources: [Hooks](https://code.claude.com/docs/en/hooks), [Agent view](https://code.claude.com/docs/en/agent-view#read-session-state-from-a-script).
- **Turn (Agent SDK)** has a narrower meaning: "A turn is one round trip inside the loop: Claude produces output that includes tool calls, the SDK executes those tools, and the results feed back to Claude automatically." One prompt can take many turns, and `maxTurns` "counts tool-use turns only". [Agent loop](https://code.claude.com/docs/en/agent-sdk/agent-loop#turns-and-messages)
- **Message**: the SDK stream yields `SystemMessage` (subtypes `init`, `compact_boundary`, `informational`, `worker_shutting_down`), `AssistantMessage`, `UserMessage`, `StreamEvent`, and `ResultMessage`. [Agent loop](https://code.claude.com/docs/en/agent-sdk/agent-loop#message-types)
- **Transcript entry**: "Each line is a JSON object for a message, tool use, or metadata entry. The entry format is internal to Claude Code and changes between versions." [Manage sessions](https://code.claude.com/docs/en/sessions#where-transcripts-are-stored)
- **Agentic loop**: the phases "gather context, take action, and verify results". [How Claude Code works](https://code.claude.com/docs/en/how-claude-code-works#the-agentic-loop)

### Identity, resume, continue, fork

- **Session ID**: a UUID. `--session-id` "must be a valid UUID". [CLI reference](https://code.claude.com/docs/en/cli-reference). Observed transcript file names are UUID v4.
- **Continue**: `claude --continue` / `-c` "Load the most recent conversation in the current directory".
- **Resume**: `claude --resume` / `-r` takes a session ID, a name, or a transcript path, or opens the picker. `/resume` does the same inside a session (alias `/continue`).
- **Fork / branch**: `--fork-session` or `/branch` "copies the history into a new session ID, leaving the original unchanged". [How Claude Code works](https://code.claude.com/docs/en/how-claude-code-works#resume-or-fork-sessions)
- **`/fork`**: copies the conversation "into a new background session and keep working here". [Commands](https://code.claude.com/docs/en/commands)
- **`/clear`**: starts a new conversation. OTel notes that `/clear` "assigns a new `session.id`". [Monitoring](https://code.claude.com/docs/en/monitoring-usage#event-correlation-attributes)
- **Session labels** ([Manage sessions](https://code.claude.com/docs/en/sessions#name-your-sessions)):
  - **name**: set with `--name` / `/rename`.
  - **generated title**: an AI summary of the first prompt.
  - **default display name**: for example `my-app-3f`. It is not a resume handle.
- **Background session short ID**: used by `claude attach|logs|stop|respawn|rm <id>`. The doc example is `7c5dcf5d`, and "each session's ID is its directory name under `~/.claude/jobs/`". [Agent view](https://code.claude.com/docs/en/agent-view#manage-sessions-from-the-shell). Observed: a job directory name equals the first 8 hex characters of that session's UUID. This is unverified as a rule.
- **`SessionStart` hook `source`**: `startup`, `resume`, `clear`, `compact`, `fork`. **`SessionEnd` `reason`**: `clear`, `resume`, `logout`, `prompt_input_exit`, `other`. [Hooks](https://code.claude.com/docs/en/hooks)

### Storage on disk

- **Transcripts**: `~/.claude/projects/<project>/<session-id>.jsonl`, where `<project>` is the working-directory path "with non-alphanumeric characters replaced by `-`". Long names are truncated to 200 characters with a hash appended. [Manage sessions](https://code.claude.com/docs/en/sessions#where-transcripts-are-stored)
- **Session subdirectories**: `projects/<project>/<session>/subagents/` holds subagent transcripts, and `projects/<project>/<session>/tool-results/` holds large tool outputs. Retention is `cleanupPeriodDays`, default 30 days. [.claude directory](https://code.claude.com/docs/en/claude-directory)
- **`~/.claude/sessions/`**: "holds one small file per running session, used to detect concurrent sessions and crashes". [.claude directory](https://code.claude.com/docs/en/claude-directory)
  - Observed file naming: `<pid>.json` plus a `<pid>.<hash>.key` file (the key was not read).
  - Observed keys: `pid`, `sessionId`, `cwd`, `startedAt`, `kind` (`interactive`/`bg`), `entrypoint`, `name`, `status` (`busy`/`idle`/`waiting`), `statusUpdatedAt`, `jobId`, `agent`, `bridgeSessionId`, `messagingSocketPath`, `version`.
- **Background session state** ([Agent view](https://code.claude.com/docs/en/agent-view#where-state-is-stored)):
  - `~/.claude/jobs/<id>/state.json` is "not a stable interface".
  - `~/.claude/daemon/roster.json` lists running background sessions.
  - `~/.claude/daemon.log` is the supervisor log.
- **Other application data**: `history.jsonl` ("Every prompt you've typed, with timestamp and project path"), `tasks/`, `plans/`, `file-history/<session>/`, `session-env/`, `debug/`. `todos/` is legacy and "No longer written". [.claude directory](https://code.claude.com/docs/en/claude-directory)
- **Observed transcript record `type` values**, across all local projects: `user`, `assistant`, `system`, `attachment`, `ai-title`, `custom-title`, `agent-name`, `last-prompt`, `mode`, `permission-mode`, `queue-operation`, `file-history-snapshot`, `file-history-delta`, `pr-link`, `worktree-state`, `bridge-session`, `continued-in`, `relocated`, `cost-state`.
  - Common message keys: `uuid`, `parentUuid`, `sessionId`, `cwd`, `gitBranch`, `isSidechain`, `entrypoint`, `userType`, `version`, `timestamp`, `message`.
  - `system` subtypes seen: `compact_boundary`, `stop_hook_summary`, `api_error`, `local_command`.
  - Content block types: `text`, `thinking`, `tool_use`, `tool_result`.
- **Observed subagent metadata**: `subagents/agent-<agentId>.meta.json` with keys `agentType`, `description`, `spawnDepth`, `toolUseId`, `requestShape`, `requestNonInteractive`.
- **Observed history keys**: `history.jsonl` lines have `display`, `pastedContents`, `project`, `sessionId`, `timestamp`.

### Working directory, project, workspace

- A session is "tied to a project directory". The `/resume` picker shows the current worktree by default; `Ctrl+W` widens to all worktrees and `Ctrl+A` to "every project on this machine". [Manage sessions](https://code.claude.com/docs/en/sessions#use-the-session-picker)
- **Project**: the `<project>` directory under `~/.claude/projects/`. `claude project purge [path]` deletes "all local Claude Code state for a project". [CLI reference](https://code.claude.com/docs/en/cli-reference)
- **Workspace**: the status-line JSON `workspace` object ([Status line](https://code.claude.com/docs/en/statusline#available-data)):
  - `workspace.current_dir`
  - `workspace.project_dir`: "Directory where Claude Code was launched"
  - `workspace.added_dirs`: from `/add-dir`
  - `workspace.git_worktree`
  - `workspace.repo.*`
- **Workspace trust**: listed in [Permissions](https://code.claude.com/docs/en/permissions).
- **`/cd`** moves a session to another directory's project storage. **Worktrees**: background sessions isolate edits under `.claude/worktrees/`. [Agent view](https://code.claude.com/docs/en/agent-view)

### Sub-agents

- **Subagent**: "specialized AI assistants that handle specific types of tasks... Each subagent runs in its own context window". [Subagents](https://code.claude.com/docs/en/sub-agents)
  - Spawned by the **`Agent`** tool. [Tools reference](https://code.claude.com/docs/en/tools-reference)
  - Built-in agents include `Explore`, `Plan`, `general-purpose`, and `claude`. Custom definitions live in `.claude/agents/` and `~/.claude/agents/`.
- **Forked subagent** (a "fork"): starts with a copy of the conversation. **Background subagent**: runs apart from the main conversation.
- **Agent ID**: "Each transcript is stored as `agent-{agentId}.jsonl`" under `~/.claude/projects/{project}/{sessionId}/subagents/`. Observed IDs look like `a` followed by 16 hex characters. [Subagents](https://code.claude.com/docs/en/sub-agents#resume-subagents)
- **Agent teams** (experimental): a **team lead** session plus **teammates**. They share a task list and a per-agent **mailbox** at `~/.claude/teams/{team-name}/inboxes/{agent-name}.json`. [Agent teams](https://code.claude.com/docs/en/agent-teams)
- **Background agent / agent view**: `claude agents` opens "one screen for all your background sessions". [Agent view](https://code.claude.com/docs/en/agent-view)
- **Hook fields**: `agent_id` (a subagent identifier) and `agent_type` (for example `Explore`). [Hooks](https://code.claude.com/docs/en/hooks)

### Every use of "task" and "todo"

1. **Task list / Task tools**: `TaskCreate`, `TaskGet`, `TaskList`, `TaskUpdate`.
   - "The task list is Claude's to-do checklist: items Claude created to plan multi-step work, with indicators showing what's pending, in progress, or complete." [Interactive mode](https://code.claude.com/docs/en/interactive-mode#task-list)
   - Stored in `~/.claude/tasks/` ("Task lists written by the task tools, one directory per list"). `CLAUDE_CODE_TASK_LIST_ID` shares a list across sessions. `Ctrl+T` toggles the list.
   - The tools are on by default only for older models. On newer models they are off unless enabled with `CLAUDE_CODE_ENABLE_TODO_TOOLS=1`. [Tools reference](https://code.claude.com/docs/en/tools-reference#task-tool-availability)
   - Observed: `~/.claude/tasks/<uuid>/<n>.json` with keys `id`, `subject`, `description`, `activeForm`, `status`, `blocks`, `blockedBy`.
   - Statuses `pending`, `in_progress`, `completed`. [Agent SDK todo tracking](https://code.claude.com/docs/en/agent-sdk/todo-tracking)
2. **`TodoWrite`**: "Manages the session task checklist", the legacy tool, re-enabled with `CLAUDE_CODE_ENABLE_TASKS=0`. **`todos/`** is a legacy directory that is no longer written. [Tools reference](https://code.claude.com/docs/en/tools-reference), [.claude directory](https://code.claude.com/docs/en/claude-directory)
3. **`TaskCreated` / `TaskCompleted` hooks**: "When a task is being created via `TaskCreate`" / "being marked as completed". [Hooks](https://code.claude.com/docs/en/hooks)
4. **Background tasks**: background Bash commands, monitors, and subagents.
   - A background command "immediately returns a background task ID". [Interactive mode](https://code.claude.com/docs/en/interactive-mode)
   - `/tasks` means "View and manage background work in the current session, including subagents". [Commands](https://code.claude.com/docs/en/commands)
   - `TaskOutput` / `TaskStop` operate on background tasks. `CLAUDE_CODE_DISABLE_BACKGROUND_TASKS` turns them off. The SDK has a `stop_task` request.
5. **Scheduled tasks**: `/loop` and cron tools. "Tasks are session-scoped". [Scheduled tasks](https://code.claude.com/docs/en/scheduled-tasks)
6. **Desktop scheduled tasks**: these "start a new session automatically". Each is stored at `~/.claude/scheduled-tasks/<task-name>/SKILL.md`. [Desktop scheduled tasks](https://code.claude.com/docs/en/desktop-scheduled-tasks)
7. **Agent teams shared task list**: at `~/.claude/tasks/{team-name}/`. "Tasks have three states: pending, in progress, and completed", and teammates **claim** them. [Agent teams](https://code.claude.com/docs/en/agent-teams)
8. **Agent view states**: `Completed` means "The task finished successfully" and `Failed` means "The task ended with an error". [Agent view](https://code.claude.com/docs/en/agent-view#read-session-state)
9. **Cloud work**: "Claude Code on the web runs tasks on Anthropic-managed cloud infrastructure" and "Move tasks between web and terminal". [Claude Code on the web](https://code.claude.com/docs/en/claude-code-on-the-web)
10. **Prose**: "When you give Claude a task...". The SDK `ResultMessage` subtype shows "whether the task succeeded". [How Claude Code works](https://code.claude.com/docs/en/how-claude-code-works), [Agent loop](https://code.claude.com/docs/en/agent-sdk/agent-loop)

**Plan**:

- **plan mode**: the `plan` permission mode, which "explores and proposes a plan without editing".
- `/plan` enters it.
- Plan files are kept in `~/.claude/plans/`.
- `ExitPlanMode` is the tool that leaves it.
- Sources: [How Claude Code works](https://code.claude.com/docs/en/how-claude-code-works), [.claude directory](https://code.claude.com/docs/en/claude-directory).

**Goal**: `/goal` sets "a completion condition" that Claude keeps working toward across turns. [Goal](https://code.claude.com/docs/en/goal)

### Hooks and lifecycle events

Hook events ([Hooks](https://code.claude.com/docs/en/hooks)):

- **Session**: `SessionStart`, `SessionEnd`, `Setup`
- **Prompt and response**: `UserPromptSubmit`, `UserPromptExpansion`, `MessageDisplay`, `Stop`, `StopFailure`
- **Tools and permissions**: `PreToolUse`, `PermissionRequest`, `PermissionDenied`, `PostToolUse`, `PostToolUseFailure`, `PostToolBatch`
- **Attention**: `Notification`
- **Agents and tasks**: `SubagentStart`, `SubagentStop`, `TaskCreated`, `TaskCompleted`, `TeammateIdle`
- **Context and config**: `InstructionsLoaded`, `ConfigChange`, `PreCompact`, `PostCompact`, `PreModelSwitch`, `PostModelSwitch`
- **Directories and files**: `CwdChanged`, `DirectoryAdded`, `FileChanged`, `WorktreeCreate`, `WorktreeRemove`
- **MCP**: `Elicitation`, `ElicitationResult`

Common hook input fields are `session_id`, `prompt_id`, `transcript_path`, `cwd`, `permission_mode`, `hook_event_name`, and, inside a subagent, `agent_id` and `agent_type`.

`Notification` `notification_type` values:

- `permission_prompt`: Claude needs approval "and the prompt has waited about six seconds"
- `idle_prompt`: "Claude finished responding about 60 seconds ago and you haven't typed since"
- `agent_needs_input`
- `agent_completed`
- `auth_success`
- `elicitation_dialog`, `elicitation_url_dialog`, `elicitation_complete`, `elicitation_response`
- `quota_auto_resume_fired`, `quota_auto_resume_stale`, `quota_auto_resume_disabled`

### Official external observation

- **`claude agents --json`**: "the supported way to read session state from outside Claude Code, for example from a status bar". [Agent view](https://code.claude.com/docs/en/agent-view#read-session-state-from-a-script)
  - It lists every live session. `--all` adds completed background sessions.
  - Fields: `cwd`, `kind`, `startedAt`, `id`, `state`, `pid`, `status`, `waitingFor`, `sessionId`, `name`.
- **Hooks**: command hooks receive JSON on stdin (see above).
- **Status line**: a command that receives session JSON on stdin, including `session_id`, `session_name`, `transcript_path`, `workspace.*`, `cost.*`, `context_window.*`, and `rate_limits.*`. [Status line](https://code.claude.com/docs/en/statusline)
- **OpenTelemetry**: [Monitoring](https://code.claude.com/docs/en/monitoring-usage)
  - Enabled with `CLAUDE_CODE_ENABLE_TELEMETRY=1`.
  - Metrics include `claude_code.session.count`, `claude_code.token.usage`, `claude_code.cost.usage`, and `claude_code.active_time.total`.
  - Events include `claude_code.user_prompt`, `claude_code.assistant_response`, `claude_code.tool_result`, `claude_code.tool_decision`, `claude_code.api_request`, `claude_code.permission_mode_changed`, and `claude_code.subagent_completed`.
  - Beta spans use the root `claude_code.interaction`.
  - The standard attribute is `session.id`.
- **Headless / Agent SDK**: `claude -p --output-format json|stream-json` and the SDK message stream. [Manage sessions](https://code.claude.com/docs/en/sessions#access-conversations-from-scripts)
- **Transcript files**: `transcript_path` is exposed, but the entry format is internal. [Manage sessions](https://code.claude.com/docs/en/sessions#where-transcripts-are-stored)

### Session status vocabulary

- **Agent view display states**: `Working`, `Needs input`, `Idle`, `Completed`, `Failed`, `Stopped`. Group labels: `Ready for review`, `Needs input`, `Working`, `Completed`. [Agent view](https://code.claude.com/docs/en/agent-view#read-session-state)
- **`claude agents --json`** ([Agent view](https://code.claude.com/docs/en/agent-view#list-sessions-as-json)):
  - `state` (background sessions): `working`, `blocked`, `done`, `failed`, `stopped`
  - `status` (live process): `busy`, `waiting`, `idle`
  - `waitingFor`: `permission prompt`, `input needed`, `sandbox request`, `worker request`, `dialog open`
- **Rule**: "A session that finished its turn and is waiting for your next instruction reads `done`, not `blocked`."
- **Teammates** go "idle" (the `TeammateIdle` hook). Completed background subagents show as "done" in `/tasks`. [Subagents](https://code.claude.com/docs/en/sub-agents)

### Approvals and permissions

- **Permission modes**: `default` (displayed as **Manual**), `acceptEdits`, `plan`, `auto`, `dontAsk`, `bypassPermissions`. [CLI reference](https://code.claude.com/docs/en/cli-reference), [Hooks](https://code.claude.com/docs/en/hooks)
- **Permission rules**: **allow**, **ask**, **deny**. [Permissions](https://code.claude.com/docs/en/permissions)
- **Other terms**:
  - **permission prompt**
  - **permission decision** (`PermissionRequest`: "When a tool call needs a permission decision")
  - **auto mode classifier**
  - "Allow for this session" grants
  - **sandbox request**
  - Sources: [Hooks](https://code.claude.com/docs/en/hooks), [Manage sessions](https://code.claude.com/docs/en/sessions#branch-a-session).

### Cloud and remote variants

- **Claude Code on the web**: **cloud sessions** run in a **cloud environment**. [Claude Code on the web](https://code.claude.com/docs/en/claude-code-on-the-web)
  - `claude --cloud` creates one; the older `--remote` spelling is a deprecated alias.
  - `--teleport` / `/teleport` pulls a cloud session into the terminal.
  - Sessions can be **archived**.
- **Remote Control**: "Remote Control sessions run directly on your machine", controlled from claude.ai or the mobile app. Started with `claude remote-control`, `--remote-control` / `--rc`, or `/remote-control`. [Remote Control](https://code.claude.com/docs/en/remote-control)
- **Routines**: "A routine is a saved Claude Code configuration... run automatically" on cloud infrastructure. "Each run creates a new session". [Routines](https://code.claude.com/docs/en/routines)
- **Channels**: push events *into* a running session from an MCP server. [Channels](https://code.claude.com/docs/en/channels)

## Codex

### Unit of one run: thread, also session, chat, rollout

Codex uses four overlapping words for one run, depending on the surface:

- **Thread** (app-server protocol): "A conversation between a user and the Codex agent. Threads contain turns." [App Server](https://learn.chatgpt.com/docs/app-server#core-primitives)
- **Session**:
  - `thread.sessionId` "identifies the current live session tree root. Root threads use their own thread id as the session id; forked threads keep the session id of the root they came from." [App Server](https://learn.chatgpt.com/docs/app-server#start-or-resume-a-thread)
  - Source `SessionMeta`: "session_id is equal to the root thread's ID." [protocol.rs](https://github.com/openai/codex/blob/60e35765c3e43e152bf5b382a38a0628efd70842/codex-rs/protocol/src/protocol.rs#L3062)
  - The hook field `session_id` is "Current Codex session id. Subagent hooks use the parent session id." [Hooks](https://learn.chatgpt.com/docs/hooks#common-input-fields)
- **Chat** (user-facing docs):
  - `/new` means "Start a new chat inside the same CLI session".
  - `/resume` means "Resume a saved chat from your session list".
  - `codex fork` means "Fork a previous interactive session into a new chat".
  - In these docs one CLI "session" can contain several "chats". [CLI commands](https://learn.chatgpt.com/docs/developer-commands)
- **Rollout** (storage): the thread's persisted JSONL log. `thread/unarchive` "restores an archived thread rollout back into the active sessions directory". [App Server](https://learn.chatgpt.com/docs/app-server#api-overview)
- **Session source**: how a thread was started.
  - `SessionSource`: `cli`, `vscode`, `exec`, `mcp`, `custom`, `internal`, `subagent`, `unknown`. [protocol.rs](https://github.com/openai/codex/blob/60e35765c3e43e152bf5b382a38a0628efd70842/codex-rs/protocol/src/protocol.rs#L2765)
  - `thread/list` `sourceKinds`: `cli`, `vscode`, `exec`, `appServer`, `subAgent`, `subAgentReview`, `subAgentCompact`, `subAgentThreadSpawn`, `subAgentOther`, `unknown`. [App Server](https://learn.chatgpt.com/docs/app-server#list-threads-with-pagination--filters)

### A single exchange: turn and item

- **Turn**: "A single user request and the agent work that follows. Turns contain items and stream incremental updates." [App Server](https://learn.chatgpt.com/docs/app-server#core-primitives)
- **Steer**: `turn/steer` appends user input to the in-flight turn "without creating a new turn".
- **Item**: "A unit of input or output (user message, agent message, command runs, file change, tool call, and more)". [App Server](https://learn.chatgpt.com/docs/app-server#core-primitives)
- **`ThreadItem` types**: `userMessage`, `agentMessage`, `plan`, `reasoning`, `commandExecution`, `fileChange`, `mcpToolCall`, `dynamicToolCall`, `collabToolCall`, `webSearch`, `imageView`, `enteredReviewMode`, `exitedReviewMode`, `contextCompaction`, `functionCallOutput`. [App Server](https://learn.chatgpt.com/docs/app-server#items)
- **Wire naming quirk**: the core event for a turn starting is serialized as `task_started` and the end as `task_complete`. The source comment reads "v1 wire format uses `task_started`; accept `turn_started` for v2 interop". [protocol.rs](https://github.com/openai/codex/blob/60e35765c3e43e152bf5b382a38a0628efd70842/codex-rs/protocol/src/protocol.rs#L1405). Observed in local rollouts as `event_msg` payload types `task_started`, `task_complete`, and `turn_aborted`.

### Identity, resume, fork

- **Thread ID**: a UUID v7 (`Uuid::now_v7()`). [thread_id.rs](https://github.com/openai/codex/blob/60e35765c3e43e152bf5b382a38a0628efd70842/codex-rs/protocol/src/thread_id.rs#L30)
  - The docs use `thr_123` / `turn_456` as placeholders.
  - Observed: turn IDs are UUIDs and item IDs in rollouts look like `item-<n>`.
- **Resume**:
  - `codex resume` means "Continue an interactive session by ID or resume the most recent chat". `--last` is scoped to the cwd unless `--all` is passed. [CLI commands](https://learn.chatgpt.com/docs/developer-commands#codex-resume)
  - `codex exec resume <SESSION_ID>` or `--last`. [Non-interactive mode](https://learn.chatgpt.com/docs/non-interactive-mode#resume-a-non-interactive-session)
  - `thread/resume`: "reopen an existing thread by id".
- **Fork**:
  - `codex fork`, `/fork`.
  - `thread/fork` means "fork a thread into a new thread id by copying stored history". It returns `forkedFromId`, and `lastTurnId` limits the copy. [App Server](https://learn.chatgpt.com/docs/app-server#api-overview)
- **Other lifecycle commands**:
  - `/new` / `/clear`: new chat.
  - `/side`: "ephemeral side chat".
  - `/rename` and `thread/name/set`.
  - `/archive` and `thread/archive`.
  - `thread/delete`.
  - `thread/rollback` (deprecated).
  - `--ephemeral`: don't persist rollouts.
  - Sources: [CLI commands](https://learn.chatgpt.com/docs/developer-commands), [Non-interactive mode](https://learn.chatgpt.com/docs/non-interactive-mode).
- **`SessionStart` hook `source`**: `startup`, `resume`, `clear`, `compact`. **`SessionEnd` `reason`**: "Currently only `other`". [Hooks](https://learn.chatgpt.com/docs/hooks#matcher-patterns)

### Storage on disk

- **Rollouts**: `~/.codex/sessions/YYYY/MM/DD/rollout-YYYY-MM-DDThh-mm-ss-<uuid>.jsonl`. Archived rollouts are in `archived_sessions`. [rollout/src/list.rs](https://github.com/openai/codex/blob/60e35765c3e43e152bf5b382a38a0628efd70842/codex-rs/rollout/src/list.rs#L436), [rollout/src/lib.rs](https://github.com/openai/codex/blob/60e35765c3e43e152bf5b382a38a0628efd70842/codex-rs/rollout/src/lib.rs#L84)
  - Observed: 269 files under dated directories, and `archived_sessions/` holds flat `rollout-*.jsonl` files.
- **Rollout line types** (`RolloutItem`): `session_meta`, `response_item`, `inter_agent_communication`, `inter_agent_communication_metadata`, `compacted`, `turn_context`, `token_usage_record`, `world_state`, `retained_context`, `security_risk_score`, `event_msg`, `realtime_item`. [history/src/rollout_payload.rs](https://github.com/openai/codex/blob/60e35765c3e43e152bf5b382a38a0628efd70842/codex-rs/history/src/rollout_payload.rs#L24)
  - Observed line keys: `timestamp`, `type`, `payload`, `ordinal`.
- **Observed `session_meta.payload` keys**: `id`, `session_id`, `forked_from_id`, `parent_thread_id`, `cwd`, `originator` (`codex-tui`, `codex_cli_rs`, `Codex Desktop`, `zed`, and others), `source`, `thread_source`, `agent_nickname`, `agent_path`, `cli_version`, `git`, `history_mode`, `model_provider`.
- **Observed `turn_context.payload` keys**: `turn_id`, `cwd`, `workspace_roots`, `approval_policy`, `sandbox_policy`, `permission_profile`, `collaboration_mode` (`mode`: `default`/`plan`), `model`, `effort`.
- **`history.jsonl`**: [Advanced config](https://learn.chatgpt.com/docs/config-file/config-advanced#history-persistence)
  - The docs describe it as "local session transcripts under `CODEX_HOME` (for example, `~/.codex/history.jsonl`)", controlled by `history.persistence` and `history.max_bytes`.
  - Observed: its lines have only `session_id`, `text`, and `ts`, which makes it prompt history. The full transcripts are the rollouts.
- **`session_index.jsonl`**: observed keys `id`, `thread_name`, `updated_at`. Implemented in `codex-rs/rollout/src/session_index.rs`. It is undocumented.
- **SQLite state**: `CODEX_SQLITE_HOME` / `sqlite_home` set where "SQLite-backed state is stored". [Environment variables](https://learn.chatgpt.com/docs/config-file/environment-variables)
  - Observed `state_5.sqlite` tables: `threads` (with `source`, `thread_source`, `agent_nickname`, `agent_role`, `project_id`, `archived`, …), `thread_spawn_edges` (`parent_thread_id`, `child_thread_id`, `status` `open`/`closed`), `projects`, `project_roots`.
  - Observed `thread_history_1.sqlite` tables: `thread_turns` (`status` `completed`/`failed`/`inProgress`/`interrupted`) and `thread_items`.
  - Observed `goals_1.sqlite` table: `thread_goals`.
  - All of these are undocumented internals.

### Working directory, project, workspace

- **`cwd`**: set per thread and overridable per turn. `thread/list` filters by "session current working directory". [App Server](https://learn.chatgpt.com/docs/app-server#list-threads-with-pagination--filters)
- **Project root**: "Codex treats a directory containing `.git` as the project root", configurable with `project_root_markers`. [Advanced config](https://learn.chatgpt.com/docs/config-file/config-advanced#project-root-detection)
- **Project trust**: `projects.<path>.trust_level` (`"trusted"` / `"untrusted"`). Project-scoped `.codex/` layers load only when the project is trusted. [Config reference](https://learn.chatgpt.com/docs/config-file/config-reference)
- **Projects (desktop app)**: "a project to organize related chats"; local projects "connect to folders on your computer". [Projects and chats](https://learn.chatgpt.com/docs/projects)
- **Workspace**:
  - The sandbox mode `workspace-write`.
  - Runtime "workspace roots": `SessionMeta.runtime_workspace_roots`, and observed `turn_context.workspace_roots`.
  - Separately, a ChatGPT **workspace** is an organization account ("workspace settings").
- **Worktree terms**: **Local checkout**, **Worktree**, **Handoff** ("moves a chat between Local and Worktree"). [Worktrees](https://learn.chatgpt.com/docs/environments/git-worktrees)

### Sub-agents

- **Core terms** ([Subagents](https://learn.chatgpt.com/docs/agent-configuration/subagents#core-terms)):
  - **Subagent workflow**: "Codex runs parallel agents and combines their results."
  - **Subagent**: "A delegated agent that Codex starts to handle a specific task."
  - **Agent thread**: "The thread where a subagent does its work."
- **Configuration and commands**:
  - `/agent` / `/subagents` switch the active agent thread. The "main thread collects the subagent results".
  - Custom agents live in `~/.codex/agents/` and `.codex/agents/`.
  - Config keys: `agents.enabled`, `agents.max_concurrent_threads_per_session` (legacy `agents.max_threads`).
- **Source model** ([protocol.rs](https://github.com/openai/codex/blob/60e35765c3e43e152bf5b382a38a0628efd70842/codex-rs/protocol/src/protocol.rs#L2847)):
  - `SubAgentSource` values: `review`, `compact`, `thread_spawn { parent_thread_id, depth, agent_path, agent_nickname, agent_role }`, `memory_consolidation`, `other`.
  - `ThreadSource`: `user`, `subagent`, `guardian_review`, `memory_consolidation`, or a feature name.
  - Observed `thread_source` values: `user`, `subagent`, `agent_created_thread`, `agent_forked_thread`, `automation`, `voice_chat`.
- **Collab tools and statuses** ([item.rs](https://github.com/openai/codex/blob/60e35765c3e43e152bf5b382a38a0628efd70842/codex-rs/app-server-protocol/src/protocol/v2/item.rs#L1115)):
  - `CollabAgentTool`: `spawnAgent`, `sendInput`, `resumeAgent`, `wait`, `closeAgent`, `sendMessage`, `followupTask`, `interruptAgent`, `listAgents`.
  - `CollabAgentStatus`: `pendingInit`, `running`, `interrupted`, `completed`, `errored`, `shutdown`, `notFound`.
- **Hooks**: `SubagentStart` / `SubagentStop` carry `agent_id` and `agent_type` ("Subagent type or profile"). [Hooks](https://learn.chatgpt.com/docs/hooks#subagentstart)

### Every use of "task" and "todo"

1. **Turn events on the wire**: `task_started` / `task_complete` mean turn start and turn end (see above).
2. **Codex cloud tasks**:
   - `codex cloud exec` "submits a task directly".
   - `codex cloud list --json` returns "a `tasks` array"; each task has `id`, `url`, `title`, `status`, `updated_at`, `environment_id`, `environment_label`, `summary`, `is_review`, `attempt_total`. [CLI commands](https://learn.chatgpt.com/docs/developer-commands#codex-cloud)
   - The same docs now call these "cloud chats".
   - Source crate `cloud-tasks`: `TaskStatus` is `pending`, `ready`, `applied`, `error`; the internal Rust enum `AttemptStatus` has variants `Pending`, `InProgress`, `Completed`, `Failed`, `Cancelled`, `Unknown` (no serde casing). [cloud-tasks-client/src/api.rs](https://github.com/openai/codex/blob/60e35765c3e43e152bf5b382a38a0628efd70842/codex-rs/cloud-tasks-client/src/api.rs#L29)
   - Cloud page heading: "Run coding tasks in parallel cloud environments". [Codex cloud](https://learn.chatgpt.com/docs/cloud)
3. **Scheduled tasks**: the Automations page is titled "Scheduled tasks" and says "Schedule recurring tasks to run in the background". [Scheduled tasks](https://learn.chatgpt.com/docs/automations). Observed: a local `~/.codex/automations/` directory.
4. **Codex Remote**: "Codex runs each task on your connected computer". [Codex Remote](https://learn.chatgpt.com/docs/remote)
5. **`/goal`**: "Set, edit, pause, resume, view, or clear a task goal". [CLI commands](https://learn.chatgpt.com/docs/developer-commands)
6. **`codex exec resume`**: "lets you continue non-interactive tasks". [CLI commands](https://learn.chatgpt.com/docs/developer-commands#codex-exec)
7. **OTel metrics**: `task.compact`, `task.review`, `task.undo`, `task.user_shell`, under the heading "Threads, tasks, and features". [Advanced config](https://learn.chatgpt.com/docs/config-file/config-advanced#threads-tasks-and-features)
8. **Collab tool**: `followupTask` (above).
9. **Plan / todo tool**: `update_plan` is documented in source as "Arguments for the `update_plan` todo/checklist tool (not plan mode)". Steps have `StepStatus` `pending`, `in_progress`, `completed`, and the file comment reads "Types for the TODO tool arguments". [plan_tool.rs](https://github.com/openai/codex/blob/60e35765c3e43e152bf5b382a38a0628efd70842/codex-rs/protocol/src/plan_tool.rs)
   - App-server `turn/plan/updated` carries `plan` entries `{ step, status }` with `pending`, `inProgress`, `completed`. [App Server](https://learn.chatgpt.com/docs/app-server#turn-events)
10. **Safety**: the security page has a section titled "Safety monitoring and paused tasks". [Agent approvals & security](https://learn.chatgpt.com/docs/agent-approvals-security)

**Plan**:

- **plan mode**: `/plan`, `collaboration_mode` `plan`.
- **`plan` item**: "proposed plan text in plan mode".
- Kept separate from the `update_plan` checklist.
- Sources: [CLI commands](https://learn.chatgpt.com/docs/developer-commands), [App Server](https://learn.chatgpt.com/docs/app-server#items).

**Goal**: a **thread goal** with `objective`, `status`, `tokenBudget`, `tokensUsed`, and `timeUsedSeconds`. `ThreadGoalStatus` is `active`, `paused`, `blocked`, `usageLimited`, `budgetLimited`, `complete`. [App Server](https://learn.chatgpt.com/docs/app-server#manage-a-thread-goal), [thread.rs](https://github.com/openai/codex/blob/60e35765c3e43e152bf5b382a38a0628efd70842/codex-rs/app-server-protocol/src/protocol/v2/thread.rs#L795)

### Hooks and lifecycle events

- **When hooks run** ([Hooks](https://learn.chatgpt.com/docs/hooks)):

  | When | Hooks |
  | --- | --- |
  | During a turn | `PreToolUse`, `PermissionRequest`, `PostToolUse`, `PreCompact`, `PostCompact`, `UserPromptSubmit`, `SubagentStop`, `Stop` |
  | When you interrupt an active turn | `Interrupt` |
  | When a session or subagent starts | `SessionStart`, `SubagentStart` |
  | When the main thread ends | `SessionEnd` |

- **Config**: `hooks.json` or `[hooks]` in `config.toml`. Non-managed hooks "must be reviewed and trusted before they run". Observed: `~/.codex/hooks.json` with keys `PermissionRequest`, `PostToolUse`, `PreToolUse`, `SessionStart`, `Stop`, `SubagentStart`, `SubagentStop`, `UserPromptSubmit`.
- **Common input fields**: `session_id`, `transcript_path`, `cwd`, `hook_event_name`, `model`; turn-scoped hooks add `turn_id`.
- **`permission_mode` values**: `default`, `acceptEdits`, `plan`, `dontAsk`, `bypassPermissions`. These are the same names Claude Code uses.
- **Transcript stability**: "the transcript format isn't a stable interface for hooks". [Hooks](https://learn.chatgpt.com/docs/hooks#common-input-fields)

### Official external observation

- **App server** (`codex app-server`) ([App Server](https://learn.chatgpt.com/docs/app-server)):
  - JSON-RPC 2.0 over stdio JSONL (default), a Unix socket, or experimental WebSocket.
  - "Use it when you want a deep integration inside your own product: authentication, conversation history, approvals, and streamed agent events."
  - Notifications: `thread/started`, `thread/status/changed`, `thread/closed`, `thread/archived`, `turn/started`, `turn/completed`, `turn/plan/updated`, `turn/diff/updated`, `item/started`, `item/completed`, `item/*/delta`, `hook/started`, `hook/completed`, `serverRequest/resolved`, `thread/tokenUsage/updated`, `account/rateLimits/updated`.
  - Server requests (approvals): `item/commandExecution/requestApproval`, `item/fileChange/requestApproval`, `item/permissions/requestApproval`, `item/tool/requestUserInput`, `mcpServer/elicitation/request`.
  - `thread/list` / `thread/read` read stored threads without resuming them.
  - Unverified: whether one app-server process can see the live status of a thread running in a *separate* TUI or desktop process. `thread/loaded/list` returns only "thread IDs currently loaded in memory".
- **Hooks**: command hooks receive JSON on stdin (see above).
- **`notify`**: runs an external program "whenever Codex emits supported events (currently only `agent-turn-complete`)". The JSON argument has `type`, `thread-id`, `turn-id`, `cwd`, `input-messages`, `last-assistant-message`. [Advanced config](https://learn.chatgpt.com/docs/config-file/config-advanced#notifications)
- **`tui.notifications`**: in-TUI terminal notifications filtered by event type (`agent-turn-complete`, `approval-requested`), with `tui.notification_condition` `unfocused` or `always`. [Advanced config](https://learn.chatgpt.com/docs/config-file/config-advanced#notify-vs-tuinotifications)
- **`codex exec --json`**: a JSONL event stream with `thread.started` (`thread_id`), `turn.started`, `turn.completed`, `turn.failed`, `item.*`, `error`. [Non-interactive mode](https://learn.chatgpt.com/docs/non-interactive-mode#make-output-machine-readable)
- **Codex SDK** (`@openai/codex-sdk`): "start, continue, and resume local Codex threads". [Codex SDK](https://learn.chatgpt.com/docs/codex-sdk)
- **OpenTelemetry** (`[otel]` in user config) ([Advanced config](https://learn.chatgpt.com/docs/config-file/config-advanced#observability-and-telemetry)):
  - Events: `codex.conversation_starts`, `codex.api_request`, `codex.sse_event`, `codex.websocket_request`, `codex.websocket_event`, `codex.user_prompt`, `codex.tool_decision`, `codex.tool_result`.
  - Event metadata includes "conversation id".
  - Metrics include `thread.started`, `conversation.turn.count`, `thread.fork`, `multi_agent.spawn`.
- **Removed**: the MCP server (`codex mcp-server`) is gone; the docs say "Use the Codex app server". [MCP server removal](https://learn.chatgpt.com/docs/mcp-server)
- **Not external**: `tui.status_line` is an "Ordered list of TUI footer status-line item identifiers". It configures the TUI footer and doesn't run an external command. [Config reference](https://learn.chatgpt.com/docs/config-file/config-reference)

### Session status vocabulary

- **`ThreadStatus`** (tagged `type`): `notLoaded`, `idle`, `systemError`, `active { activeFlags }`. `ThreadActiveFlag` is `waitingOnApproval` or `waitingOnUserInput`. [thread.rs](https://github.com/openai/codex/blob/60e35765c3e43e152bf5b382a38a0628efd70842/codex-rs/app-server-protocol/src/protocol/v2/thread.rs#L1644)
  - The docs show `{ "type": "active", "activeFlags": ["waitingOnApproval"] }`. [App Server](https://learn.chatgpt.com/docs/app-server#track-thread-status-changes)
- **`TurnStatus`**: `inProgress`, `completed`, `interrupted`, `failed`. [turn.rs](https://github.com/openai/codex/blob/60e35765c3e43e152bf5b382a38a0628efd70842/codex-rs/app-server-protocol/src/protocol/v2/turn.rs#L32)
- **`TurnAbortReason`**: `interrupted`, `replaced`, `review_ended`, `budget_limited`. [protocol.rs](https://github.com/openai/codex/blob/60e35765c3e43e152bf5b382a38a0628efd70842/codex-rs/protocol/src/protocol.rs#L4207)
- **Command and file-change item status**: `inProgress`, `completed`, `failed`, `declined`. [App Server](https://learn.chatgpt.com/docs/app-server#command-execution-approvals)
- **Desktop app**:
  - A pet shows a chat as **Running**, **Needs input**, **Ready**, or **Blocked**.
  - The Activity view lists chats that are "unread, running, or waiting for your response".
  - [Notifications](https://learn.chatgpt.com/docs/notifications)
- **Other status enums**: subagent (`CollabAgentStatus`), goal (`ThreadGoalStatus`), and cloud task (`TaskStatus`) statuses are listed above.

### Approvals and permissions

- **Approval policy** (`approval_policy` / `--ask-for-approval`): `on-request`, `never`, `granular`. `untrusted` "no longer supported". [Advanced config](https://learn.chatgpt.com/docs/config-file/config-advanced#approval-policies-and-sandbox-modes), [Agent approvals & security](https://learn.chatgpt.com/docs/agent-approvals-security)
- **Sandbox mode** (`sandbox_mode` / `--sandbox`): `read-only`, `workspace-write`, `danger-full-access`. [config_types.rs](https://github.com/openai/codex/blob/60e35765c3e43e152bf5b382a38a0628efd70842/codex-rs/protocol/src/config_types.rs#L104)
- **Approvals reviewer**: `approvals_reviewer` is `user` or `auto_review`.
- **Presets and UI modes**:
  - The `Auto` preset is `workspace-write` plus `on-request`.
  - UI permission modes: **Ask for approval**, **Approve for me** (called **Auto-review** in settings), **Full access**.
  - Sources: [Agent approvals & security](https://learn.chatgpt.com/docs/agent-approvals-security), [Permissions](https://learn.chatgpt.com/docs/permission-modes).
- **Permission profiles** (beta) configure filesystem and network access together. [Permissions](https://learn.chatgpt.com/docs/permissions)
- **App-server approval decisions**: `accept`, `acceptForSession`, `decline`, `cancel`. Permission grants use `scope` `"turn"` or `"session"`. [App Server](https://learn.chatgpt.com/docs/app-server#approvals)
- **Rules / execpolicy** control "which commands Codex can run outside the sandbox". [Rules](https://learn.chatgpt.com/docs/agent-configuration/rules)

### Cloud and remote variants

- **Codex cloud**: "Delegate work to Codex in isolated cloud environments". [Codex cloud](https://learn.chatgpt.com/docs/cloud)
  - Terms: **environment**, **cloud chats** / **tasks**, `codex cloud`, `codex cloud exec`, `codex cloud list`.
  - The source crate is still named `cloud-tasks`.
- **Codex Remote**: "Start, guide, approve, and review Codex tasks on a connected computer from your phone". [Codex Remote](https://learn.chatgpt.com/docs/remote)
- **Remote connections**: access a connected computer, or connect the desktop app to projects over SSH. [Remote connections](https://learn.chatgpt.com/docs/remote-connections)
- **Remote TUI**: `codex --remote ws://…` connects the terminal UI to an `app-server` running elsewhere. [App Server](https://learn.chatgpt.com/docs/app-server#connect-the-cli-terminal-ui)

## Side-by-side

| Concept | Claude Code term | Codex term | Mismatch notes |
| --- | --- | --- | --- |
| One run / conversation | session (= conversation); "main thread" vs subagents | thread (protocol); chat (user docs); session (hooks, CLI, root of a thread tree); rollout (file) | In Codex a session ID is the root thread's ID and forks keep it. In Claude Code a fork gets a new session ID. |
| Stored log of a run | transcript (`<session-id>.jsonl`) | rollout (`rollout-<ts>-<uuid>.jsonl`) | Both formats are officially internal. |
| One user request + agent work | prompt / interaction (`prompt.id`); "turn" in loose CLI prose | turn (`turn/start`, `turn/completed`) | Agent SDK "turn" is one model round trip, so one prompt is many turns. Codex v1 wire calls a turn `task_started` / `task_complete`. |
| Unit inside an exchange | message (SDK types); transcript entry; content block (`text`, `tool_use`, …) | item (`ThreadItem`: `userMessage`, `agentMessage`, `commandExecution`, …) | Claude Code has no "item" term. |
| Session ID | UUID (v4 observed); background short ID (8 hex) | thread ID, UUID v7; `sessionId` = root thread ID | |
| Continue latest | `claude --continue` | `codex resume --last` | Both are scoped to the cwd. |
| Resume by ID | `claude --resume <id\|name\|path>`, `/resume` | `codex resume <id>`, `codex exec resume`, `thread/resume` | |
| Fork | `--fork-session`, `/branch` (new session ID); `/fork` (new background session) | `codex fork`, `/fork`, `thread/fork` (`forkedFromId`) | `/fork` means different things. |
| New conversation in-process | `/clear` (new `session.id`) | `/new`, `/clear` (new chat in same CLI session) | |
| Name | name (`--name`, `/rename`), generated title, default display name | thread name (`thread/name/set`, `/rename`); `thread_name` in `session_index.jsonl` | |
| History location | `~/.claude/projects/<cwd-slug>/` | `~/.codex/sessions/YYYY/MM/DD/`, plus SQLite state | Claude groups by project dir; Codex groups by date. |
| Prompt history | `~/.claude/history.jsonl` | `~/.codex/history.jsonl` | Codex docs call it "session transcripts"; observed content is prompt text only. |
| Directory scoping | project directory; `workspace.project_dir` / `current_dir` / `added_dirs` | `cwd`; project root (`.git` marker); `workspace_roots`; desktop Projects | |
| Trust | workspace trust | project `trust_level` `trusted` / `untrusted` | |
| Subagent | subagent (`Agent` tool); fork; background subagent; agent ID | subagent; agent thread; subagent workflow; `agent_nickname` / `agent_role` | Codex subagents are full threads with parent/child edges. |
| Multi-agent peers | agent team: team lead, teammates, mailbox | collab tools (`spawnAgent`, `sendMessage`, …) | |
| Background runs | background session / background agent / job; supervisor; agent view | none equivalent in CLI; desktop Activity view; background terminals (`/ps`) | |
| Checklist | task list (`TaskCreate`…), legacy `TodoWrite` | plan (`update_plan`, "todo/checklist tool"; `turn/plan/updated`) | Claude says "task"; Codex says "plan step". |
| Checklist item status | `pending`, `in_progress`, `completed` | `pending`, `in_progress` / `inProgress`, `completed` | |
| Plan mode | `plan` permission mode; `~/.claude/plans/` | `plan` collaboration mode; `plan` item | Claude treats plan as a permission mode; Codex treats it as a collaboration mode. |
| Goal | `/goal` completion condition | thread goal (`/goal`, `thread/goal/*`), with token budget and time used | |
| Lifecycle hooks | 30+ events incl. `SessionStart`, `Stop`, `Notification`, `SubagentStart/Stop`, `TaskCreated/Completed`, `TeammateIdle`, `SessionEnd` | `SessionStart`, `SessionEnd`, `UserPromptSubmit`, `PreToolUse`, `PermissionRequest`, `PostToolUse`, `PreCompact`, `PostCompact`, `SubagentStart`, `SubagentStop`, `Stop`, `Interrupt` | Codex copies Claude's event and field names (`session_id`, `transcript_path`, `permission_mode`) and adds `turn_id`. |
| Live session state | `state`: `working` / `blocked` / `done` / `failed` / `stopped`; `status`: `busy` / `waiting` / `idle`; `waitingFor` | `ThreadStatus`: `notLoaded` / `idle` / `systemError` / `active` + `waitingOnApproval` / `waitingOnUserInput` | Claude `done` roughly equals Codex `idle` after a completed turn. Claude `blocked` roughly equals Codex `active` + a waiting flag. |
| Exchange outcome | `Stop` / `StopFailure` hooks | `TurnStatus`: `inProgress` / `completed` / `interrupted` / `failed` | |
| Permission levels | permission modes `default` (Manual), `acceptEdits`, `plan`, `auto`, `dontAsk`, `bypassPermissions` | approval policy (`on-request`, `never`, `granular`) × sandbox mode (`read-only`, `workspace-write`, `danger-full-access`); UI modes Ask for approval / Approve for me / Full access | Codex hooks still report Claude-style `permission_mode` names. |
| Approval request | permission prompt / permission decision; rules allow / ask / deny | approval request; decisions `accept` / `acceptForSession` / `decline` / `cancel` | |
| Cloud run | cloud session (`--cloud`), cloud environment, routine | Codex cloud task / cloud chat, environment, scheduled task | |
| Remote control of local run | Remote Control session | Codex Remote; remote connections; `codex --remote` | |
| Push-based observation | hooks, status line command, OTel | hooks, `notify`, app-server notifications, OTel | |
| Poll-based observation | `claude agents --json` | app-server `thread/list` / `thread/read` | |

## Collisions with adhd-ranch terms

- **Task**: heavy collision in both tools. Claude Code uses "task" for:
  - the checklist item (`TaskCreate`, the task list, `~/.claude/tasks/`, `TaskCreated` / `TaskCompleted` hooks)
  - background work (`/tasks`, background task ID, `TaskStop`)
  - scheduled tasks (`/loop`, Desktop)
  - agent-team shared tasks
  - "the task" a session is working on (agent view `Completed` / `Failed`)

  Codex uses "task" for:
  - turns on the wire (`task_started` / `task_complete`)
  - cloud runs (`codex cloud` tasks, `TaskStatus`)
  - scheduled tasks
  - Remote tasks
  - "task goal"
  - OTel `task.*` metrics
  - the `followupTask` collab tool

  "Todo": Claude `TodoWrite` and legacy `todos/`; Codex source calls `update_plan` the "todo/checklist tool".
- **Timer**: no tool-level concept found. Near-misses: a Claude goal has a "timer" reset on resume ([Goal](https://code.claude.com/docs/en/goal)); a Codex thread goal tracks `timeUsedSeconds`. Delay-style terms exist but aren't called timers: Claude's `idle_prompt` after about 60 seconds and Codex's `autoResolutionMs`.
- **Focus**: Claude Code has a `/focus` command that toggles the "focus view" ([Commands](https://code.claude.com/docs/en/commands)). Codex has `tui.notification_condition = "unfocused"`, a terminal-focus condition. Neither tool has a focus-session concept.
- **Session**: both tools use it as the top-level run, but with different scope. Claude Code: session = one conversation, and `/clear` starts a new one. Codex: "session" can mean the root thread tree (`sessionId`), a whole CLI process holding several chats, a `command/exec` session, or a fuzzy-file-search session.
- **Project**: both tools use it as a directory scope. Claude Code: `~/.claude/projects/<cwd-slug>` and `claude project purge`. Codex: project root, `projects.<path>.trust_level`, desktop Projects that group chats, and observed `projects` / `project_roots` SQLite tables. GitLab "project" also appears in Codex cloud docs.
- **Workspace**: both tools, with different meanings. Claude Code: the status-line `workspace.*` object (dirs and repo), "workspace trust", and the OTel `workspace.host_paths` attribute. Codex: the `workspace-write` sandbox mode, `workspace_roots`, and a ChatGPT workspace (organization account).
- **Agent**: both tools. Claude Code: subagent, background agent, agent view, `--agent`, agent team, `agent_id` / `agent_type`. Codex: subagent, agent thread, custom agents (`~/.codex/agents/`), `[agents]` config, `AGENTS.md`, and "the Codex agent".
- **Thread**: Codex's core noun (thread = conversation; agent thread = subagent run). Claude Code only uses "main thread" for the top-level conversation versus subagents.
- **Limit**: both tools, for usage and rate limits. Claude Code: status-line `rate_limits.five_hour` / `seven_day` / `spend_limit`, `StopFailure` `rate_limit`, `maxTurns`, and `quota_auto_resume_*` notifications. Codex: `UsageLimitExceeded`, goal `usageLimited` / `budgetLimited`, `TurnAbortReason` `budget_limited`, `account/rateLimits/*`, and `agents.max_concurrent_threads_per_session` (a cap).
- **Notification**: both tools. Claude Code: the `Notification` hook event with `notification_type`, plus `agentPushNotifEnabled` / `inputNeededNotifEnabled` push settings. Codex: `notify` (external program on `agent-turn-complete`), `tui.notifications`, and JSON-RPC "notifications" (app-server messages without an `id`, such as `turn/completed`).

Other adhd-ranch words that also appear:

- **Complete**: Claude `TaskCompleted`, task status `completed`, agent view `Completed`. Codex `turn/completed`, `TurnStatus.completed`, goal `complete`.
- **Clear**: Claude `/clear` (new conversation) and `SessionStart` source `clear`. Codex `/clear` (new chat), `thread/goal/clear`, and `SessionStart` source `clear`.
- **Delete**: Claude `claude rm` (a background session; the transcript is kept). Codex `thread/delete` (permanent).
- **Settings**: Claude `settings.json`. Codex `config.toml`, but "Settings" in the desktop app.

## External observation mechanisms, one line each

**Claude Code**:

- `claude agents --json` is the documented, supported poll interface for live session state (`state`, `status`, `waitingFor`, `sessionId`, `cwd`).
- Hooks are command hooks that receive JSON on stdin for 30+ lifecycle events, including `SessionStart`, `Stop`, `Notification`, and `SessionEnd`.
- The status line is a user command that receives per-update session JSON (`session_id`, `workspace`, `cost`, `context_window`, `rate_limits`).
- OpenTelemetry exports metrics, logs/events, and beta traces keyed by `session.id` and `prompt.id`.
- Agent SDK and `claude -p --output-format stream-json` stream the messages of runs your program starts.
- Transcript JSONL is reachable via `transcript_path`, but the docs call the format internal.

**Codex**:

- App server is a JSON-RPC 2.0 protocol with thread, turn, and item notifications (`thread/status/changed`, `turn/completed`, `item/*`) plus approval requests. It is the documented integration surface for clients.
- Hooks are command hooks that receive JSON on stdin for 12 lifecycle events, including `SessionStart`, `Stop`, `Interrupt`, and `SessionEnd`.
- `notify` runs an external program with a JSON argument on `agent-turn-complete` only.
- `codex exec --json` gives a JSONL event stream for non-interactive runs your program starts.
- Codex SDK (TypeScript) starts, continues, and resumes local threads programmatically.
- OpenTelemetry (`[otel]`) exports log events and metrics carrying the conversation or thread ID.
- Rollout JSONL files and SQLite state are on disk, but hooks docs say the transcript format "isn't a stable interface". SQLite schemas are undocumented.

## Unverified or inferred

- The rule that a Claude Code background job directory name equals the first 8 hex characters of the session UUID is inferred from three local directories and isn't documented.
- The field lists for `~/.claude/sessions/<pid>.json`, `~/.claude/jobs/<id>/state.json`, subagent `.meta.json`, `~/.codex/session_index.jsonl`, and the Codex SQLite tables are observed only. Neither vendor documents them.
- Codex source line anchors point at commit `60e3576`, fetched 2026-09-14. The protocol files change often.
- Whether an external app-server client can observe live status for threads running in a separate Codex TUI or desktop process isn't documented.

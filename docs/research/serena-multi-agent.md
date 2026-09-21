# Serena across multiple agents and worktrees

How to configure the Serena MCP server for this repo, and what breaks when agents run in git worktrees. Written after testing the failure directly; the negative results are recorded so nobody repeats them.

Sources and versions, checked 2026-09-21:

- Serena 1.7.0, installed at `~/.local/bin/serena`. Source paths below are relative to `~/.local/share/uv/tools/serena-agent/lib/python3.13/site-packages/`.
- Serena docs at `https://oraios.github.io/serena/`, issues at `https://github.com/oraios/serena`.
- Claude Code docs at `https://code.claude.com/docs/en/<page>`.
- Facts marked **observed** come from experiments on this machine and can change in any release.

## The three rules

1. **Commit `.serena/`.** `project.yml` and `memories/` are versioned; Serena's own `.serena/.gitignore` excludes `cache/` and `project.local.yml`.
2. **One session per worktree, started with the working directory already inside it.** Never create or enter a worktree mid-session.
3. **Block Serena on worktree-isolated agents** with `disallowedTools: mcp__serena__*` in the agent definition.

Rules 1 and 2 are Serena's documented recommendations. Rule 3 exists because Claude Code and Serena disagree about what a worktree is.

## How Serena picks a project

`find_project_root()` (`serena/cli.py:87-101`) walks up from the working directory and stops at the first ancestor containing `.serena/project.yml` **or** `.git`. `.git` is tested with `.exists()`, so a worktree's `.git` *pointer file* counts and the worktree wins over any ancestor.

That is deliberate. Serena v1.6.0 changed it specifically to fix worktrees:

> Fix `--project-from-cwd` hijacking git worktrees nested under a Serena project… This previously bound CLI agents (Claude Code, Codex, Gemini) launched from inside a worktree to the parent repo, causing stale reads and misdirected edits.

On any version before 1.6.0, a worktree under `<repo>/.claude/worktrees/<name>` resolved to the parent repo. On 1.6.0+ it does not. Check the version before trusting older notes on this.

Resolution happens **once**, at server startup (`cli.py:363` → `serena/agent.py:670-676`). `get_active_project()` (`agent.py:916-920`) is a plain getter — no re-resolution, no path argument. The `claude-code` context sets `single_project: true`, which disables `activate_project` entirely, so a running server can never change its root.

## What worktree isolation breaks

Claude Code starts **one Serena server per session**, as a child of the Claude Code host process, inheriting the session's working directory. Subagents share it. A subagent launched with `isolation: "worktree"` gets file tools scoped to its worktree but Serena tools still resolved against the parent checkout.

**Observed.** A subagent edited `slugify` in its own worktree with built-in tools, then read it back through Serena: `find_symbol` returned the parent checkout's body, without the edit and byte-identical to the parent's copy. `find_referencing_symbols` returned the parent's callers. A file created only in the worktree came back `FileNotFoundError`.

Nothing warns the agent. Every Serena path is relative to the resolved root, and `get_current_config` and `activate_project` are both excluded under `single_project: true`, so the agent cannot discover its own root. A `replace_symbol_body` from that agent writes to the parent checkout.

Upstream has ruled on this. [oraios/serena#1496](https://github.com/oraios/serena/issues/1496) is closed as completed with no code change:

> This confirms that this is a problem in Claude Code, not Serena, and there is not really anything we can do about it.

That thread also reports (tested against Claude Code v2.1.219) that Claude Code never fires `notifications/roots/list_changed` for `git worktree add` mid-session, for worktree-isolated subagents, for Agent Teams teammates, or for the native `EnterWorktree` tool. Hence rule 2: the only confirmed-safe arrangement is a top-level session whose working directory is already the worktree.

Claude Code's own [worktrees doc](https://code.claude.com/docs/en/worktrees.md) does not mention MCP servers in either direction. This is unspecified behaviour, not documented behaviour.

## Three fixes that do not work

**`excluded_tools` in `project.yml`.** Bans Serena's write tools repo-wide, including in the main session where Serena is correct. It also under-fixes: reads are wrong in a worktree too, and a worktree agent reading the parent's code cannot tell.

**Permission `deny` rules.** `mcp__serena__*` patterns are valid in `settings.json` ([permissions](https://code.claude.com/docs/en/permissions.md)), but permission rules are global — there is no documented way to scope one to a subagent. Same over-broad trade.

**Inline `mcpServers` in agent frontmatter.** The most promising idea, and wrong in the most dangerous way. Agent frontmatter can declare its own MCP server, connected only while that agent runs. It looks worktree-scoped. It is not.

**Observed.** An agent under worktree isolation declared its own Serena. The server spawned (a second process, 36 s after the session's), and its log read:

```
Auto-detected project root: /Users/ryan/Code/killallgit/adhd-ranch
Activating adhd-ranch at /Users/ryan/Code/killallgit/adhd-ranch
```

`lsof` confirmed its working directory was the main checkout. Both servers returned byte-identical bodies for a symbol that had been edited in the worktree. The inline server is spawned as a child of the Claude Code **host** process, whose working directory is the session's — worktree isolation never changes it.

This is worse than having no Serena, because the configuration reads as isolated while silently resolving to the parent.

## What works: `disallowedTools`

Documented in [sub-agents](https://code.claude.com/docs/en/sub-agents.md): `disallowedTools` applies first, then `tools` resolves against what remains. `mcp__serena__*` removes one server, `mcp__*` removes all MCP tools.

**Observed.** With `disallowedTools: mcp__serena__*`, Claude Code reports the agent's toolset as "All tools except `mcp__serena__*`". See `.claude/agents/worktree-coder.md`.

Do not rely on allowlist-by-omission — a `tools:` list containing only built-ins appears to exclude MCP tools, but that is not documented.

## This repo's configuration

`.serena/project.yml` is tracked. `~/.gitignore_global` has a blanket `.serena` entry, so the root `.gitignore` re-includes it:

```
!.serena/
!.serena/project.yml
```

`language_servers` is `[rust, typescript]`. Both are needed — the frontend is ~95 TS/TSX files against ~72 Rust files. This matters more than it looks: when no `project.yml` exists, Serena auto-generates one non-interactively and guesses. Observed bad guesses: `[typescript]` for this repo with Rust silently dropped, `[cpp]` for a Python repo, `[python]` for four worktrees of one repo. `serena project create` prompts when it finds several languages; the `--project-from-cwd` autogeneration path does not.

Dashboard settings live in `~/.serena/serena_config.yml` and are global, not per-repo. For multi-agent work set `web_dashboard_open_on_launch: false` and `web_dashboard_interface: browser`. Leaving the interface unset means the macOS default of `app`, which adds a tray icon per running instance — Serena's own config comments warn about this for multi-agent setups. `tray_manager` is experimental and untested on macOS. The dashboard stays reachable at `http://localhost:24282/dashboard/`, and later instances at 24283, 24284, and so on.

## Known hazards

**Registry growth.** Serena appends every resolved project to `projects:` in `~/.serena/serena_config.yml`, including every worktree, with auto-registration hardcoded and no opt-out. Dead paths are pruned at load time (`config/serena_config.py:1066-1076`) and the file self-heals on the next write, so removed worktrees clean themselves up. Live ones accumulate: observed going from 6 to 8 entries in about an hour across several sessions. `project remove` is not in 1.7.0 — it exists only in unreleased main ([#2029](https://github.com/oraios/serena/issues/2029)), where the maintainer's position is that the file is meant to be hand-edited.

**Concurrent registry writes.** [#1850](https://github.com/oraios/serena/issues/1850): parallel Serena processes auto-registering projects can overwrite each other's `projects:` list, because the persist path reloads from disk and then writes back its own stale copy. Fixed only in unreleased main. 1.7.0 has no locking. This is live for anyone running several sessions at once.

**Duplicate project names.** Now that `project.yml` is tracked, every worktree carries `project_name: "adhd-ranch"`. `get_registered_project()` raises `Multiple projects found with name '{name}'` on ambiguity (`config/serena_config.py:1216-1220`). Path lookup is unaffected, so `--project-from-cwd` is fine, but never reference a project by name. [Discussion #718](https://github.com/oraios/serena/discussions/718) asks this and has no maintainer answer.

**CRLF on fresh checkouts.** `core.autocrlf` is `true` globally on this machine — the Windows recommendation, wrong on macOS. Existing working-tree files are unaffected, so this is easy to miss. A *fresh* checkout is not: `git checkout-index` of the committed `project.yml` produced 9962 bytes with 169 CR lines, against 9793 bytes and zero in the blob. Every new worktree and clone gets CRLF. Serena has no `.gitattributes` guidance; setting `core.autocrlf` to `input` fixes it at the source.

**Fresh worktrees have no `dist/`.** It is gitignored, and `src-tauri/tauri.conf.json` points `frontendDist` at `../dist`, so `tauri::generate_context!` panics in a new worktree and rust-analyzer degrades across `src-tauri/`. Build the frontend in the worktree before trusting Rust analysis there.

**Two rust-analyzers.** Serena spawns its own as a child of its server. `.claude/settings.json` also enables the `rust-analyzer-lsp` plugin, which Claude Code spawns separately for inline diagnostics. Both index this repo. Observed during active work: roughly 282 MB and 2.2 GB respectively. They serve different consumers, so this is a cost to accept or a plugin to disable, not a bug.

## `query-projects` mode

The `query-projects` mode adds `list_queryable_projects` and `query_project` for reading from projects other than the active one. `list_queryable_projects` works in-process. `query_project` with any **symbolic** tool requires a separate project server:

```
ConnectionError: ProjectServer is not reachable at http://127.0.0.1:24225.
```

Start it with `serena start-project-server`. It runs its own agent and spawns language servers per queried project, and serializes requests behind one lock. Enable the mode only alongside that server, and only for a real cross-repo need.

## HTTP transport

`--transport streamable-http` serves at `http://<host>:<port>/mcp`, and lets several agents share one instance and one set of language servers. Its limits are narrower than they first appear. Serena is stateful with one active project per process, so the docs restrict this to clients working on **the same project**. All tool calls from all connected clients are serialized through a single FIFO queue (`serena/task_executor.py:124-155`), so N agents contend, bounded by `tool_timeout`. For several agents on *different* projects, the documented answer is a separate stdio instance each — which is what Claude Code already does.

## Applying this to another repo

1. Re-include `.serena/` if a global gitignore hides it, then commit `.serena/project.yml` and `.serena/.gitignore`.
2. Check `language_servers` against what the repo actually contains. Autogenerated values are frequently wrong.
3. Copy `.claude/agents/worktree-coder.md`, or add `disallowedTools: mcp__serena__*` to whichever agent definitions run under worktree isolation.
4. Prefer a session per worktree over worktree-isolated subagents whenever the work needs Serena.

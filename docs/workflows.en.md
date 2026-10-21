# Khaslana workflows

This is the English reference corresponding to [workflows.md](workflows.md). Workflows execute sequential Git operations in the active repository. JSON5/JSONC v1 and v2 are supported. V2 adds local stdio MCP, restricted JavaScript and AI Skills; arbitrary shell and Python scripts are not supported.

## Create and edit templates

Choose New on the workflow page. Start from a preset (synchronize the current branch, create and push a feature branch, or merge and push), or add steps manually. Enter the display name and required filename, add Skill, JavaScript or Git steps, then optionally define inputs and fixed variables. Steps can be searched, reordered, duplicated and deleted. Save validates the definition and selects the saved template; it never runs it automatically.

The template context menu offers Edit, Duplicate and Delete. Editing overwrites the original file, including `.jsonc`; changing the filename renames it by writing the new file and deleting the old one. Duplicate requires another filename and leaves the original intact. Delete requires confirmation and only deletes the template. Visual editing regenerates formatting and removes handwritten comments (with a warning); use manual editing or Duplicate to retain the source. Same-name parameters survive step-type changes. Dangerous deletion defaults to dry run.

New templates use V2. V1 retains its editor and can be copied into V2. MCP is a tool used by Skill/JavaScript, not a separate new step. Tool selection is optional and collapsed when empty; collapsing it does not clear selections. Pure JavaScript needs no MCP. Existing standalone MCP steps remain compatible.

### AI assistant

Describe the desired workflow, then choose AI Generate. The configured AI receives the request, the current template when editing, and this reference. Parsed results fill the editor for inspection and manual saving. Configuration must be enabled first. Invalid output reports an error without replacing the current form. You can edit the form during generation, but cannot start another generation until it finishes. The system-instruction language follows the AI language preference; generated natural-language labels and descriptions follow the UI language captured when the task starts. Existing user content is not automatically translated.

## Run a workflow

1. Open the repository and choose Workflow mode.
2. Select a discovered template or choose a `.json5` / `.jsonc` file.
3. Fill the declared text, multiline, select or date inputs.
4. Inspect the expanded step preview and choose Run.
5. Inspect the execution log. Cancel stops subsequent steps after the current Git step; completed effects are not rolled back.

Remote Git operations use the existing credential and proxy mechanisms. A branch context menu can open a template with that local or remote branch as the source, without switching branches or automatically running. `${git.initialBranch}` then refers to that selected source; ordinarily it is the branch at workflow start. A missing source reference stops execution before operations begin.

## Template directory

The visible directory is `C:\Users\<user>\.khaslana\workflows`, resolved through the application's data-directory mechanism. Startup scans its direct `.json5` / `.jsonc` children in the background; repository files are not automatically discovered. Refresh rescans; Open Directory opens the system file manager. Download Templates lists the [CNB catalogue](https://cnb.cool/liuchenchen/work-studio/-/tree/main/work-studio-files), downloading only selected files and never overwriting existing names. Downloads remain available offline. Legacy `remote-cnb` files are imported once, with local names taking precedence; originals remain backups and deleting an imported template does not reimport it. Browsing and downloads do not save credentials. Invalid templates remain visible and report their parse error when loaded.

## File format

JSON5 permits comments, trailing commas and unquoted keys.

```json5
{
  version: 1,
  name: "Create A from master and merge B",
  defaults: { requireCleanWorktree: true },
  inputs: {
    target: { label: "Target branch", default: "A-${date:%Y%m%d}", required: true },
    source: { label: "Branch to merge", default: "B", required: true },
  },
  vars: { remote: "origin", base: "master" },
  steps: [
    { op: "checkout", branch: "${base}" },
    { op: "pull", remote: "${remote}" },
    { op: "createBranch", name: "${target}", from: "${base}", checkout: true },
    { op: "merge", branch: "${source}" },
    { op: "push", remote: "${remote}", branch: "${target}", setUpstream: true },
  ],
}
```

| Field | Required | Meaning |
| --- | --- | --- |
| `version` | yes | `1` or `2`; new templates use V2, editing preserves V1 unless explicitly migrated. |
| `name` | no | Display name; empty means an unnamed workflow. |
| `defaults` | no | Default behavior. |
| `inputs` | no | Pre-run form inputs whose values are strings. |
| `vars` | no | User variables; values must be strings. |
| `steps` | yes | A nonempty array executed sequentially. |

### V2 extensible steps

Use `op: "invoke"`, a unique `id`, `uses` and object `with` (defaults to `{}`). Built-in actions are `git.checkout`, `git.fetch`, `git.pull`, `git.createBranch`, `git.merge`, `git.push`, `git.guardRemoteBranch`, `git.ensureClean`, `git.assertBranch`, `git.filterBranches`, `git.deleteBranches`; their arguments match the V1 steps below. V1 steps remain usable.

```json5
{
  version: 2,
  steps: [
    { op: "invoke", id: "clean", uses: "git.ensureClean", with: {} },
    { op: "invoke", id: "create", uses: "git.createBranch",
      with: { name: "demo", from: "${run.sourceBranch}", checkout: false } },
  ],
}
```

`${run.sourceBranch}` is the selected source and `${run.sourceKind}` is `local` or `remote`. Extension actions can store JSON using `saveAs`, referenced later as `${result.branch}`. Built-in `git.*` actions do not support `saveAs`; use their existing output parameters.

### Local MCP and JavaScript

Explicitly configure custom stdio servers in `workflow-mcp.json5` in the active data directory (the parent of the workflows directory). Templates cannot specify startup commands. The built-in `browser.edge` needs no custom configuration.

```json5
{
  servers: {
    localTools: {
      command: "C:\\path\\to\\mcp-server.exe", args: [],
      tools: { read_page: { access: "read" }, submit_form: { access: "write" } },
    },
  },
}
```

The local configuration determines `access`; a server's read-only hint grants no permission. Custom processes inherit the client environment, but must handle their own authentication and proxy; application credentials and proxy are not implicitly passed. Never put secrets into commands or arguments. Built-in `browser.edge` provides all 25 default tools of pinned Playwright MCP 0.0.82, without extra `--caps`, and uses application proxy settings. Browser proxy does not support authentication or separate HTTP/HTTPS proxy URLs.

AI Settings → MCP → Add Service accepts `npx` and one argument per line (for example `-y`, `chrome-devtools-mcp@latest`). Windows resolves PATH and `.cmd`/`.bat` entries, preserving full paths and old `npx.cmd` settings; standard npm/npx launchers preferably run through their neighboring Node executable. npx uses the active data directory's `workflow-npm-cache`, unless `npm_config_cache` is explicitly set. First connection may download packages. Connection testing waits up to 120 seconds and only lists tools. Custom npx requires system Node.js including npm and an application restart after installation; the built-in browser runtime is not a general npm installation.

Saving a service requires no connection test. New servers, and file configurations omitting `tools`, default to automatic discovery. Unknown tools require per-run write authorization regardless of server hints. `enabled: false` disables a server. `autoDiscover: false` restricts calls to `tools`; legacy explicit tools remain an allowlist. `cachedTools` is a display cache, not schema or permission. Advanced configuration can change access hints; disabling individual tools switches to allowlist mode, and retesting preserves selections. Opening settings/runtime/editor reads local records only, without launching Node/MCP. System Node detection is explicit, and also performed when installing runtime components.

Legacy direct MCP steps remain supported, but new templates should call tools within Skill or JS:

```json5
{ op: "invoke", id: "read", uses: "mcp.call", saveAs: "page",
  with: { server: "localTools", tool: "read_page", arguments: { url: "https://example.com" } } }
```

Server/tool names must be fixed; arguments must be an object and may use workflow variables. Execution connects, discovers tools and validates JSON Schema before one call. Declared output schemas are validated too. Connections and tool definitions are reused within a run, rediscovered on reconnect, and closed on every exit. `saveAs` prefers structured output, otherwise the content array. Failures stop execution; write tools are never automatically retried. Preview preserves expressions depending on unavailable results until execution.

JavaScript is a function body with `input` and a JSON return value. Only declared tools permit `mcp.call(server, tool, arguments)`:

```json5
{ op: "invoke", id: "transform", uses: "js.run", saveAs: "result",
  with: {
    input: { text: "hello" },
    tools: [{ server: "localTools", tool: "read_page" }],
    script: "const page = mcp.call('localTools', 'read_page', {url: input.text}); return {page};",
  } }
```

Scripts have no filesystem, network or process APIs. `${...}` in script source remains JS template syntax; workflow interpolation applies to `input`. Limits: 8 MiB memory, 512 KiB stack, 40 seconds, 256 KiB input, 1 MiB output. MCP arguments are limited to 256 KiB and output to 1 MiB. Pure JS and JS/Skill using only built-in Edge automatically prepare per-run authorization without confirmation. Custom MCP requires confirmation showing script summaries and tool permissions; dismissing it executes nothing. The step's tools declaration always limits calls. Cancellation does not undo completed writes. MCP/JS audit logs record server/tool/status/duration, not arguments or full output.

### Local AI Skills

Explicitly install a reviewed package in `workflow-skills/<name>/SKILL.md` under the active data directory. No automatic downloads or updates. Names contain lowercase letters, digits and hyphens only. The YAML manifest needs a matching name and nonempty description:

```markdown
---
name: edge-web-form
description: Read a public demo form and fill a text box.
---

Instructions for the AI go here.
```

Optional `references/` contains up to 16 `.md`/`.txt` files; combined package size is at most 128 KiB. Links, nested resources and other file types are rejected. Preview validates packages and AI configuration. Package contents are frozen before execution/confirmation; later disk changes do not alter the authorized instructions. Custom MCP and standalone Skills show fingerprints; built-in Edge-only Skills automatically prepare authorization.

```json5
{ op: "invoke", id: "assistant", uses: "skill.run", saveAs: "answer",
  with: { skill: "edge-web-form", task: "Read the page and fill sample text",
    tools: [{ server: "browser.edge", tool: "browser_snapshot" }] } }
```

`skill.run` uses the enabled AI provider. `task` is user content; tools must satisfy service policy and per-run authorization. User Skill instructions, reference files, task text, catalog descriptions and schemas are not translated. The host's instructions use the selected AI language, while generated natural-language results use the UI language frozen at workflow start.

Budgets: 6 rounds, 8 calls, 180 seconds total, 256 KiB total context including protocol reasoning state, 32 KiB per tool result. Output tokens, temperature, reasoning effort and read-idle timeout follow the configured connection; no hidden per-request limit is added. Host revalidates the step allowlist and schema for each call. Tool failure is not retried. Streaming checks cancellation/deadline at event boundaries; blocking reads remain bounded by configured idle timeout. Logs include AI's brief action descriptions and tool/status/duration, not arguments or full tool output. `saveAs` stores final text.

Before running, the host reads actual schemas for authorized tools (at most 64 KiB, counted against context), reusing the run's MCP session. Reading schemas calls no tools, and metadata cannot expand permissions or change the task.

### Browser examples and guards

[Branch-to-browser example](examples/branch-to-browser/README.md): create/switch a branch, merge a source, assert the branch and fill its name through Chrome DevTools MCP using `new_page`, `take_snapshot`, `fill`. No clipboard or form submission. Git failure/conflicts stop before browser work.

`with.browserGuard` may specify a fixed HTTPS `url`, nonempty `contains` marker list and permitted `target`. Guarded Skills allow navigation, snapshot and typing only. The host rejects other URLs, unverified pages, wrong targets and typing without `submit: false`; a subsequent snapshot must contain the entered text. Mismatch/timeout stops the run.

[Edge demo templates](examples/workflow-edge-demo/) provide deterministic JS and AI Skill examples for the [public Selenium form](https://www.selenium.dev/selenium/web/web-form.html). They fill Text input without Submit/save. Allowlist: `browser_navigate`, `browser_snapshot`, `browser_type`. Install Edge and copy templates into workflows; download missing browser components when prompted. The client checks system Node and downloads Node/pinned Playwright MCP into the data directory if needed, then resumes authorization. System npm/npx is unnecessary. AI variant also needs the Skill package and enabled provider. Legacy unmodified `edgeDemo` configuration maps to the built-in service; customized entries remain unchanged. Public page changes may require revising the example.

### Defaults and inputs

`defaults.requireCleanWorktree` defaults to `true`: uncommitted changes stop execution.

| Input field | Default | Meaning |
| --- | --- | --- |
| `label` | variable name | Display label. |
| `description` | none | Help text. |
| `default` | empty | Initial string with interpolation. |
| `required` | `true` | Empty required values prevent preview/run. |
| `type` | `text` | `text`, `textarea`, `select`, `date`. |
| `options` | none | Select-only nonempty array of `{label, value}` with unique nonempty values. |

```json5
inputs: {
  summary: { type: "textarea", label: "Change summary", required: true },
  environment: { type: "select", label: "Environment", required: true,
    options: [{ label: "Validation", value: "uat" }, { label: "Production", value: "prod" }] },
  completionDate: { type: "date", label: "Target date", required: true },
}
```

Select passes value, not label; no default means no automatic first selection. Date must be valid `YYYY-MM-DD`. Multiline values preserve newlines. Defaults are expanded before validation. Invalid options, dates or missing required values stop before Git operations; preview and executor share validation. Values apply only to the current preview/run and are not written back. Inputs override same-name vars and cannot use reserved `git.`, `run.` or `date:` namespaces.

## Supported steps

All steps have `op`; string arguments support `${...}` interpolation.

| op | Fields and behavior |
| --- | --- |
| `checkout` | Required `branch`: switch to a local branch. |
| `fetch` | Optional `remote`: fetch refs. Default selected remote or `origin`. |
| `pull` | Optional `remote`: pull the current branch's upstream. Default selected remote or `origin`. |
| `createBranch` | Required `name`; optional `from` (HEAD), `checkout` (`true`). Create a local branch, optionally switch to it. |
| `guardRemoteBranch` | Required `branch` without remote prefix; `remote` defaults selected/origin, `fetch` true, `onExists` "fail", `onMissing` "continue". Both policies accept "fail"/"continue". |
| `merge` | Required `branch` (local/remote/reference): merge into current branch. Conflicts stop execution and remain for manual resolution. |
| `push` | Optional `remote` (selected/origin), `branch` (current at this step), `setUpstream` (true). Push local branch to same-name remote branch. |
| `ensureClean` | Stop on uncommitted changes, even if the default clean-worktree check was disabled. |
| `assertBranch` | Required `branch`: stop unless it matches the current branch. |
| `filterBranches` | Required `output`, `pattern`, `olderThanMonths`; optional `dateFormat` (`%Y%m%d`), `dateGroup` (`date`), `skipCurrent` (true). Read-only local-branch filtering with array output. |
| `deleteBranches` | Required `branches` (array expression or newline-separated string); `dryRun` true and `skipCurrent` true by default. Local deletion only. |

Use guardRemoteBranch before creating/pushing a branch; after an earlier fetch set `fetch: false`. To require existence use `onExists: "continue", onMissing: "fail"`. Preview infers current branch after checkout/createBranch with checkout true, but does not simulate fetch/pull/merge.

filterBranches matches a regular expression, reads the named dateGroup (falling back to the first capture), parses dateFormat and selects only branches whose calendar-month difference `(current.year - branch.year) * 12 + current.month - branch.month` is strictly greater than olderThanMonths (an interpolated integer string). Unparseable dates and branches below the threshold are logged as skipped, not output. Example regex `^${prefixAlt}_[^_]+_(?P<date>\\d{8})_` needs doubled backslashes in JSON5. Use `prefixAlt: "${prefixes|alt}"` with input `dev,uat,release`. deleteBranches consumes the array, or splits strings into trimmed nonempty lines. Keep dryRun true to inspect, then explicitly rerun with false; batch deletion refreshes once.

## Variables and expressions

String interpolation supports concatenation, e.g. `"release/${date:%Y%m%d}-${git.initialBranch}"`. Vars may reference other vars/built-ins; cycles such as a → b → a fail. Input values override vars.

| Variable | Meaning |
| --- | --- |
| `${date:%Y%m%d}` | Date at workflow start, using chrono formats. |
| `${date:%Y-%m-%d}` | Formatted date. |
| `${date:%Y%m%d-%H%M%S}` | Date/time; %Y year, %m month, %d day, %H 24h hour, %M minute, %S second. |
| `${run.id}` | Run ID based on start milliseconds. |
| `${run.startedAt:%Y%m%d}` | Formatted run start timestamp. |
| `${run.sourceBranch}` | Selected source branch. |
| `${run.sourceKind}` | `local` or `remote`. |
| `${git.initialBranch}` | Frozen initial/source branch. |
| `${git.currentBranch}` | Re-read before each step. |
| `${git.head}` | Current HEAD SHA. |
| `${git.repoName}` | Repository directory name. |

Detached HEAD may make branch variables unavailable and stops resolution.

### Pipeline methods

Methods run left to right inside the expression, never outside: `${prefixes|alt}` applies the method, `${prefixes}|alt` appends literal text. Separate arguments with `:`; quote arguments containing `:` or `|` (e.g. `replace:'|':'-'`). No nested `${...}` in method arguments, loops or conditions.

| String method | Meaning |
| --- | --- |
| `trim` | Strip surrounding whitespace. |
| `lower` / `upper` | Change case. |
| `replace:from:to` | Replace every literal occurrence. |
| `truncate:n` / `suffix:n` | First/last n characters. |
| `default:value` | Fallback for blank values. |
| `slug` | Lowercase, replace non-ASCII-alphanumeric characters with -, collapse repeated -. |
| `split:delimiter` | Split by a literal delimiter into an array. |
| `alt` | Convert comma-separated prefixes to a regex alternation, e.g. `(dev|uat|release)`. |

| Array method | Meaning |
| --- | --- |
| `compact` | Remove blank items. |
| `first[:default]` / `last[:default]` | First/last item, with optional empty-array fallback. |
| `nth:index[:default]` | Zero-based indexed item with optional fallback. |
| `join:delimiter` | Join into a string. |

Temporary arrays produced by split must become strings through first/last/nth/join. Step output arrays such as filterBranches' `${out.staleBranches}` retain array semantics when occupying the entire argument; use `${out.staleBranches|join:', '}` for a summary. Do not put pipeline methods outside `${...}`.

## Complete examples

### Create and push a dated release branch

```json5
{
  version: 1, name: "Create today's release branch",
  vars: { remote: "origin", base: "master", release: "release/${date:%Y%m%d}" },
  steps: [
    { op: "checkout", branch: "${base}" },
    { op: "pull", remote: "${remote}" },
    { op: "guardRemoteBranch", remote: "${remote}", branch: "${release}", fetch: false },
    { op: "createBranch", name: "${release}", from: "${base}", checkout: true },
    { op: "push", remote: "${remote}", branch: "${release}", setUpstream: true },
  ],
}
```

### Create A, merge B, assert and push

```json5
{
  version: 1, name: "Create A and merge B",
  inputs: { target: { label: "Target branch", default: "A" }, source: { label: "Merge branch", default: "B" } },
  vars: { remote: "origin", base: "master" },
  steps: [
    { op: "checkout", branch: "${base}" },
    { op: "guardRemoteBranch", remote: "${remote}", branch: "${target}" },
    { op: "createBranch", name: "${target}", from: "${base}", checkout: true },
    { op: "merge", branch: "${source}" },
    { op: "assertBranch", branch: "${target}" },
    { op: "push", remote: "${remote}", branch: "${target}", setUpstream: true },
  ],
}
```

### Temporary working branch

```json5
{
  version: 1, name: "Create temporary branch",
  vars: { branch: "tmp/${git.repoName}-${git.initialBranch}-${run.startedAt:%Y%m%d-%H%M%S}" },
  steps: [{ op: "ensureClean" }, { op: "createBranch", name: "${branch}", checkout: true }],
}
```

### Filter and dry-run stale local branches

```json5
{
  version: 1, name: "Clean stale named branches",
  defaults: { requireCleanWorktree: true },
  inputs: {
    prefixes: { label: "Prefixes (comma separated)", default: "dev,uat,release", required: true },
    months: { label: "Older than months", default: "3", required: true },
  },
  vars: { prefixAlt: "${prefixes|alt}" },
  steps: [
    { op: "filterBranches", output: "out.staleBranches",
      pattern: "^${prefixAlt}_[^_]+_(?P<date>\\d{8})_", dateFormat: "%Y%m%d", dateGroup: "date",
      olderThanMonths: "${months}", skipCurrent: true },
    { op: "deleteBranches", branches: "${out.staleBranches}", dryRun: true, skipCurrent: true },
  ],
}
```

Inspect matched branches in preview/logs before changing dryRun to false.

## Browser output cleanup and submit demo

Each built-in Edge session has an isolated `workflow-browser-sessions/` directory in the active data directory. The client tracks ownership of structural snapshots and screenshots, including explicit filenames. Every exit (success, failure, cancellation, connection failure or panic) closes MCP/background runtime before deleting owned files. Logs, downloads and other exports remain. Cleanup failure is reported and retried at startup, skipping live sessions.

The [release form example](examples/release-form/README.md) demonstrates four input types → create release branch → JS summary → fill and verify Edge form → submit exactly once. It uses a public test page and needs no AI configuration. Real systems need their own URL, field mapping and summary rules.

## Limitations and recommendations

- Sequential steps only: no conditions, loops, parallel execution or manual pauses.
- No arbitrary shell/Python; restricted JS is available only through V2 js.run.
- Active repository only; no cross-repository orchestration or line/hunk operations.
- Pipelines support string/temporary-array processing, not regex; regex is specific to filterBranches.pattern.
- Cancellation does not forcibly interrupt an in-flight remote Git operation or undo completed effects.
- Repository templates require explicit file selection; only the user template directory is auto-discovered.
- Keep requireCleanWorktree true, assert critical branch assumptions, include date/run time in generated names, and verify remotes/credentials before pushing.

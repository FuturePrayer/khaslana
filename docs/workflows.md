# Khaslana 工作流使用说明

Khaslana 工作流用于把一组 Git 操作按顺序自动执行，例如：基于 `master` 创建分支、合并另一个分支、再推送到远端。当前版本只支持 JSON5/JSONC 风格的结构化工作流文件，不支持 YAML、JavaScript、Python 或任意 shell 脚本。

## 如何运行

1. 打开目标仓库。
2. 点击顶部工具栏的“工作流”。
3. 点击“选择文件”，选择 `.json5` 或 `.jsonc` 工作流文件。
4. 在“步骤预览”中确认变量展开后的步骤。
5. 点击“运行”。
6. 在“运行日志”中查看每一步执行状态。

工作流始终作用于当前激活仓库。涉及远端认证时，继续使用 Khaslana 现有凭据机制和认证弹窗。

## 文件格式

工作流文件使用 JSON5，因此可以写注释、尾随逗号和未加引号的对象 key。

```json5
{
  version: 1,
  name: "基于 master 创建 A 并合并 B",

  defaults: {
    // 默认 true：运行前要求工作区干净
    requireCleanWorktree: true,
  },

  vars: {
    remote: "origin",
    base: "master",
    target: "A-${date:%Y%m%d}",
    source: "B",
  },

  steps: [
    { op: "checkout", branch: "${base}" },
    { op: "pull", remote: "${remote}" },
    { op: "createBranch", name: "${target}", from: "${base}", checkout: true },
    { op: "merge", branch: "${source}" },
    { op: "push", remote: "${remote}", branch: "${target}", setUpstream: true },
  ],
}
```

### 顶层字段

| 字段 | 必填 | 说明 |
| --- | --- | --- |
| `version` | 是 | 当前只支持 `1`。 |
| `name` | 否 | 工作流显示名称。为空时显示“未命名工作流”。 |
| `defaults` | 否 | 默认行为设置。 |
| `vars` | 否 | 用户自定义变量表，值必须是字符串。 |
| `steps` | 是 | 要顺序执行的步骤数组，至少需要一个步骤。 |

### defaults

| 字段 | 默认值 | 说明 |
| --- | --- | --- |
| `requireCleanWorktree` | `true` | 运行前检查工作区是否干净。若存在未提交更改，工作流会拒绝运行。 |

## 支持的步骤

所有步骤都使用 `op` 字段声明类型。字符串字段都支持 `${...}` 变量插值。

### checkout

切换到本地分支。

```json5
{ op: "checkout", branch: "master" }
```

字段：

| 字段 | 必填 | 说明 |
| --- | --- | --- |
| `branch` | 是 | 本地分支名。 |

### fetch

获取远端引用。

```json5
{ op: "fetch", remote: "origin" }
```

字段：

| 字段 | 必填 | 说明 |
| --- | --- | --- |
| `remote` | 否 | 远端名。省略时使用当前选中的远端；如果没有选中远端则使用 `origin`。 |

### pull

从远端拉取当前分支对应的上游分支。

```json5
{ op: "pull", remote: "origin" }
```

字段：

| 字段 | 必填 | 说明 |
| --- | --- | --- |
| `remote` | 否 | 远端名。省略时使用当前选中的远端；如果没有选中远端则使用 `origin`。 |

### createBranch

创建本地分支，可指定起点，并可创建后立即切换过去。

```json5
{ op: "createBranch", name: "feature/demo", from: "master", checkout: true }
```

字段：

| 字段 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `name` | 是 | 无 | 新分支名。 |
| `from` | 否 | 当前 `HEAD` | 起点分支或引用。 |
| `checkout` | 否 | `true` | 创建后是否切换到新分支。 |

### merge

把指定分支合并到当前分支。

```json5
{ op: "merge", branch: "B" }
```

字段：

| 字段 | 必填 | 说明 |
| --- | --- | --- |
| `branch` | 是 | 要合并进当前分支的本地分支、远端分支或引用。 |

如果合并产生冲突，工作流会停止，并保留冲突状态供用户处理。

### push

推送本地分支到远端同名分支。

```json5
{ op: "push", remote: "origin", branch: "feature/demo", setUpstream: true }
```

字段：

| 字段 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `remote` | 否 | 当前选中远端或 `origin` | 目标远端。 |
| `branch` | 否 | 当前分支 | 要推送的本地分支。 |
| `setUpstream` | 否 | `true` | 推送成功后是否设置 upstream。 |

### ensureClean

检查工作区是否干净。

```json5
{ op: "ensureClean" }
```

如果存在未提交更改，工作流会停止。即使 `defaults.requireCleanWorktree` 被设置为 `false`，也可以在关键步骤前手动插入这个检查。

### assertBranch

确认当前分支符合预期。

```json5
{ op: "assertBranch", branch: "release/${date:%Y%m%d}" }
```

字段：

| 字段 | 必填 | 说明 |
| --- | --- | --- |
| `branch` | 是 | 期望的当前分支名。 |

如果当前分支不一致，工作流会停止。

## 变量与字符串拼接

任意字符串字段都支持 `${...}` 插值。变量可以和普通文本拼接：

```json5
{
  vars: {
    target: "release/${date:%Y%m%d}-${git.initialBranch}",
  },
  steps: [
    { op: "createBranch", name: "${target}", from: "master" },
  ],
}
```

### 用户变量

用户变量定义在 `vars` 中：

```json5
vars: {
  base: "master",
  target: "feature/${git.repoName}-${date:%Y%m%d}",
}
```

使用方式：

```json5
{ op: "checkout", branch: "${base}" }
{ op: "createBranch", name: "${target}" }
```

用户变量可以引用其他变量或内置变量。循环引用会报错并停止解析，例如 `a -> b -> a`。

### 日期变量

| 写法 | 说明 | 示例结果 |
| --- | --- | --- |
| `${date:%Y%m%d}` | 工作流启动日期 | `20260609` |
| `${date:%Y-%m-%d}` | 年月日 | `2026-06-09` |
| `${date:%Y%m%d-%H%M%S}` | 日期和时间 | `20260609-142530` |

日期格式使用 Rust `chrono` 的格式语法。常用片段：

| 片段 | 含义 |
| --- | --- |
| `%Y` | 四位年份 |
| `%m` | 两位月份 |
| `%d` | 两位日期 |
| `%H` | 两位小时，24 小时制 |
| `%M` | 两位分钟 |
| `%S` | 两位秒 |

### 运行变量

| 变量 | 说明 |
| --- | --- |
| `${run.id}` | 本次运行 ID，基于启动时间毫秒。 |
| `${run.startedAt:%Y%m%d}` | 本次运行启动时间，可自定义日期格式。 |

### Git 变量

| 变量 | 说明 |
| --- | --- |
| `${git.initialBranch}` | 工作流开始时的当前分支。运行过程中不会变化。 |
| `${git.currentBranch}` | 每个步骤执行前读取到的当前分支。切换分支后会变化。 |
| `${git.head}` | 当前 `HEAD` 指向的提交 SHA。 |
| `${git.repoName}` | 当前仓库目录名。 |

注意：如果当前处于 detached HEAD，`${git.initialBranch}` 或 `${git.currentBranch}` 可能无法解析，工作流会报错。

## 完整示例

### 示例 1：从 master 拉取后创建发布分支

```json5
{
  version: 1,
  name: "创建当天发布分支",
  vars: {
    remote: "origin",
    base: "master",
    release: "release/${date:%Y%m%d}",
  },
  steps: [
    { op: "checkout", branch: "${base}" },
    { op: "pull", remote: "${remote}" },
    { op: "createBranch", name: "${release}", from: "${base}", checkout: true },
    { op: "push", remote: "${remote}", branch: "${release}", setUpstream: true },
  ],
}
```

### 示例 2：基于 master 创建 A，合并 B，再推送 A

```json5
{
  version: 1,
  name: "创建 A 并合并 B",
  vars: {
    remote: "origin",
    base: "master",
    target: "A",
    source: "B",
  },
  steps: [
    { op: "checkout", branch: "${base}" },
    { op: "createBranch", name: "${target}", from: "${base}", checkout: true },
    { op: "merge", branch: "${source}" },
    { op: "assertBranch", branch: "${target}" },
    { op: "push", remote: "${remote}", branch: "${target}", setUpstream: true },
  ],
}
```

### 示例 3：用仓库名和当前分支生成临时分支

```json5
{
  version: 1,
  name: "创建临时工作分支",
  vars: {
    branch: "tmp/${git.repoName}-${git.initialBranch}-${run.startedAt:%Y%m%d-%H%M%S}",
  },
  steps: [
    { op: "ensureClean" },
    { op: "createBranch", name: "${branch}", checkout: true },
  ],
}
```

## 当前限制

- 只支持顺序执行，不支持条件、循环、并发或手动暂停。
- 不支持执行 shell、JavaScript、Python 等脚本。
- 不支持跨仓库编排；工作流只作用于当前激活仓库。
- 不支持行级别或 hunk 级别操作。
- 取消运行目前只在步骤之间有意义；正在执行的 Git 远程操作不会被强制中断。
- 工作流文件不会自动保存到仓库，也不会自动发现；需要通过“工作流”弹窗手动选择。

## 建议

- 默认保持 `requireCleanWorktree: true`，避免自动流程覆盖或混入本地未提交修改。
- 对关键流程添加 `assertBranch`，防止工作流在意外分支上继续执行。
- 使用变量生成分支名时，优先包含日期或运行时间，避免重复分支名。
- 推送前先确认远端和凭据配置，远端步骤会使用 Khaslana 当前的远端和凭据机制。

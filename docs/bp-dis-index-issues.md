# khaslana-code-index 问题汇总报告（bp-dis 仓库）

本报告仅记录**实际执行的命令与观察到的输出**。未经验证的因果机制不写入（见 §8）。

---

## 1. 测试环境

| 项 | 值 |
|---|---|
| MCP 连接 | `khaslana-code-index` · stdio · `D:/khaslana/khaslana.exe mcp` |
| 被测仓库 | `c52d6094` / `D:\workspace\bp\bp-dis` |
| branch | `dev_lcc_20260821_新多元银保活动量系统对接` |
| 规模 | 1,545 文件（Java 1,515 = main 1,438 + test 77）；14,894 符号；18,275 节点；27,400 边 |
| 索引库 | `index.db` 25.99 MB，写入于 2026/10/10 10:48:27 |
| generation | `5489b1249bb25a8d33e7ee3d20540251` |
| 构建 | Maven，`pom.xml` 依赖外部模块 `com.cathay:api`、`com.cathay:common`（父工程 `BMPIA-parent`） |

### 索引自报统计

```json
{"calls":470, "edges":27400, "symbols":14894,
 "unresolved_calls":36354, "unresolved_no_candidate":29534, "unresolved_with_candidates":6820,
 "coverage":{"indexed":1515,"unsupported":30}}
```

`CALLS` 边仅 **470** 条，占全部边（27,400）的 1.7%。

### 测量方法

- 索引侧：`trace_path`、`search_symbols`、`check_index_coverage`、`get_architecture`、`index_status` 的返回值。
- 源码侧基准：自建 Node 脚本独立解析 Java 声明与限定调用（`Class.method(`），作为对照组。
- 性能：索引侧通过直接驱动 MCP stdio JSON-RPC 协议计时（5 次取均值）；grep 侧用 `Select-String` 计时（预热后 5 次取均值）。
- token 估算：`tokens = CJK 字符数 × 1 + ASCII 字符数 ÷ 4`。**为估算值，非真实分词器输出。**

---

## 2. 问题汇总

| 编号 | 问题 | 严重度依据 |
|---|---|---|
| **Q1** | **重载方法的所有调用边丢失** | `STRING.emptyIf`：实测 225 处调用，`trace_path` 返回 0 |
| **Q2** | `CALLS` 边总量异常偏低 | 470 条边 / 14,894 符号 = 0.032（对照仓库为 0.53、1.06） |
| **Q3** | 索引 token 开销高于 grep | 同一问题 3,604 tok vs 740 tok（4.9×） |
| **Q4** | 真实会话中模型 0 次使用索引 | 34 次工具调用中索引工具 0 次 |
| **Q5** | 性能优势无法转化为实际收益 | 索引 9.7 ms vs grep 470 ms（48×），但见 Q4 |

---

## 3. Q1：重载方法调用边丢失（本次新发现）

### 3.1 决定性证据

`STRING.emptyIf` 在 `STRING.java` 中有**两个重载**：

```java
L10: public static String emptyIf(String source, String defaultValue) {
L28: public static String emptyIf(Object source, String defaultValue) {
```

`search_symbols` 正确识别出两个重载（`emptyIf` 与 `emptyIf#2`）。但对两者分别执行 `trace_path(inbound)`：

```json
{"function":"emptyIf","qualified_name":"...STRING.java.STRING.emptyIf","callers_total":0}
{"function":"emptyIf","qualified_name":"...STRING.java.STRING.emptyIf#2","callers_total":0}
```

**实测调用点（文件系统检索）**：

```
$ Select-String -Pattern "STRING\s*\.\s*emptyIf\s*\(" （递归全部 .java）
  total matches: 214
  distinct files: 9
```

按文件分布：

| 文件 | 调用数 |
|---|---|
| `PerformanceServiceImpl.java` | 77 |
| `InsureClaimServiceImpl.java` | 74 |
| `RecordForAppServiceImpl.java` | 35 |
| `OfficialWebsiteQueryServiceImpl.java` | 16 |
| `OfficialWebsiteRosterCheckController.java` | 5 |
| `AgentModelServiceImpl.java` | 3 |
| `ActivityGeneralTwoServiceImpl.java` | 2 |
| `OfficialWebsiteInsuranceController.java` | 1 |
| `SerialNumServiceImpl.java` | 1 |
| **合计** | **214** |

调用方显式导入该类：

```java
import com.cathay.dis.util.STRING;
```

调用实例：

```java
PerformanceServiceImpl.java:167
  noSaleSum = DOUBLE.add(Double.parseDouble(STRING.emptyIf(htadn001.getNoSale(), "0")), noSaleSum);
OfficialWebsiteInsuranceController.java:128
  type = STRING.emptyIf(type,"1");
```

**索引报告 0，实际 214。**

### 3.2 对照实验：2×2 隔离变量

为区分「嵌套调用位置」与「方法重载」两个假设，选取四类方法对照：

| 方法 | 重载数 | 嵌套调用占比 | 实测调用数 | 索引报 | 结论 |
|---|---|---|---|---|---|
| `PositionEnum.getEnum` | 1 | 0% | 10 | **4**（callers_total） | 建边 |
| `WorkDayCountUtils.dateCompare` | 1 | 100% | 6 | **3** | 建边 |
| `OfficeUtil.mapToVo` | 1 | 0% | 8 | **8** | 建边 |
| `DateUtils.getDateMap` | 1 | 0% | 5 | **5** | 建边 |
| **`STRING.emptyIf`** | **2** | 81% | **214** | **0** | **丢边** |
| **`OfficeUtil.mapKeyToCamel`** | **2** | 100% | **4** | **0** | **丢边** |
| **`OfficeUtil.mapObjToStr`** | **2** | 100% | **4** | **0** | **丢边** |
| **`HttpClientUtil.sendPost`** | **2** | — | **1** | **0** | **丢边** |

**嵌套假设被推翻**：`WorkDayCountUtils.dateCompare` 的 6 处调用**全部是嵌套形式**，索引仍建边成功（返回 3 个 caller，`call_sites` 行号 794/801/812 与 849/860/871）。

```
源码（PerformanceServiceImpl.java）:
  L794: ... WorkDayCountUtils.dateCompare(...) ...
索引返回: callers_total=3, call_sites.lines=[794,801,812] + [849,860,871]
```

**重载假设成立**：4 个重载方法（`emptyIf` 2 个定义、`mapKeyToCamel` 2 个、`mapObjToStr` 2 个、`sendPost` 2 个）**无一例外全部返回 0**；4 个单定义方法**无一例外全部建边**。

### 3.3 全仓库扫描：重载 vs 单定义

对全仓库「仓库内类的 public/protected static 方法」分类：

```
single-definition methods : 14   (total call sites 53)
OVERLOADED methods        :  2   (total call sites 226)
```

**重载方法清单（全部）：**

| 方法 | 定义数 | 实测调用数 | 索引 callers_total |
|---|---|---|---|
| `STRING.emptyIf` | 2 | 225 | **0** |
| `HttpClientUtil.sendPost` | 2 | 1 | **0** |

**单定义方法对照（有调用边的）：**

| 方法 | 定义数 | 实测调用数 | 索引 callers_total |
|---|---|---|---|
| `PositionEnum.getEnum` | 1 | 10 | 4 |
| `AssignEnum.getEnum` | 1 | 6 | — |
| `WorkDayCountUtils.dateCompare` | 1 | 6 | 3 |
| `DateUtils.getDateMap` | 1 | 5 | 5 |
| `PosClassEnum.getEnum` | 1 | 4 | — |
| `EffDateTpEnum.getName` | 1 | 4 | — |
| `PositionEnum.getDescByCode` | 1 | 4 | — |
| `PolCheckIndEnum.getName` | 1 | 3 | — |
| `SchoolDegreeEnum.getEnum` | 1 | 3 | — |

### 3.4 规模影响

被测静态方法中，重载方法占调用总量的 **226 / 279 = 81%**。即：在这批样本里，**绝大多数调用点落在会丢边的方法上**。

---

## 4. Q2：CALLS 边总量偏低

| 仓库 | 符号 | CALLS 边 | calls/symbol |
|---|---|---|---|
| `bcs_utf8` (Java) | 760,242 | 263,288 | 0.35 |
| `deepseek-harness` (TS) | 40,196 | 21,225 | 0.53 |
| **`bp-dis` (Java)** | **14,894** | **470** | **0.032** |

`bp-dis` 的比值比对照仓库低 11–17 倍。

同时 `check_index_coverage` 对单文件报告：

| 文件 | `call_sites` | `unresolved_calls` | 比例 |
|---|---|---|---|
| `AgentModelServiceImpl.java` | 3,429 | 3,331 | 97% |
| `OfficeUtil.java` | 53 | 49 | 92% |
| `DateUtils.java` | 7 | 7 | 100% |

`unresolved_calls` 分为两类（索引自报）：`unresolved_no_candidate` 29,534（无同语言仓库定义，可能为外部库）、`unresolved_with_candidates` 6,820（有候选但目标不确定）。

该仓库大量调用外部模块（`com.cathay:api`、`com.cathay:common`）与第三方库（hutool、fastjson、Spring），因此高 no_candidate 值与依赖结构相符。但 §3 的丢边案例（`STRING`、`OfficeUtil`、`HttpClientUtil` 均为仓库内类）**不属于外部库**。

---

## 5. Q3：Token 消耗 —— grep 更省

### 5.1 模型可见成本（关键区分）

MCP 原始报文含 52% 的 JSON 转义开销（`content[0].text` 被作为 JSON 字符串嵌入），但该开销**不进入模型上下文**。依据源码 `packages/mcp/mcp-client/src/tools.ts:286-289`：

```ts
render(_args: unknown, value: JsonValue) {
  const result = value as unknown as McpResult
  return [{ type: 'text', text: extractText(result.content, rawName) }]
}
```

仅返回 text 块；`structuredContent` 与转义外壳不进入模型。实测同一批 8 次调用：

| | 原始报文 | 模型可见 |
|---|---|---|
| 8 次调用合计 | 15,078 tok | **7,212 tok** |

### 5.2 同一问题的索引 vs grep（模型可见 token）

| 问题 | 索引 | grep | 差值 |
|---|---|---|---|
| `OfficeUtil.mapToVo` 的调用者（8 处） | 926 | **283** | grep 省 643 |
| `DateUtils.getDateMap` 的调用者（5 处） | 756 | **137** | grep 省 619 |
| `mapToVo` 的定义 | 1,922 | **320** | grep 省 1,602 |
| **合计** | **3,604** | **740** | **grep 省 2,864（4.9×）** |

单次调用明细（模型可见）：

| 工具调用 | 模型可见 tok |
|---|---|
| `search_symbols.mapToVo` | 1,922 |
| `search_graph`（query="Service"） | 1,642 |
| `trace_path.mapToVo` | 926 |
| `get_architecture` | 763 |
| `trace_path.getDateMap` | 756 |
| `list_projects` | 651 |
| `get_code_snippet.mapToVo` | 280 |
| `index_status` | 272 |

### 5.3 开销构成分解（`trace_path.mapToVo`，模型可见 926 tok）

```
inner JSON 总量                    3,526 chars
  ├─ callers[] 数组                3,157 chars
  │    ├─ qualified_name × 8         848 chars  ~212 tok
  │    ├─ file_path × 8              511 chars  ~128 tok
  │    ├─ name × 8                   130 chars  ~ 33 tok
  │    └─ JSON 键与结构              216 chars  ~ 54 tok
  └─ coverage_note（固定样板）        88 chars  ~ 66 tok
```

`qualified_name` 每条平均 106 字符，且**内嵌完整文件路径**：

```
name          : getNewDocs
qualified_name: bp-dis.src.main.java.com.cathay.dis.service.impl.DocServiceImpl.java.DocServiceImpl.getNewDocs
file_path     : src/main/java/com/cathay/dis/service/impl/DocServiceImpl.java
```

`coverage_note`（66 tok / 每次调用）为固定文案，每次返回均重复。

### 5.4 常驻开销

| 项 | 模型可见 tok |
|---|---|
| `khaslana` 11 个工具 schema | 2,243 |
| 全部 68 个工具 schema | 13,844 |

（工具描述已由中文改为英文；此前 11 个工具 schema 为 3,527 tok。）

---

## 6. Q4：真实会话中的索引使用情况

独立 subagent（全新上下文），任务：修改 `OfficeUtil.mapToVo` 以支持嵌套字段映射，调查定义、调用点与改动面。

### 工具调用实测

| 项 | 值 |
|---|---|
| **索引工具调用** | **0 次** |
| 总工具调用 | 34 次（grep / glob / read / pwsh） |
| 命中的调用点 | 8 处，与索引返回的 8 处完全一致 |
| 失败尝试 | 2 次（花括号 glob 不生效；在错误模块查找类声明） |

该 subagent 自述：

> 我全程没有使用 MCP 代码索引工具（`mcp__khaslana-code-index__*` 一次都没调用），也没有用 `list_projects` 建索引——上面的定位完全靠 grep/glob/read 完成。

### 有效检索路径（前 12 步）

```
1.  grep  "mapToVo"                                   → 9 命中
2.  glob  **/OfficeUtil*.java                         → 1 文件
3.  read  OfficeUtil.java                             → 136 行
4.  grep  "class OfficeUtil|OfficeUtil\."             → 9 命中
5.  read  DocServiceImpl.java                         → 110 行
6.  grep  辅助方法名                                   → 20 命中
7.  glob  {6个VO}                                     → 失败（brace 不生效）
8.  read  RoleServiceImpl.java
9.  glob  **/vo/**/*.java                             → 7 文件
10. grep  class (VO 名)                                → 无命中
11. pwsh  列目录                                       → 确证 VO 不在本模块
12. read  PersonnelServiceImpl.java
```

第 11 步后转向 `D:\workspace\bp` 多模块工作区，在兄弟模块 `bp-api` 中找到 6 个 VO。

---

## 7. Q5：性能对比

### 7.1 索引侧（MCP 端到端往返，5 次均值）

| 工具 | 均值 | min | max | 报文大小 |
|---|---|---|---|---|
| `trace_path` | **9.7 ms** | 8.6 | 11.6 | 7,382 B |
| `get_code_snippet` | 10.7 ms | 7.9 | 12.8 | 2,378 B |
| `index_status` | 13.8 ms | 12.6 | 15.7 | 1,654 B |
| `search_symbols` | 15.3 ms | 11.0 | 22.6 | 15,246 B |
| `get_architecture` | 62.3 ms | 38.4 | 146.0 | 6,364 B |
| `list_projects` | 427.3 ms | 266.1 | 880.5 | 3,966 B |

协议握手：`initialize` 41 ms，`tools/list` 1 ms。

### 7.2 grep 侧（`Select-String`，预热后 5 次均值）

| 模式 | 均值 | min | max |
|---|---|---|---|
| `mapToVo` | 559 ms | 518 | 583 |
| `getDateMap` | 420 ms | 385 | 479 |
| `buildAgentInfo` | 435 ms | 401 | 520 |

底层文件扫描上限（枚举 + 读取全部 1,515 个 `.java`）：

```
pass 1: 461 ms
pass 2: 400 ms
pass 3: 359 ms
avg   : 407 ms
```

### 7.3 结论

单次调用索引比 grep 快约 **48 倍**（9.7 ms vs 470 ms）。但该优势未在真实会话中体现（§6：索引使用 0 次）。

---

## 8. 未验证事项

1. **重载导致丢边的实现机制未验证。** 本报告仅确认「重载方法返回 0、单定义方法正常建边」这一相关性（§3.2），未检查索引实现代码以确定成因。
2. **嵌套调用与丢边无关（已验证）。** 该假设曾被提出，经 `WorkDayCountUtils.dateCompare`（100% 嵌套、成功建边）推翻。
3. **`CALLS` 边偏低的完整原因未确定。** §4 给出比值与 `unresolved` 分类，但未逐条追查。
4. **重载丢边的全仓库影响面未穷尽。** §3.3 扫描的是「仓库内类的 public/protected static 方法」（16 个有调用点的方法），未覆盖实例方法、非 public 方法、枚举与接口方法。
5. **token 为估算值**，未用真实分词器校验。
6. **本次未做多轮重复。** 性能为单次会话内的 5 次采样；行为测试为 1 个 subagent 样本。
7. **`bp-dis` 依赖的外部模块**（`com.cathay:api`、`com.cathay:common`）不在索引范围内，其调用必然计入 `unresolved_no_candidate`，这不构成缺陷。

---

## 9. 复现命令

### 9.1 索引侧

```
trace_path  { repo:"c52d6094", function_name:"bp-dis.src.main.java.com.cathay.dis.util.STRING.java.STRING.emptyIf", direction:"inbound" }
            → callers_total: 0

trace_path  { repo:"c52d6094", function_name:"...STRING.java.STRING.emptyIf#2", direction:"inbound" }
            → callers_total: 0

trace_path  { repo:"c52d6094", function_name:"...WorkDayCountUtils.java.WorkDayCountUtils.dateCompare", direction:"inbound" }
            → callers_total: 3, call_sites.lines=[794,801,812]
```

### 9.2 源码侧基准（PowerShell）

```powershell
$root = "D:\workspace\bp\bp-dis"

# 重载丢边：实测调用数
(Get-ChildItem $root -Recurse -File -Filter *.java |
  Select-String -Pattern "STRING\s*\.\s*emptyIf\s*\(").Count
# → 214

# 按文件分布
Get-ChildItem $root -Recurse -File -Filter *.java |
  Select-String -Pattern "STRING\s*\.\s*emptyIf\s*\(" |
  Group-Object { [System.IO.Path]::GetFileName($_.Path) } |
  Sort-Object Count -Descending

# 重载定义
Select-String -LiteralPath "$root\src\main\java\com\cathay\dis\util\STRING.java" `
  -Pattern "public static.*emptyIf"
# → L10: emptyIf(String, String)
# → L28: emptyIf(Object, String)
```

### 9.3 性能计时

```powershell
# grep 侧
$sw = [System.Diagnostics.Stopwatch]::StartNew()
Get-ChildItem $root -Recurse -File -Filter *.java |
  Select-String -Pattern "mapToVo" -EA SilentlyContinue
$sw.Stop()
```

索引侧：以 JSON-RPC 直接驱动 `khaslana.exe mcp` 的 stdin/stdout，对 `tools/call` 计时。

---

*报告撰写期间索引 `index_status` 返回 `indexing / refresh.phase=running`。测试过程中连接曾被停用一次（`配置 / disabled`），恢复后数据一致。*

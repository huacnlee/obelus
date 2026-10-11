# 快捷键与查找交互开发交接

更新时间：2026-10-11。本文供接手开发的 agent 使用。

## 用户确认的范围

1. 新增 **VS Code** 与 **TextMate** keymap，保留原本的方案。
2. VS Code keymap **不使用 F 键**。当前草稿也使 TextMate 不使用 F 键。
3. 支持 macOS **Cmd**，提供配置命令和 welcome 选择 keymap 界面。
4. 改进快速查找文件、搜索文件，借鉴 LazyVim 的查找交互。
5. **暂时不要 LazyVim keymap，不增加 Vim 模式，不实现 leader。** 用户已明确否定 `Alt+Space`、`Ctrl+k` 作为 leader 的提议。
6. 用户授权并行多 agents 开发。各 agent 共用工作区，划清文件所有权后再修改。

Obelus 是贴合一个人工作习惯、内置 agent 的编辑器；不是追求编辑器功能清单的产品。此次改善入口、熟悉感和可发现性，不增加窗格、多光标或 Vim 编辑语法。

## 当前工作区：已有未提交草稿

用户要求交接时已停止各 agent 的实现工作。以下修改留在工作区，**并非已交付实现**。没有提交、推送或创建 PR，没有改动用户的真实配置。接手先读 `git status --short`、`git diff`，不要盲目重做或清空这些修改。

| 文件 | 草稿内容 | 状态 |
| --- | --- | --- |
| `crates/obelus-keymap/src/lib.rs` | `Layout::VsCode/TextMate`、Cmd 解析与显示、Shift+Ctrl 字母、新预设与终端备用键 | 相关测试通过；说明与破坏验证未完成 |
| `crates/obelus-keymap/tests/keymap.rs` | 新预设和 Cmd 的测试、调整旧 SUPER 预期 | 尚未执行故意破坏验证 |
| `crates/obelus-keymap/Cargo.toml`、`Cargo.lock` | 增加 `obelus-config` 依赖，识别前端 | 随预设草稿产生 |
| `crates/obelus-editing/src/lib.rs` | `BINDABLE_MODIFIERS` 接受 SUPER，仍精确匹配修饰键 | 旧模块说明未更新 |
| `crates/obelus-config/src/lib.rs` | 四种预设、首次启动判定、保存预设 helper、显式保存 `obelus` | 配置测试和故意破坏验证已执行 |
| `crates/obelus-app/src/startup.rs` | 导出 `pub mod keys` | 未整体编译验证 |
| `crates/obelus-app/src/startup/keys.rs` | 新文件，共享 CLI 查询和切换逻辑，输出 stderr | 未测试 |
| `crates/obelus-cli/src/main.rs` | `ob keys` / `ob keys set <preset>` | 未编译、未测试 |
| `crates/obelus-gui/src/main.rs` | 相同的 `obg keys` 子命令 | 未编译、未测试 |
| `crates/obelus-component/src/window.rs` | `Move::in_picker` 导航别名 | 未测试 |
| `crates/obelus-component/src/picker/mod.rs` | 选择列表与只读列表调用上述别名 | 未测试 |

**尚未实现**：welcome keymap 选择、应用内 `choose-keymap` 命令、选择器说明与名称更新、查找导航帮助提示、站点/README 文档、完整验证。

## 预设与兼容策略

配置继续用 `keys_from`，不建立第二套配置来源：

```toml
keys_from = "vscode"

[keys]
open-file = "Ctrl+o"
```

预设值：`obelus`、`mnemonic`、`vscode`、`textmate`。`mnemonic` 是既有可选项，保留兼容，不擅自删除。

四个预设写在 `crates/obelus-keymap/keymaps/*.json`（扩展名是 `.json`，按 JSONC 解析：可写 `//` 注释，允许末尾逗号），由 `include_str!` 编进二进制，首次用到时解析并缓存（`src/shipped.rs`）。`obelus.json` 是完整的表；其余用 `extends` 继承它，`move` 把命令原有的键全部换成新键（保留原上下文），`bindings` 逐条追加。命令名与键的写法和设置文件的 `[keys]` 一致，键不区分大小写（`ctrl+t` 与 `Ctrl+T` 相同，Shift 要写成 `shift+`），文件里统一用全小写。改预设只改 JSON，不动 Rust；用户不能另加预设文件，自定义仍走 `[keys]`。

- 旧配置没有 `keys_from` 时保持原先的 `obelus` 行为。
- 切换预设保留 `[keys]` 自定义覆盖；应让界面说明覆盖仍有效。
- macOS 窗口的新预设使用 Cmd；其他系统与终端使用 Ctrl。
- 窗口/增强键盘协议可提供 Ctrl+Shift 常用键，传统终端另有可达的 Alt 备用键。
- 不掩掉额外修饰键：`Ctrl+Super+q` 不能被解释成 `Ctrl+q`。
- 不把 Linux/Windows 的 Super 无条件转成 Ctrl，它通常是桌面快捷键。

草稿 API：`Keymap::laid_out_for(layout, command_key: bool)` 可明确构造 Cmd 或 Ctrl 版本，便于测试；`laid_out` 依据 macOS 与 `obelus_config::in_a_window()` 选择。

### 当前草稿的主要映射

以下是已经写入草稿的映射，仍需要审查冲突、上下文和提示；不是要求原样冻结。

| 功能 | VS Code | TextMate |
| --- | --- | --- |
| 打开文件 | 主修饰键+p | 主修饰键+t |
| 命令面板 | 主修饰键+Shift+p；Alt+Shift+p 备用 | 同左 |
| 文件内搜索 | 主修饰键+f | 同左 |
| 项目搜索 | 主修饰键+Shift+f；Alt+/ 备用 | 同左 |
| 当前文件符号 | 主修饰键+Shift+o；Alt+o 备用 | 主修饰键+Shift+t；Alt+o 备用 |
| 已打开文档 | Alt+e | 同左 |
| 改动文件 | Alt+g | 同左 |
| 会话列表 | Alt+Shift+n | 同左 |
| 文件／项目历史 | Alt+h / Alt+Shift+h | 同左 |
| 跳转定义 | Alt+. | 同左 |
| Pull requests / Issues | Alt+u | 同左 |

“主修饰键”是 macOS 窗口的 Cmd，其他环境的 Ctrl。代码的字母规范化把 Shift 字母写成大写，因此 `Ctrl+P` 表示 Ctrl+Shift+p，不是 Ctrl+p。注意现有显示方式可能不符合迁移用户的直觉；若调整显示，必须保持配置解析往返一致与旧文件兼容。

草稿把原有 Ctrl 绑定转换为 Cmd 后再叠加新映射。需重点检查 TextMate 的 Cmd+t 与原 `TodoOpen`、VS Code 的 Ctrl+p 与原命令面板、终端上下文的退出/粘贴/关闭，以及被移除原绑定后是否漏掉必要上下文。部分冲突已在测试中发现并修复，但不能据此认为全部正确。

## welcome 与应用内选择：建议实现路径

复用现有选择器，不新建另一套列表绘制、导航或滚动机制。

1. 在 `obelus-command` 增加 `Command::KeymapSelect`，命令名 `choose-keymap`，分类 Obelus，要求 `Requires::Nothing`。
2. 补齐所有穷尽匹配：command 元数据、`drawn`、`opens_a_list`、`opens_a_view`、`group`、`requires` 等，以及 `obelus-icons::for_command` 与 `app::dispatch`。dispatch 不得加入 wildcard。
3. 实现 `App::open_keymap_picker`，复用 `app/project/preferences.rs` 的 `open_choices("keys_from", ...)` 和 `PickerValue::Setting`。可为专用入口设置 opener、标题/占位文本。
4. 当前 `open_choices` 只解释 `classic/mnemonic`，`called` 只显示 Obelus/Mnemonic。增加 VS Code、TextMate 的显示名称、简短说明，最好显示文件与命令面板的示例键位。
5. 在 `obelus-ui/src/welcome.rs` 增加“Choose a keymap”入口。候选单键为 `Alt+k`：用户拒绝的是 leader，不是切换预设的命令键。该键尚未加入；检查冲突后实现。
6. 所有提示从 App 的键表读取，不能硬编码预设中的实际按键。
7. 只有全局配置文件不存在时自动显示首次选择。已有配置（即使没有 `keys_from`）不重复 onboarding；不可读文件不触发会覆盖它的设置流程。
8. 首次选择可以 Esc 放弃，继续使用原默认键位；无需制造无法退出的 onboarding。
9. 选中后立刻应用并保存。选择 `obelus` 也要记录，避免每次启动又询问。写入失败应沿用现有状态提示并保留原文件。

配置草稿提供 `Reading::keymap_is_chosen()`：只有 `Reading::Nothing` 返回 false，已有 Settings、Unreadable、Nowhere 都返回 true。

### 启动顺序必须核对

- `startup::build`：定位项目 → `App::new` → `load_config` → `reopen_what_was_open`。
- `App::start` 位于 `app/lifecycle.rs`：建立事件通道、启动 watcher/server、报告启动问题，最后按 `list_at_start` 打开文件选择器。
- **不要在 load_config 中直接打开首次选择，然后被 start 的文件列表覆盖。** 建议保存首次选择的需求，在生命周期的正确位置显示。
- 启动问题列表优先；项目 chooser、文件打开、重开文档、headless 都要明确行为。
- headless 不显示选择界面。
- 测试通过 `config_file_for_test` 指向临时路径，不能写真实 `~/.config/obelus`。

## CLI：补完与验证

草稿使用 clap 原生子命令，两套前端共享 `startup::keys::run`：

```bash
ob keys
ob keys set vscode
ob keys set textmate
ob keys set obelus
obg keys set vscode
```

- 查询显示当前预设与可选项；切换只保存全局设置，不启动编辑器。
- stdout 是绘制面，CLI 信息使用 stderr。
- 检查 `#[arg(value_parser = obelus_config::KEY_LAYOUTS)]` 是否编译；必要时使用 `PossibleValuesParser::new`。
- 为两个 binary 测试解析、帮助、非法预设、无窗口早退出，以及路径和子命令的歧义。需要打开名为 keys 的路径时验证 `./keys` 或 `--`。
- `write_keymap(path, preset)` 验证预设、读取最新配置、拒绝不可读文件、通过 `save_to` 原子替换。
- watcher 应把一处切换传到其他窗口，沿用已有配置变更事件；不能轮询。

**待审查的持久化问题**：现有 `save_to` 原子替换保证文件不出现半写状态，但 read/modify/write 没有跨进程锁。新 helper 仍存在并发更新丢失的窗口；不能声称已解决并发合并。若修复，要协调所有配置写入路径，仅给 CLI 加锁不能解决其他 writer 的竞态。当前 helper 会规范化已知默认字段，保留语义和未知内容，不保证逐字节保留全部原配置。

## 快速查找／搜索：已选定的增量

现有实现已经有模糊文件筛选、实时搜索、预览、切换搜索范围保留 query。先复用并完善这些机制，不重复造一个 Telescope 界面。

当前导航草稿通过 `Move::in_picker` 增加：

| 按键 | 行为 |
| --- | --- |
| Ctrl+n / Ctrl+p | 下一项／上一项 |
| Ctrl+d / Ctrl+u | 下一页／上一页 |
| Enter | 选择并跳转 |
| Esc | 放弃当前列表 |

- query 持续可输入，裸 n/p/d/u 仍然是搜索文本。
- 精确判断修饰键，不接管 Ctrl+Shift、Super 等其他组合。
- picker 内的 Ctrl+p 优先导航，不打开全局命令面板或文件查找；退出 picker 后恢复预设含义。
- 列表移动复用 Window/Move，保留禁用行跳过、分页边界与正在填充列表的原规则。
- 当前 Ctrl+d/u 是**整页**，不是 half-page；帮助与文档不能描述成半页。
- 只读列表也接收这些别名，需审查是否符合其语义。
- 给文件/搜索 picker 的按键帮助补充导航提示，确保窄窗口不会挤掉关键操作。
- 导航归组件，不新增 `cursor-up` 等命令，不影响编辑器、消息输入框或子进程终端。

此部分只完成导航代码草稿，还没有测试或提示。文件 `:line` 跳转等扩展未实现，也未纳入本次已确认的增量，避免未经验证扩大范围。

## 实际验证记录

这些是各 agent 在交接时报告的记录，不代表整个 workspace 已验证。

- keymap/editing：`cargo test --offline -p obelus-keymap -p obelus-editing` 通过（6 个 editing、7 个 keymap 单元、18 个 keymap 集成测试）。新增测试**未**故意破坏验证；相关注释只是计划，接手必须执行后才能保留为事实。
- config：`cargo test -p obelus-config` 通过 37 个测试与 doc tests。随后 5 次故意破坏均出现预期失败：删新预设、略去默认 keys_from、把已有 Settings 视为未选择、helper 忽略已有配置、helper 不验证预设。源码已恢复，尚未重新跑恢复后的测试。
- CLI 与 picker：未编译验证、未新增测试、未执行故意破坏。
- 全项目：未运行格式化、完整 build/test/clippy。
- 停止时没有遗留的故意破坏修改。

### 必须补的行为测试

- 每个新预设无 F 键；已知键执行预期动作；Cmd/Ctrl/Shift 精确匹配；额外修饰键不误执行。
- macOS Cmd 的 GUI 翻译、输入法/布局处理，以及终端 Ctrl 备用键实际可达。
- keymap 配置往返、自定义覆盖保留、默认 obelus 兼容，旧值 `classic` 读作 `obelus`、不可读配置拒绝覆盖。
- welcome 的首次显示、旧用户跳过、Esc、选中保存、重启不重复、窄屏、启动问题与项目选择的优先级。
- choose-keymap 在命令面板可发现，切换后 welcome/命令提示与行为同步。
- picker Ctrl+n/p/d/u 的选择、query 不丢失、裸字母输入、禁用行与分页边界，以及不同预设下的上下文优先级。
- 两个 binary 的 CLI parser 与切换操作。

**每个新增测试必须故意破坏对应实现，观察失败再恢复，并在注释写明实际做过的破坏。** 不能用实现本身计算 expected 来冒充行为验证。

遵循根 `AGENTS.md`，最终检查使用：

```bash
cargo +nightly fmt --all
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-features --all-targets
```

需要 fixture 更新时先看差异，再运行 `UPDATE_FIXTURES=1 cargo test --workspace`；不要把环境路径或偶然 git 状态写进 fixture。Clippy 必须无警告。跨平台 CI 通过前不宣称 Windows/macOS 都验证过，任何合入 master 都走 PR。

## 建议接手顺序

1. 读 AGENTS、本文与现有 diff，建立分工；先确认草稿编译并修正明显冲突和过时注释。
2. 完成 choose-keymap、welcome 与生命周期集成；把一次配置选择串通 CLI/UI/watcher。
3. 补完查找导航提示与行为测试，核对新预设的直达入口。
4. 对新增测试执行故意破坏验证；格式化与 workspace 检查。
5. 更新 README、`docs/reference.html`、`docs/zh/reference.html` 中的预设、CLI 和终端键位说明。
6. 用实际键盘检查窗口、传统终端和增强协议终端，汇报实际验证范围与剩余限制。

## 官方参考

- [VS Code 默认快捷键](https://code.visualstudio.com/docs/reference/default-keybindings)
- [VS Code 快捷键配置与连续组合键](https://code.visualstudio.com/docs/configure/keybindings)
- [TextMate 文件打开](https://macromates.com/textmate/manual/opening-files)
- [TextMate 键位设计说明](https://macromates.com/manual/ja/key_bindings.html)
- [LazyVim 键位与查找入口](https://www.lazyvim.org/keymaps)（仅参考交互，不加入该 keymap 或 Vim 模式）

Obelus 扩展映射要标明是适配，不宣称完整复制其他编辑器。无 F 键、传统终端可达和用户自定义兼容优先于逐键照搬。

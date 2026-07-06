# JS 全量迁移 TypeScript 执行计划

> 本文档是临时执行文档，迁移完成并验收通过后由用户删除。执行期间若本文档与 `AGENTS.md`、`README.md` 或其他文档冲突，以本文档为准。

## 1. 唯一目标与边界

唯一目标：将仓库内可迁移的 JavaScript 源码、测试和配置迁移为 TypeScript，并保证原有功能与已修复问题不回退。

本轮不考虑向后兼容和数据迁移；不新增功能；不重构 UI、业务流程或 Rust 架构；不因为类型迁移顺手改交互、样式、文案或存储格式。若发现某个 `.js` 文件受当前工具链硬性限制不能转为 `.ts`，必须先停止并汇报原因、证据和备选方案，不得静默保留。

## 2. 当前迁移范围

必须处理的文件类型：

- 配置：`vite.config.js`、`svelte.config.js`、`jsconfig.json`。
- 路由脚本：`src/routes/+layout.js`。
- 前端模块：`src/lib/reminders.js`、`src/lib/i18n.js`。
- Svelte 组件：`src/routes/+page.svelte`、`src/routes/overlay/+page.svelte`、`src/lib/ReminderEditor.svelte`、`src/lib/SettingsPanel.svelte`、`src/routes/+layout.svelte` 的 `<script>`。
- 测试：`tests/unit/*.test.js`、`tests/integration/*.test.js`。
- 文档：`AGENTS.md`、必要时更新 `README.md` 中与 JS/命令冲突的内容。

不迁移 Rust、JSON、CSS、HTML、图片、字体、lockfile。

## 3. 执行顺序

### 3.1 建立基线

1. 运行 `git status --short`，记录已有未提交改动，不覆盖用户改动。
2. 运行 `bun run check` 和 `bun run test`，记录迁移前是否已有失败。
3. 用 `rg --files -g "*.js" -g "*.svelte"` 列出迁移清单，后续验收必须确认业务/测试 JS 已清空或有明确工具链例外。

### 3.2 TypeScript 配置

1. 将 `jsconfig.json` 替换为 `tsconfig.json`，保留当前严格检查意图：`strict`、`moduleResolution: "bundler"`、`resolveJsonModule`、`forceConsistentCasingInFileNames` 等。
2. 更新 `package.json` 的 `check` 脚本为 `svelte-kit sync && svelte-check --tsconfig ./tsconfig.json`。
3. 测试脚本必须能运行 `.test.ts`。优先使用当前 Node 24 能力保留 `node:test` 与 `mock.module`，例如 `node --test --experimental-test-module-mocks --experimental-strip-types "tests/**/*.test.ts"`。若本地 Node 不支持该模式，停止并提出最小依赖方案，不能直接丢失 mock 能力。

### 3.3 配置文件迁移

1. `vite.config.js` 改为 `vite.config.ts`，去掉不必要的 `@ts-expect-error`，为 `process.env.TAURI_DEV_HOST` 提供清晰类型处理。
2. 尝试将 `svelte.config.js` 改为 `svelte.config.ts`，并通过 `bun run check`、`bun run build` 验证 SvelteKit/Vite 都能发现配置。
3. 若 Svelte 插件不能发现 `svelte.config.ts`，不得自行绕过；必须汇报当前依赖证据，并由用户决定是否保留唯一 `.js` 例外或升级工具链。

### 3.4 共享类型设计

只新增直接服务迁移的类型，不抽象业务。建议新增 `src/lib/types.ts`，集中以下类型：

- `ReminderConfig`：`id`、`name`、`text`、`interval_secs`、`display_secs`、`enabled`、`play_sound`、`fullscreen`。
- `CountdownItem` 与 `CountdownMap`。
- `AppConfig`：至少包含当前前端读取的 `hide_main_window_on_startup`。
- `LocaleCode`：`"zh" | "en"`。
- `Translations`：覆盖 `i18n` 中字符串与格式化函数。

类型必须与 Rust Tauri 命令返回结构一致；不做字段重命名，不做数据迁移。

### 3.5 前端模块迁移

1. `src/lib/reminders.js` 改为 `src/lib/reminders.ts`，为 `invoke` 返回值、参数和 `createDefaultReminder` 加类型。
2. `src/lib/i18n.js` 改为 `src/lib/i18n.ts`，为 locale store、translations、formatter 参数加类型。
3. 更新所有导入路径。优先使用无扩展名导入，如 `$lib/reminders`、`../../src/lib/reminders`，避免残留 `.js` 后缀。
4. 不修改默认提醒值、倒计时格式、locale 检测逻辑、Tauri command 名称。

### 3.6 Svelte 组件迁移

1. 所有含脚本的 `.svelte` 文件改为 `<script lang="ts">`。
2. 为 `$state` 状态、事件处理函数、DOM 引用、Tauri event payload 添加必要类型。
3. 事件类型要具体，例如 `KeyboardEvent`、`MouseEvent`、`Event`；DOM target 需要收窄后再访问。
4. 不改组件结构、CSS、文案、按钮行为、窗口拖动/关闭逻辑、右键菜单拦截逻辑。

### 3.7 测试迁移与补齐

1. 将测试文件改为 `.test.ts`，保留现有断言覆盖：
   - `createDefaultReminder` 使用翻译默认值和英文 fallback。
   - `formatDurationLocale` 与 `formatCountdownLocale` 的边界值。
   - reminder Tauri command wrapper 调用命令名和 payload。
   - locale 初始化、切换、localStorage 和 `set_locale` 同步。
2. 测试代码必须使用类型化 mock 数据，尤其是 `ReminderConfig` 和 translation formatter。
3. 如果迁移过程中发现类型缺口，只补与迁移相关的测试，不新增无关测试场景。
4. 原测试语义不得弱化：不能删除断言，不能把失败断言改成只检查 truthy。

### 3.8 文档同步

1. `AGENTS.md` 必须明确以后新增和修改前端代码时优先使用 TypeScript：`.ts` 模块和 `<script lang="ts">`。
2. 若 `README.md` 或其他文档出现 `jsconfig.json`、`.js` 测试名或旧命令，按迁移结果同步。
3. 不新增长期迁移说明；本文档是临时执行依据，完成后可删除。

## 4. 防走偏规则

- 每次只处理一个层级：配置、共享类型、模块、组件、测试、文档。
- 不引入状态管理库、格式化工具、lint 工具或 UI 库。
- 不把类型迁移变成目录重构。
- 不修改 Rust 数据结构，除非 TypeScript 类型暴露出当前前后端字段确实不一致；这种情况必须先汇报。
- 不删除已解决问题相关逻辑：右键菜单禁用、WebView 白屏规避、托盘打开设置、倒计时 tick、locale 同步、全屏提醒退出逻辑、自动启动设置。

## 5. 验收标准

必须全部满足：

1. `rg --files -g "*.js"` 不再列出业务、路由、测试或 Vite 配置 JS；若存在 `svelte.config.js`，必须有用户批准的工具链例外说明。
2. `bun run check` 通过。
3. `bun run test:unit` 通过。
4. `bun run test:integration` 通过。
5. `bun run test` 通过。
6. `bun run build` 通过。
7. 关键人工冒烟流程正常：
   - 主窗口能打开，无白屏。
   - 新建、编辑、保存、删除提醒正常。
   - 启用/禁用提醒后列表与倒计时刷新正常。
   - 中英文切换正常，并调用 `set_locale`。
   - 设置页能打开，自动启动、启动隐藏、音量设置不报错。
   - overlay 页面显示提醒，倒计时和提前退出逻辑正常。
   - 非编辑区域右键菜单仍被禁用，输入框/文本框仍保留原生编辑菜单。

## 6. 最终交付要求

最终回复必须列出：

- 实际迁移的文件清单。
- 仍保留的 `.js` 文件及原因；理想情况为无。
- 已运行的命令和结果。
- 人工冒烟项是否完成；未完成项必须说明原因。
- 是否修改了 `AGENTS.md` 和其他文档。

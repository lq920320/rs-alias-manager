# rs-alias-manager 项目代码格式规范

## 1. Rust 代码格式

### 1.1 格式化工具

项目使用 **rustfmt** 作为统一的代码格式化工具，配置文件为根目录的 `rustfmt.toml`。

```bash
# 格式化前端代码
cargo fmt --all

# 格式化后端代码
cd src-tauri && cargo fmt --all
```

### 1.2 格式规则

| 规则 | 设定值 | 说明 |
|------|--------|------|
| `edition` | `2021` | Rust 2021 Edition |
| `max_width` | `100` | 单行最大宽度 |
| `tab_spaces` | `4` | 缩进为 4 个空格 |
| `hard_tabs` | `false` | 禁止使用 Tab 字符 |
| `newline_style` | `Auto` | 依据操作系统自动选择换行符 |
| `use_small_heuristics` | `Max` | 尽量减少单行化 |
| `reorder_imports` | `true` | 自动排序 import 语句 |
| `reorder_modules` | `true` | 自动排序 mod 声明 |
| `match_block_trailing_comma` | `true` | match 块末尾自动加逗号 |

### 1.3 推荐工作流

```bash
# 每次提交前运行格式化
cargo fmt --all
cd src-tauri && cargo fmt --all && cd ..

# 检查是否有格式问题（CI 中推荐）
cargo fmt --all -- --check
cd src-tauri && cargo fmt --all -- --check
```

---

## 2. HTML 格式

### 2.1 文件

- `index.html` — 唯一 HTML 文件，Trunk 构建入口

### 2.2 规则

- 缩进：2 个空格
- 使用 Trunk `data-trunk` 属性标记资源
- CSS 通过 `data-trunk rel="css"` 引入

---

## 3. CSS 格式

### 3.1 文件

- `style.css` — 全局样式表

### 3.2 规则

- **命名规范**：BEM（Block Element Modifier）
  - Block: `.sidebar`
  - Element: `.sidebar__nav-item`
  - Modifier: `.sidebar__nav-item--active`
- **CSS 变量**：全部定义在 `:root` 中
- **暗色模式**：通过 `[data-theme="dark"]` 覆盖变量
- **缩进**：2 个空格
- 每个主要模块前添加分隔注释块

### 3.3 变量命名

```css
--color-*       /* 颜色 */
--text-*        /* 文字颜色 */
--bg-*          /* 背景色 */
--border-*      /* 边框颜色 */
--shadow-*      /* 阴影 */
--space-*       /* 间距 */
--radius-*      /* 圆角 */
--font-*        /* 字体 */
--transition-*  /* 过渡动画 */
--sidebar-*     /* 侧边栏专用 */
```

---

## 4. 命名规范

### 4.1 Rust 代码

| 类型 | 格式 | 示例 |
|------|------|------|
| 模块/文件 | `snake_case` | `alias_page.rs` |
| 结构体/枚举 | `PascalCase` | `AppState` |
| 函数 | `snake_case` | `list_aliases` |
| 常量 | `SCREAMING_SNAKE_CASE` | `DEFAULT_SHELL` |
| 组件 | `PascalCase` | `Sidebar` |
| CSS class | `kebab-case` (BEM) | `sidebar__nav-item--active` |

### 4.2 Leptos 组件规范

```rust
/// 组件文档注释（/// 三斜杠）
#[component]
pub fn ComponentName(
    prop1: ReadSignal<String>,
    prop2: Callback<()>,
) -> impl IntoView {
    // 1. 获取 context
    // 2. 定义 signals
    // 3. 定义 effects
    // 4. 定义 callbacks
    // 5. view! 宏
    view! { ... }
}
```

### 4.3 Tauri 命令规范

```rust
/// 命令文档注释
#[tauri::command]
pub async fn command_name(arg: String) -> Result<Output, String> {
    // 业务逻辑
}
```

---

## 5. 文件组织

```
src/
├── main.rs          # WASM 入口
├── app.rs           # 根组件（路由 + 布局）
├── api/             # Tauri 后端 API 封装
├── state/           # 全局响应式状态
├── components/      # 可复用 UI 组件
└── pages/           # 路由页面

src-tauri/
├── src/
│   ├── main.rs      # Tauri 入口
│   ├── lib.rs       # 插件注册 + 命令挂载
│   ├── commands/    # Tauri 命令实现
│   ├── models/      # 数据模型
│   └── services/    # 业务逻辑层
└── Cargo.toml       # 后端依赖
```

---

## 6. 检查清单

提交代码前确认：

- [ ] `cargo fmt --all` 无格式问题
- [ ] `cd src-tauri && cargo fmt --all` 无格式问题
- [ ] `trunk build` 编译通过
- [ ] `cd src-tauri && cargo check` 编译通过
- [ ] 无新增 `unwrap()` / `expect()` （除非有充分理由）
- [ ] 组件的文档注释完整

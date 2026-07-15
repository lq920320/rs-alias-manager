# rs-alias-manager (别名管理器)

<p align="center">
  <img src="docs/logo.png" alt="rs-alias-manager" width="200" />
</p>

<p align="center">
  <strong>🦀 用 Rust 构建的现代化桌面端 Shell 别名管理器</strong>
</p>

<p align="center">
  <a href="#项目背景">背景</a> ·
  <a href="#功能特性">特性</a> ·
  <a href="#快速开始">快速开始</a> ·
  <a href="#项目结构">结构</a> ·
  <a href="#技术栈">技术栈</a> ·
  <a href="#贡献">贡献</a>
</p>

---

## 项目背景

在日常终端开发中，Shell 别名（alias）是提升效率的利器。但管理它们通常需要手动编辑 `.bashrc` 或 `.zshrc`，不仅容易因语法错误导致配置文件失效，在多台机器间同步别名也极其繁琐。

**rs-alias-manager** 提供了一套可视化的别名管理方案：

- 告别手动编辑配置文件，通过 GUI 表单增删改查别名
- 内置常用别名模板库，新机初始化时几分钟完成环境搭建
- 别名变更后自动即时生效，无需 `source` 或重启终端
- 支持 Bash、Zsh、Fish 多种 Shell，自动检测与切换

## 功能特性

| 功能 | 说明 |
|------|------|
| 别名列表 | 自动读取 Shell 配置文件，解析并以卡片列表展示 |
| 增删改查 | 可视化表单添加/编辑/删除别名，原子写入保证配置安全 |
| 模板库 | 预置 Git、Docker、文件操作、网络等分类模板，一键导入 |
| 搜索过滤 | 按别名名称、命令内容或标签实时搜索 |
| 彩色标签 | 为别名添加自定义标签，6 种颜色自动轮换，支持暗色模式 |
| 导出导入 | JSON 格式导出/导入别名配置，便于跨机器迁移 |
| 批量操作 | 批量添加/删除别名，一次调用完成 |
| 多 Shell | 自动检测并支持 Bash / Zsh / Fish，可手动切换 |
| 深色模式 | 支持浅色/深色主题切换，跟随系统偏好 |
| 国际化 | 支持英文和中文界面，可在设置中切换 |
| 安全写入 | 写临时文件 → 原子替换，避免写入中断损坏配置 |
| 设置缓存 | 后端缓存设置，减少文件 I/O |
| CSP 安全 | 配置了 Content Security Policy，防止 XSS |

## 快速开始

### 环境要求

- **Rust** 1.75+（[安装指引](https://www.rust-lang.org/tools/install)）
- **macOS** 10.15+ / **Linux**（Wayland / X11）/ **Windows** 10+
- **Trunk**（WASM 构建工具）

```bash
# 安装 Trunk
cargo install trunk

# 添加 WASM 编译目标
rustup target add wasm32-unknown-unknown
```

### 开发运行

```bash
# 克隆项目
git clone https://github.com/your-username/rs-alias-manager.git
cd rs-alias-manager

# 启动开发模式（Tauri 桌面窗口 + 热重载）
cargo tauri dev
```

### 仅运行前端（浏览器预览）

```bash
# 通过 Trunk 单独启动前端 WASM 服务（无 Tauri 后端，使用 mock 数据）
trunk serve
# 打开 http://127.0.0.1:1420
```

### 生产构建

```bash
cargo tauri build

# macOS 产物在 src-tauri/target/release/bundle/
# Linux 产物为 .deb / .AppImage
# Windows 产物为 .msi / .exe
```

### 安装预编译版本

从 [Releases 页面](../../releases) 下载。

**macOS**：由于应用未使用 Apple Developer 证书签名，首次打开时 macOS Gatekeeper 可能提示"文件已损坏"。这是安全策略拦截，**并非文件真的损坏**。修复方法：

```bash
sudo xattr -rd com.apple.quarantine /Applications/rs-alias-manager.app
```

或在访达中右键点击应用 → **打开** → 在弹窗中确认打开。

**Windows**：SmartScreen 可能提示"未知发布者"，点击"更多信息" → "仍要运行"即可。

## 项目结构

```
rs-alias-manager/
├── index.html                 # HTML 入口
├── style.css                  # 全局样式（含暗色主题变量）
├── Trunk.toml                 # Trunk 构建配置
├── Cargo.toml                 # 前端 Rust 依赖
├── clippy.toml                # Clippy 配置
│
├── src/                       # 前端（Leptos WASM）
│   ├── main.rs                # WASM 入口，挂载根组件
│   ├── app.rs                 # 根组件，路由与布局
│   ├── i18n.rs                # 国际化模块（中英双语）
│   ├── utils.rs               # 共用工具函数
│   ├── api/commands.rs        # Tauri 后端 API 封装
│   ├── state/app_state.rs     # 全局响应式状态
│   ├── components/
│   │   ├── sidebar.rs         # 侧边栏导航 + 主题切换
│   │   ├── alias_list.rs      # 别名列表 + 多选
│   │   ├── alias_form.rs      # 别名添加/编辑表单
│   │   ├── search_bar.rs      # 搜索过滤栏
│   │   ├── template_category_tabs.rs  # 模板分类 Tab
│   │   ├── template_list.rs   # 模板列表
│   │   └── settings_form.rs   # 设置表单（语言、Shell 等）
│   └── pages/
│       ├── alias_page.rs      # 别名管理页
│       ├── template_page.rs   # 模板库页
│       └── settings_page.rs   # 设置页
│
├── src-tauri/                 # 后端（Tauri Rust）
│   ├── Cargo.toml
│   ├── tauri.conf.json        # Tauri 应用配置
│   ├── clippy.toml            # 后端 Clippy 配置
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs             # 插件注册 + 命令挂载
│   │   ├── state.rs           # 应用状态（含设置缓存）
│   │   ├── error.rs           # 统一错误类型
│   │   ├── commands/          # Tauri 命令
│   │   │   ├── alias_cmds.rs  # 别名 CRUD + 批量操作
│   │   │   ├── template_cmds.rs
│   │   │   └── settings_cmds.rs
│   │   ├── models/            # 数据模型
│   │   │   ├── alias.rs
│   │   │   ├── shell_type.rs
│   │   │   └── template.rs
│   │   └── services/          # 业务逻辑
│   │       ├── alias_parser.rs    # 配置文件解析（含标签注释）
│   │       ├── shell_config.rs    # Shell 配置读写
│   │       ├── safe_writer.rs     # 原子安全写入
│   │       ├── template_library.rs # 模板数据（JSON 驱动）
│   │       ├── templates.json     # 外置模板定义文件
│   │       └── app_settings.rs    # 应用设置持久化
│   └── icons/                 # 应用图标
│
└── docs/                      # 文档
    ├── prd.md                 # 产品需求文档
    ├── architecture.md        # 架构设计
    ├── CODE_STYLE.md          # 代码风格规范
    ├── class-diagram.mermaid
    └── sequence-diagram.mermaid
```

## 技术栈

| 层 | 技术 | 说明 |
|---|------|------|
| **前端框架** | [Leptos 0.8](https://leptos.dev/) | Rust WASM 响应式 UI 框架（CSR 模式） |
| **路由** | [leptos_router 0.8](https://docs.rs/leptos_router/) | 前端路由 |
| **桌面框架** | [Tauri v2](https://v2.tauri.app/) | Rust 桌面应用框架 |
| **构建工具** | [Trunk](https://trunkrs.dev/) | Rust WASM 打包与开发服务器 |
| **后端** | Rust | 纯 Rust 后端，无 Node.js 依赖 |
| **插件** | tauri-plugin-fs / dialog / shell | 文件系统、文件对话框、Shell 命令 |
| **样式** | Vanilla CSS | BEM 命名，CSS 自定义属性主题系统 |
| **国际化** | 自研（纯 Rust） | Signal 驱动，零依赖翻译系统 |

## 贡献

欢迎贡献！无论是 Bug 报告、功能建议还是代码 PR。

### 如何贡献

1. **Fork** 本仓库
2. 创建特性分支：`git checkout -b feature/amazing-feature`
3. 提交更改：`git commit -m 'feat: add amazing feature'`
4. 推送分支：`git push origin feature/amazing-feature`
5. 提交 **Pull Request**

### 开发指引

```bash
# 前端检查
cargo check
trunk build

# 后端检查 + 测试
cd src-tauri && cargo check && cargo test
```

### 提交规范

采用 [Conventional Commits](https://www.conventionalcommits.org/)：

- `feat:` 新功能
- `fix:` Bug 修复
- `docs:` 文档更新
- `style:` 样式调整
- `refactor:` 代码重构
- `perf:` 性能优化
- `test:` 测试相关
- `chore:` 构建/工具链变更

## 许可证

本项目采用 [MIT License](LICENSE) 开源。

---

<p align="center">
  Made with Rust 🦀 and Tauri ❤️
</p>

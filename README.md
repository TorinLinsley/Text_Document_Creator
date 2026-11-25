# 文本文档创建器

[English](README-EN.md) | 简体中文 | [繁体中文](README-HK.md)

用 Rust 重新制作的 Windows 托盘小工具：按下全局快捷键就在当前资源管理器目录（或桌面）创建一份新的文本文档，也可以用快捷键打开记事本。

界面不是标准控件拼出来的，而是一套自绘的现代设置窗口：无边框、圆角、柔和阴影、自动/浅色/深色主题、20 种界面语言，全部由纯 GDI 绘制，整套程序只依赖系统自带的 DLL。

## 特性

- **单文件 exe**：静态链接 CRT，只依赖 `user32` / `gdi32` / `dwmapi` / `shell32` / `advapi32` / `uxtheme` / `ole32`，复制到任意 Windows 10/11 机器即可运行
- **单进程**：没有子进程、没有额外的宿主进程；任务管理器「进程」与「详细信息」里只有一个进程，进程名随系统语言显示
- **配置即文件**：全部设置存放在 exe 同目录的 `tdc_config.json`，首次运行自动生成出厂配置，删掉这个文件就回到出厂状态
- **全局快捷键**：默认 `Win+F` 创建文本文档、`Win+J` 打开记事本，包含 `Win` 的组合也能被覆盖
- **托盘常驻**：启动后只出现在托盘，不弹设置窗口；左键单击托盘打开设置，右键菜单为 设置 / 开机自启动 / 退出
- **现代化界面**：悬停、按压、下拉展开、开关滑块、主题切换均为动画，且可被打断
- **20 种界面语言**：启动默认跟随 Windows 系统语言，下拉支持滚轮滚动，手动选择后记住该选择
- **高 DPI 自适应**：Per-Monitor V2，125% / 150% 下文字清晰；运行中修改缩放或分辨率实时重排
- **轻量**：exe 约 508 KB；只驻留托盘、未打开设置界面且静止时，专用工作集仅约 **280 KB**，打开设置界面后私有内存约 1.5 MB

## 下载安装

### 直接下载

1. 打开 [Releases](https://github.com/TorinLinsley/Text_Document_Creator/releases) 下载最新的 `text_document_creator_版本号.exe`
2. 放到任意目录（例如 `D:\Tools\TextDocumentCreator`），双击即可运行
3. 无需安装，也不需要 .NET 或 VC++ 运行库

### 自行编译

```bash
git clone https://github.com/TorinLinsley/Text_Document_Creator.git
cd Text_Document_Creator
cargo build --release
```

需要 Rust 工具链（MSVC 目标）与 Windows SDK，产物为 `target\release\text_document_creator_版本号.exe`。

### 首次运行

- 启动后不会弹窗，图标直接出现在托盘，左键单击打开设置界面
- 出厂默认只开启「创建文本文档」，「打开记事本」与「开机自启动」默认关闭，需要时在设置界面或托盘菜单里自行打开
- 卸载：见下方「[卸载](#卸载)」

## 使用

1. 双击运行，程序驻留托盘（启动时不显示窗口）
2. 按 `Win+F`：在**当前聚焦的资源管理器目录**中创建 `新建文本文档.txt`，已存在则依次为 `新建文本文档_1.txt`、`_2`…；焦点在桌面时创建到桌面
3. 按 `Win+J`：每次都在新窗口打开记事本，连续按下可同时打开多个，从按下到窗口出现约 0.15 秒
4. 左键单击托盘图标打开设置：开关功能、绑定快捷键、切换语言与主题、查看关于

## 快捷键

- 修饰键：`Ctrl`、`Shift`、`Alt`、`Win`，左右两侧均可，任意组合
- 主键：字母 A–Z、数字 0–9、`F1`–`F24`、小键盘 `Num0`–`Num9` 与 `Num*` `Num+` `Num-` `Num.` `Num/`、方向键、`Home`/`End`/`PageUp`/`PageDown`/`Insert`/`Delete`、`Space`/`Tab`/`Enter`/`Backspace`/`Esc`、符号键与多媒体键
- 在输入框中直接按下目标组合即可绑定，`Esc` 取消；「清除」清空当行，「重置」恢复该行默认，「重置全部」恢复所有默认
- 全局钩子运行在独立的 TIME_CRITICAL 线程上，优先级高于系统与其他程序；命中组合键时吞掉按键，且不会残留 `Win` 状态（不会误弹开始菜单或系统搜索）
- 绑定时按住 `Win` 再按其他键，不会再弹出开始菜单

### 默认快捷键

| 功能 | 默认快捷键 |
| --- | --- |
| 创建文本文档 | `Win+F` |
| 打开记事本 | `Win+J` |
| 开关-创建文档 | 未设置 |
| 开关-打开记事本 | 未设置 |

## 设置项

- **功能开关**：创建文本文档、打开记事本、开机自启动（与托盘菜单双向实时同步）
- **快捷键设置**：四行快捷键绑定，逐行清除/重置，一键重置全部
- **语言**：20 种语言下拉，支持滚轮滚动与聚焦后 ↑ ↓ 切换
- **主题**：自动 / 浅色 / 深色，自动模式跟随系统主题实时切换

核心功能为新建文本文档，打开记事本为可选功能，默认为关闭状态，需要可自行在设置中打开。

## 注意事项

- 一直按住快捷键会持续触发（120ms 节流），可连续创建多个文件或打开多个记事本窗口
- 创建文件失败时不弹错误框，避免打断当前操作

## 架构

整个程序是单个 Win32 窗口 + 一个托盘图标 + 一个全局钩子，没有子进程，也没有常驻的辅助线程（按键对应的动作按需临时起线程，做完即退出）。

- **窗口模型**：`WS_POPUP` + `WS_THICKFRAME`，`WM_NCCALCSIZE` 返回 0 让客户区铺满整窗，标题栏、最小化/关闭按钮全部自绘；`WM_NCHITTEST` 把顶部判为 `HTCAPTION`、四边与四角判为缩放边框，因此拖动、贴边、Snap Layouts 都是系统原生行为
- **窗口外观**：`DwmSetWindowAttribute` 设置圆角偏好（`DWMWCP_ROUND`）、沉浸式深色模式与边框色，失焦时也不出现白边
- **绘制**：`WM_PAINT` 里创建兼容位图，全部内容画完后一次 `BitBlt` 到窗口，避免闪烁
- **动画**：所有过渡量都是指数逼近（`v += (target - v) * (1 - exp(-speed * dt))`），由 15ms 定时器驱动，只在有动画时存在，动画结束立即 `KillTimer`，空闲 0% CPU
- **热键动作**：命中快捷键后只向窗口投递消息，具体动作（建文件 / 启动记事本）在临时线程里执行，主线程不阻塞，因此连按不会排队
- **状态**：`App` 结构持有全部界面状态，通过窗口用户数据指针共享给窗口过程；设置变化时写注册表并立即同步热键与托盘

## 渲染管线

- **技术栈**：纯 GDI（`gdi32`），不使用 GDI+、Direct2D 或任何第三方 UI 库
- **每帧流程**：`CreateCompatibleDC` + `CreateCompatibleBitmap`（按客户区尺寸）→ 绘制 → `BitBlt` 到窗口 → `DeleteObject` + `DeleteDC`，缓冲不常驻，这也是内存能压到几 MB 的关键
- **图元**：圆角矩形用 `RoundRect`（同时支持填充与 1px 描边）、圆形用 `Ellipse`、线段用 `CreatePen` + `MoveToEx/LineTo`
- **渐变与阴影**：渐变按钮在圆角路径裁剪区内画多条水平色带（`BeginPath` / `EndPath` / `SelectClipPath`），阴影用几层逐渐变淡的圆角外扩模拟
- **透明度**：GDI 没有 alpha 混合，所有半透明颜色在绘制前与当前背景色做一次逐通道混合（`blend`），既得到正确的视觉效果，又不引入额外依赖
- **文字**：`CreateFontW` + `DrawTextW`，开启 ClearType 与 `DT_END_ELLIPSIS`；中文/日文/韩文用 `Microsoft YaHei UI`，其余用 `Segoe UI`；文案过长时按可用宽度逐级缩小字号再省略
- **DPI**：Per-Monitor V2 感知，所有坐标以逻辑像素为单位，绘制时统一乘以 `dpi / 96`；`WM_DPICHANGED` 与 `WM_DISPLAYCHANGE` 触发布局重排与屏幕适配

## 全局热键实现

- `SetWindowsHookExW(WH_KEYBOARD_LL)` 装在独立线程，线程优先级 `THREAD_PRIORITY_TIME_CRITICAL`，安装完成后主线程才继续，确保任何程序里的组合键都先被本程序处理
- 修饰键状态自己维护：用原子位掩码记录左右 Ctrl / Shift / Alt / Win 的按下情况，而不是依赖 `GetAsyncKeyState`，避免 `Win` 按下时状态不可靠
- `Win` 键被钩子吞下后：命中自定义组合键就丢弃，单独按 `Win` 时补发一次 `Win` 按下与抬起（开始菜单照常打开），其余组合键（如 `Win+E`）则先补发 `Win` 再放行，因此不会把系统状态弄乱
- 自身注入的按键带 `dwExtraInfo` 标记，钩子里直接放行，避免自吞
- 命中后有 120ms 节流，长按不会刷屏式触发
- 建文件时取前台窗口的 `HWND`：桌面（`Progman` / `WorkerW`）落到桌面目录，资源管理器窗口通过 `IShellWindows` 枚举匹配 `HWND` 并读取当前路径，取不到才回退桌面
- 打开记事本用 `CreateProcessW` 启动 `%SystemRoot%\System32\notepad.exe`（工作目录留空，避免商店版记事本找不到依赖 DLL），再短暂等待并用 `EnumWindows` 找到本次进程的窗口置前

## 语言

- 20 种语言：简体中文、繁體中文、English (US)、English (UK)、日本語、한국어、Français、Deutsch、Español、Italiano、Русский、Português (BR)、Polski、Nederlands、Türkçe、Українська、Tiếng Việt、Bahasa Indonesia、ไทย、हिन्दी
- 启动时用 `GetUserDefaultLocaleName` 判断系统区域（`zh-CN` 等 → 简体中文，`zh-TW`/`zh-HK`/`zh-MO` → 繁體），再回退到 `GetUserDefaultUILanguage`；配置里的 `language` 为 `auto` 时跟随系统，手动选过语言后写入对应代码（如 `zh_hk`），不再被系统语言覆盖
- 版本资源内嵌 8 种语言的 `FileDescription`，任务管理器「进程」页的进程名随系统语言变化

## 性能与内存

| 指标                 | 实测                                      |
| ------------------ | --------------------------------------- |
| exe 体积             | 约 508 KB                                |
| 私有内存（PrivateBytes） | 约 1.5 MB                                |
| 托盘驻留、未打开设置界面（静止）   | 专用工作集约 280 KB，提交大小约 1.5 MB            |
| 设置界面打开但静止          | 专用工作集约 0.2 MB（1.5 秒无操作后 trim）           |
| 重绘峰值               | 约 1.5 MB（1060×620 的 32bpp 兼容位图，每帧创建后释放） |
| 进程数 / 子进程          | 1 / 0                                   |
| 空闲 CPU             | 0%                                      |

## 构建

```
cargo build --release
```

产物：`target\release\text_document_creator.exe`

- 需要 Rust 工具链（MSVC 目标）与 Windows SDK（`build.rs` 自动查找 `rc.exe` 编译图标与版本资源）
- `.cargo/config.toml` 启用 `+crt-static`，配合 `opt-level="z"`、LTO、`panic="abort"`、strip 得到免依赖的单个 exe
- 图标由 `assets/icon.ico`（16–256 共 9 个尺寸）编入 exe，任务栏、资源管理器与任务管理器均显示高分辨率图标

## 项目结构

| 文件 | 作用 |
| --- | --- |
| `src/main.rs` | 窗口创建、消息循环、托盘、DPI 与主题变化、单实例 |
| `src/app.rs` | 设置界面布局、命中测试、绘制与交互 |
| `src/render.rs` | 纯 GDI 绘制层（文字、圆角、渐变、阴影、图标） |
| `src/theme.rs` | 深浅色调色板、颜色混合与系统主题判定 |
| `src/anim.rs` | 指数逼近动画 |
| `src/i18n.rs` | 20 种语言文案与系统语言检测 |
| `src/config.rs` | JSON 配置读写与开机自启动登记 |
| `src/hotkey.rs` | 全局键盘钩子、按键绑定、建文件与开记事本 |
| `src/ffi.rs` / `src/sys.rs` | Win32 常量与 API 声明 |

## 数据位置

### 2.1.0 及以后

- 设置：exe 同目录的 `tdc_config.json`（首次运行时自动写出厂配置，之后每次修改即时写回；也可以直接编辑该文件，重启程序后生效）

字段说明：

| 字段 | 含义 |
| --- | --- |
| `version` | 配置格式版本号，固定为 `1`；只在将来配置结构变化时用于迁移，平时不用改 |
| `enable_create` | 创建文本文档开关，出厂 `true` |
| `enable_notepad` | 打开记事本开关，出厂 `false` |
| `autostart` | 开机自启动开关，出厂 `false` |
| `language` | 界面语言，`auto` 表示跟随系统 |
| `theme` | `0` 自动、`1` 浅色、`2` 深色 |
| `hotkey_create` / `hotkey_notepad` | 主功能快捷键，如 `Win+F`、`Win+J` |
| `hotkey_toggle_create` / `hotkey_toggle_notepad` | 两个开关快捷键，留空表示未设置 |

`language` 可用的语言代码：

| 代码 | 语言 | 代码 | 语言 |
| --- | --- | --- | --- |
| `auto` | 跟随系统 | `ru_ru` | Русский |
| `zh_cn` | 简体中文 | `pt_br` | Português (BR) |
| `zh_hk` | 繁體中文 | `pl_pl` | Polski |
| `en_us` | English (US) | `nl_nl` | Nederlands |
| `en_uk` | English (UK) | `tr_tr` | Türkçe |
| `ja_jp` | 日本語 | `uk_ua` | Українська |
| `ko_kr` | 한국어 | `vi_vn` | Tiếng Việt |
| `fr_fr` | Français | `id_id` | Bahasa Indonesia |
| `de_de` | Deutsch | `th_th` | ไทย |
| `es_es` | Español | `hi_in` | हिन्दी |
| `it_it` | Italiano | | |
- 开机自启动：计划任务 `\TextDocumentCreator`（`ONLOGON` + 最高权限，登录时静默启动、不弹 UAC；关闭开关会删除该任务）

### 2.0.0 及更早

- 设置：注册表 `HKEY_CURRENT_USER\Software\TextDocumentCreator`，全部是 `REG_DWORD` 值：`EnableCreate`、`EnableNotepad`、`AutoStart`、`VkCreate`、`ModCreate`、`VkNotepad`、`ModNotepad`、`VkSwitchCreate`、`ModSwitchCreate`、`VkSwitchNotepad`、`ModSwitchNotepad`、`Language`、`LanguageAuto`、`ThemeMode`
- 开机自启动：计划任务 `TextDocumentCreator`；最早的两版没有任务，只写 `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run` 下的 `Text_Document_Creator` 值（内容为 exe 完整路径）
- 这些旧版没有配置文件，升级到 2.1.0 后首次运行会读不到 `tdc_config.json`，按出厂值重建，旧注册表项不再被使用

## 卸载

- **2.1.0 及以后**
  1. 在设置界面或托盘菜单里关掉「开机自启动」，这一步会删除计划任务 `\TextDocumentCreator`
  2. 从托盘菜单选「退出」
  3. 删除 exe 和同目录的 `tdc_config.json`
  4. 完成，注册表和系统里不留任何设置残留
- **2.0.0 及更早**
  1. 从托盘菜单选「退出」
  2. 删除计划任务：以**管理员身份**打开「命令提示符」或「PowerShell」（开始菜单搜索后右键「以管理员身份运行」），执行下面的命令；需要管理员权限，否则会报「拒绝访问」
     ```bat
     schtasks /Delete /TN TextDocumentCreator /F
     ```
     提示「成功: 已成功删除计划任务 "TextDocumentCreator"」即表示删除完成；如果提示「系统找不到指定的文件」，说明该任务本来就不存在，直接进行下一步
  3. 打开注册表编辑器，删除 `HKEY_CURRENT_USER\Software\TextDocumentCreator`
  4. 若存在，删除 `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run` 下的 `Text_Document_Creator` 值（最早两版才有）
  5. 删除 exe

# Text Document Creator

English | [简体中文](README.md) | [繁体中文](README-HK.md)

A Windows tray utility remade with Rust: a global hotkey creates a new text document in the currently focused Explorer folder (or on the desktop), and another hotkey opens Notepad.

The settings window is not assembled from stock controls but drawn from scratch: borderless, rounded, soft shadows, Auto/Light/Dark themes and 20 UI languages, all rendered with plain GDI. The whole program depends only on DLLs that ship with Windows.

## Features

- **Single exe**: statically linked CRT, only depends on `user32` / `gdi32` / `dwmapi` / `shell32` / `advapi32` / `uxtheme` / `ole32`, so it runs from any Windows 10/11 machine as-is
- **Single process**: no child processes and no host process; one entry in both the Processes and Details tabs of Task Manager, with a display name that follows the system language
- **Settings are just a file**: everything lives in `tdc_config.json` next to the exe, written on first run and updated on every change, so deleting that one file restores the factory state
- **Global hotkeys**: `Win+F` creates a text document and `Win+J` opens Notepad by default; combinations that include `Win` are overridden as well
- **Tray resident**: starts silently in the tray without showing a window; left click opens the settings, right click shows Settings / Run at startup / Exit
- **Modern UI**: hover, press, dropdown, toggle and theme transitions are all animated and interruptible
- **20 UI languages**: follows the Windows system language on startup, the dropdown scrolls with the wheel and a manual choice is remembered
- **High DPI aware**: Per-Monitor V2, crisp text at 125% / 150%, live reflow when scaling or resolution changes
- **Lightweight**: about 508 KB exe; sitting in the tray with no settings window open and nothing happening costs about **280 KB** of private working set, and about 1.5 MB private memory once the settings window is open

## Usage

1. Double click to run; the app stays in the tray and shows no window on startup
2. Press `Win+F`: creates `新建文本文档.txt` in the **currently focused Explorer folder**, or `新建文本文档_1.txt`, `_2`... when it already exists; when the desktop is focused the file is created there
3. Press `Win+J`: opens Notepad in a new window every time, so several can be opened at once; about 0.15 s from key press to window
4. Left click the tray icon to open the settings: toggle features, bind shortcuts, switch language and theme, or read About

### Download and install

#### Download the exe

1. Open [Releases](https://github.com/TorinLinsley/Text_Document_Creator/releases) and download the latest `text_document_creator_版本号.exe`
2. Put it in any folder (for example `D:\Tools\TextDocumentCreator`) and double click it
3. Nothing to install, no .NET or VC++ runtime required

#### Build from source

```bash
git clone https://github.com/TorinLinsley/Text_Document_Creator.git
cd Text_Document_Creator
cargo build --release
```

This needs the Rust toolchain (MSVC target) and the Windows SDK; the output is `target\release\text_document_creator.exe`.

#### First run

- No window appears on startup: the icon simply shows up in the tray, and a left click opens the settings
- Out of the box only "Create text file" is enabled; "Open Notepad" and "Run at startup" are off until you turn them on in the settings or the tray menu
- To uninstall: see [Uninstall](#uninstall)

### Uninstall

- **2.1.0 and later**
  1. Turn "Run at startup" off in the settings window or the tray menu; this deletes the `\TextDocumentCreator` scheduled task
  2. Choose Exit in the tray menu
  3. Delete the exe and `tdc_config.json` in the same folder
  4. Done, nothing is left in the registry or anywhere else
- **2.0.0 and earlier**
  1. Choose Exit in the tray menu
  2. Delete the scheduled task: open Command Prompt or PowerShell **as administrator** (search it in the Start menu, then choose Run as administrator) and run the command below; administrator rights are required, otherwise it fails with "Access is denied"
     ```bash
     schtasks /Delete /TN TextDocumentCreator /F
     ```
     "SUCCESS: The scheduled task \"TextDocumentCreator\" was successfully deleted." means it is gone; "ERROR: The system cannot find the file specified." means the task never existed, so continue with the next step
  3. Open Registry Editor and delete `HKEY_CURRENT_USER\Software\TextDocumentCreator`
  4. If it exists, delete the `Text_Document_Creator` value under `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run` (only the two earliest releases)
  5. Delete the exe

## Shortcuts

- Modifiers: `Ctrl`, `Shift`, `Alt`, `Win`, either side of the keyboard, in any combination
- Keys: A-Z, 0-9, `F1`-`F24`, numpad `Num0`-`Num9` plus `Num*` `Num+` `Num-` `Num.` `Num/`, arrow keys, `Home`/`End`/`PageUp`/`PageDown`/`Insert`/`Delete`, `Space`/`Tab`/`Enter`/`Backspace`/`Esc`, symbol keys and media keys
- Focus a shortcut field and press the desired combination to bind it; `Esc` cancels. Clear empties the row, Reset restores its default and Reset all restores every default
- The global hook runs on its own TIME_CRITICAL thread with priority above the system and other applications; a matched combination is swallowed and no `Win` state is left behind (no accidental Start menu or system search)
- Holding `Win` while binding another key no longer pops up the Start menu

### Default hotkeys

| Action | Default hotkey |
| --- | --- |
| Create text file | `Win+F` |
| Open Notepad | `Win+J` |
| Toggle - create | Not set |
| Toggle - notepad | Not set |

## Settings

- **Features**: create text file, open Notepad, run at startup (kept in sync with the tray menu in both directions)
- **Shortcuts**: four bindable rows, per-row clear and reset, plus reset all
- **Language**: 20 languages in the dropdown, scrollable and switchable with the arrow keys while focused
- **Theme**: Auto / Light / Dark, with Auto following the system theme live

The core feature is creating a new text document; opening Notepad is optional and disabled by default, so enable it in the settings when needed.

## Notes

- Holding a shortcut keeps triggering it with a 120 ms throttle, creating several files or opening several Notepad windows
- A failed file creation is reported in the UI without an error dialog, so the current work is not interrupted

## Architecture

The program is one Win32 window, one tray icon and one global hook. It spawns no child process and keeps no helper thread; actions triggered by a hotkey run on a short-lived thread that exits when done.

- **Window model**: `WS_POPUP` + `WS_THICKFRAME` with `WM_NCCALCSIZE` returning 0 so the client area fills the frame; the title bar and the minimize/close buttons are drawn in-house. `WM_NCHITTEST` reports `HTCAPTION` for the header and resize borders for the edges, so dragging, snapping and Snap Layouts stay native behaviour
- **Window chrome**: `DwmSetWindowAttribute` sets the corner preference (`DWMWCP_ROUND`), immersive dark mode and the border colour, which also removes the white border on deactivation
- **Painting**: `WM_PAINT` creates a compatible bitmap, draws everything and blits it once, avoiding flicker
- **Animation**: every transition is an exponential approach, `v += (target - v) * (1 - exp(-speed * dt))`, driven by a 15 ms timer that only exists while something animates; the timer is killed when animation ends, so idle CPU stays at 0%
- **Hotkey actions**: a matched shortcut only posts a message to the window; the actual work (creating a file, launching Notepad) happens on a temporary thread, so the UI thread never blocks and repeated presses do not queue up
- **State**: the `App` struct owns all UI state and is shared with the window procedure through the window user-data pointer; settings changes are written to the registry and pushed to the hook and tray immediately

## Rendering pipeline

- **Stack**: plain GDI (`gdi32`), no GDI+, no Direct2D and no third-party UI library
- **Per frame**: `CreateCompatibleDC` + `CreateCompatibleBitmap` sized to the client area, draw, `BitBlt` to the window, then `DeleteObject` + `DeleteDC`. Keeping no persistent buffer is what keeps memory in the low megabytes
- **Primitives**: `RoundRect` for rounded rectangles (filling and 1 px strokes), `Ellipse` for circles, `CreatePen` + `MoveToEx`/`LineTo` for lines
- **Gradients and shadows**: gradient buttons draw horizontal colour bands inside a rounded clip path (`BeginPath` / `EndPath` / `SelectClipPath`), shadows are a few progressively fainter rounded layers
- **Transparency**: GDI has no alpha blending, so every translucent colour is blended per channel against the current backdrop (`blend`) before drawing. This gives the correct look without extra dependencies
- **Text**: `CreateFontW` + `DrawTextW` with ClearType and `DT_END_ELLIPSIS`; `Microsoft YaHei UI` for Chinese/Japanese/Korean and `Segoe UI` otherwise, with the font size stepped down when a label does not fit
- **DPI**: Per-Monitor V2, coordinates are logical pixels and are multiplied by `dpi / 96` while drawing; `WM_DPICHANGED` and `WM_DISPLAYCHANGE` re-run layout and screen fitting

## Global hotkey implementation

- `SetWindowsHookExW(WH_KEYBOARD_LL)` is installed on a dedicated thread with `THREAD_PRIORITY_TIME_CRITICAL`; the main thread waits for installation to finish, so a combination is handled here before the system or any other application sees it
- Modifier state is tracked manually in an atomic bitmask covering left/right Ctrl, Shift, Alt and Win instead of relying on `GetAsyncKeyState`, whose state is unreliable while `Win` is held
- After swallowing `Win`, a matched custom combination is dropped; pressing `Win` alone replays a `Win` down/up pair so the Start menu still opens, and for other combinations (such as `Win+E`) `Win` is replayed before the key is let through, so the system state never gets stuck
- Injected keys carry a `dwExtraInfo` marker and are passed straight through, so the hook never swallows its own events
- A 120 ms throttle stops a held shortcut from firing continuously
- File creation resolves the foreground window: the desktop (`Progman` / `WorkerW`) maps to the desktop folder, Explorer windows are matched by `HWND` through `IShellWindows` to read their current path, and the desktop is the fallback
- Notepad is launched with `CreateProcessW` on `%SystemRoot%\System32\notepad.exe` with an empty working directory (so the Store build still finds its DLLs), then the new process window is found with `EnumWindows` and brought to the front

## Languages

- 20 languages: Simplified Chinese, Traditional Chinese, English (US), English (UK), Japanese, Korean, French, German, Spanish, Italian, Russian, Portuguese (BR), Polish, Dutch, Turkish, Ukrainian, Vietnamese, Indonesian, Thai and Hindi
- On startup `GetUserDefaultLocaleName` decides the locale (`zh-CN` and similar map to Simplified Chinese, `zh-TW`/`zh-HK`/`zh-MO` to Traditional), with `GetUserDefaultUILanguage` as the fallback; `language: auto` follows the system, while a manual pick stores its code (for example `zh_hk`) and is no longer overridden
- The version resource embeds `FileDescription` in 8 languages, so the process name in Task Manager follows the system language

## Performance and memory

| Metric                                | Measured                                                                           |
| ------------------------------------- | ---------------------------------------------------------------------------------- |
| exe size                              | about 508 KB                                                                       |
| Private bytes                         | about 1.5 MB                                                                       |
| Tray resident, settings closed (idle) | about 280 KB private working set, about 1.5 MB commit                            |
| Settings open but idle                | about 0.2 MB private working set (trimmed 1.5 s after the last input)              |
| Repaint peak                          | about 1.5 MB (a 1060x620 32 bpp compatible bitmap, created and released per frame) |
| Processes / child processes           | 1 / 0                                                                              |
| Idle CPU                              | 0%                                                                                 |

## Build

```
cargo build --release
```

Output: `target\release\text_document_creator.exe`

- Requires the Rust toolchain (MSVC target) and the Windows SDK (`build.rs` locates `rc.exe` to compile the icon and version resource)
- `.cargo/config.toml` enables `+crt-static`; with `opt-level="z"`, LTO, `panic="abort"` and strip the result is one dependency-free exe
- The icon comes from `assets/icon.ico` (nine sizes from 16 to 256), so the taskbar, Explorer and Task Manager all show the high-resolution artwork

## Project layout

| File | Responsibility |
| --- | --- |
| `src/main.rs` | Window creation, message loop, tray, DPI and theme changes, single instance |
| `src/app.rs` | Settings layout, hit testing, painting and interaction |
| `src/render.rs` | Pure GDI drawing (text, rounded shapes, gradients, shadows, icons) |
| `src/theme.rs` | Dark and light palettes, colour blending and system theme detection |
| `src/anim.rs` | Exponential-approach animation |
| `src/i18n.rs` | Text for 20 languages and system language detection |
| `src/config.rs` | JSON settings file and run-at-startup registration |
| `src/hotkey.rs` | Global hook, key binding, file creation and Notepad launch |
| `src/ffi.rs` / `src/sys.rs` | Win32 constants and API declarations |

## Data locations

### 2.1.0 and later

- Settings: `tdc_config.json` next to the exe (factory values are written on first run, every change is saved immediately, and you can edit the file by hand and restart the app)

Fields:

| Field | Meaning |
| --- | --- |
| `version` | Config format version, always `1`; only used for migration if the structure ever changes, leave it alone |
| `enable_create` | Create text file switch, factory `true` |
| `enable_notepad` | Open Notepad switch, factory `false` |
| `autostart` | Run at startup switch, factory `false` |
| `language` | UI language, `auto` follows the system |
| `theme` | `0` auto, `1` light, `2` dark |
| `hotkey_create` / `hotkey_notepad` | Main shortcuts such as `Win+F` and `Win+J` |
| `hotkey_toggle_create` / `hotkey_toggle_notepad` | The two toggle shortcuts, empty means not set |

Available `language` codes:

| Code | Language | Code | Language |
| --- | --- | --- | --- |
| `auto` | Follow system | `ru_ru` | Русский |
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
- Run at startup: the `\TextDocumentCreator` scheduled task (`ONLOGON` with highest privileges, so logon is silent and shows no UAC prompt; turning the switch off deletes it)

### 2.0.0 and earlier

- Settings: registry `HKEY_CURRENT_USER\Software\TextDocumentCreator`, all `REG_DWORD`: `EnableCreate`, `EnableNotepad`, `AutoStart`, `VkCreate`, `ModCreate`, `VkNotepad`, `ModNotepad`, `VkSwitchCreate`, `ModSwitchCreate`, `VkSwitchNotepad`, `ModSwitchNotepad`, `Language`, `LanguageAuto`, `ThemeMode`
- Run at startup: the `TextDocumentCreator` scheduled task; the two earliest releases had no task and written the `Text_Document_Creator` value under `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run` (the exe path)
- Those versions have no config file, so the first run of 2.1.0 writes factory defaults and the old registry keys are no longer read

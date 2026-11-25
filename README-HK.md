# 文字文件建立器

[English](README-EN.md) | [简体中文](README.md) | 繁体中文

用 Rust 重新製作的 Windows 匣盤小工具：按下全域快速鍵就在目前檔案總管目錄（或桌面）建立一份新的文字文件，也可以用快速鍵開啟記事本。

介面不是用標準控制項拼出來的，而是一套自繪的現代設定視窗：無邊框、圓角、柔和陰影、自動/淺色/深色主題、20 種介面語言，全部以純 GDI 繪製，整套程式只依賴系統內建的 DLL。

## 特色

- **單一 exe**：靜態連結 CRT，只依賴 `user32` / `gdi32` / `dwmapi` / `shell32` / `advapi32` / `uxtheme` / `ole32`，複製到任意 Windows 10/11 電腦即可執行
- **單一處理序**：沒有子處理序、沒有額外的裝載處理序；工作管理員「處理程序」與「詳細資料」中都只有一個項目，名稱隨系統語言顯示
- **設定即檔案**：所有設定存放在 exe 同目錄的 `tdc_config.json`，首次執行自動產生出廠設定，刪除這個檔案就回到出廠狀態
- **全域快速鍵**：預設 `Win+F` 建立文字文件、`Win+J` 開啟記事本，含 `Win` 的組合也能被覆蓋
- **匣盤常駐**：啟動後只出現在匣盤，不彈出設定視窗；左鍵單擊開設定，右鍵選單為 設定 / 開機自動啟動 / 結束
- **現代化介面**：滑鼠移入、按下、下拉展開、開關滑塊、主題切換皆有動畫，且可被中斷
- **20 種介面語言**：啟動時預設跟隨 Windows 系統語言，下拉支援滾輪捲動，手動選擇後會記住該選擇
- **高 DPI 自適應**：Per-Monitor V2，125% / 150% 下文字清晰；執行中變更縮放或解析度會即時重排
- **輕量**：匣盤靜止時記憶體約佔 **64–88 KB**，開啟設定介面後也只有 1 MB 上下

## 下載、使用與解除安裝

### 下載

#### 直接下載

1. 開啟 [Releases](https://github.com/TorinLinsley/Text_Document_Creator/releases) 下載最新的 `text_document_creator_版本号.exe`
2. 放到任意目錄（例如 `D:\Tools\TextDocumentCreator`），雙擊即可執行
3. 無需安裝，也不需要 .NET 或 VC++ 執行階段

#### 自行編譯

```bash
git clone https://github.com/TorinLinsley/Text_Document_Creator.git
cd Text_Document_Creator
cargo build --release
```

需要 Rust 工具鏈（MSVC 目標）與 Windows SDK，產物為 `target\release\text_document_creator.exe`。

#### 首次執行

- 啟動後不會彈窗，圖示直接出現在匣盤，左鍵單擊開啟設定介面
- 需要開機自動啟動時，在設定介面或匣盤選單中開啟「開機自動啟動」

### 使用

1. 雙擊執行，程式常駐匣盤（啟動時不顯示視窗）
2. 按 `Win+F`：在**目前聚焦的檔案總管目錄**中建立 `新建文本文檔.txt`，已存在則依序為 `新建文本文檔_1.txt`、`_2`…；焦點在桌面時建立到桌面
3. 按 `Win+J`：每次都在新視窗開啟記事本，連續按下可同時開啟多個，從按下到視窗出現約 0.15 秒
4. 左鍵單擊匣盤圖示開啟設定：開關功能、綁定快速鍵、切換語言與主題、檢視關於

### 解除安裝

- **2.1.0 及之後**
  1. 在設定介面或匣盤選單關閉「開機自動啟動」，這一步會刪除排程工作 `\TextDocumentCreator`
  2. 從匣盤選單選「結束」
  3. 刪除 exe 與同目錄的 `tdc_config.json`
  4. 完成，登錄檔與系統中不會留下任何設定殘留
- **2.0.0 及更早**
  1. 從匣盤選單選「結束」
  2. 刪除排程工作：以**管理員身分**開啟「命令提示字元」或「PowerShell」（開始功能表搜尋後按右鍵「以管理員身分執行」），執行下面的指令；需要管理員權限，否則會出現「拒絕存取」
     ```bash
     schtasks /Delete /TN TextDocumentCreator /F
     ```
     顯示「成功: 已成功刪除排程工作 "TextDocumentCreator"」即表示刪除完成；若顯示「系統找不到指定的檔案」，表示該工作本來就不存在，直接進行下一步
  3. 開啟登錄編輯器，刪除 `HKEY_CURRENT_USER\Software\TextDocumentCreator`
  4. 若存在，刪除 `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run` 下的 `Text_Document_Creator` 值（最早兩版才有）
  5. 刪除 exe

## 快速鍵

- 修飾鍵：`Ctrl`、`Shift`、`Alt`、`Win`，左右兩側皆可，任意組合
- 主鍵：字母 A–Z、數字 0–9、`F1`–`F24`、小鍵盤 `Num0`–`Num9` 與 `Num*` `Num+` `Num-` `Num.` `Num/`、方向鍵、`Home`/`End`/`PageUp`/`PageDown`/`Insert`/`Delete`、`Space`/`Tab`/`Enter`/`Backspace`/`Esc`、符號鍵與多媒體鍵
- 在輸入框中直接按下目標組合即可綁定，`Esc` 取消；「清除」清空該行，「重設」還原該行預設，「全部重設」還原所有預設
- 全域鉤子執行於獨立的 TIME_CRITICAL 執行緒，優先順序高於系統與其他程式；命中組合鍵時會吞掉按鍵，且不會殘留 `Win` 狀態（不會誤彈開始功能表或系統搜尋）
- 綁定時按住 `Win` 再按其他鍵，不會再彈出開始功能表

### 預設快速鍵

| 功能 | 預設快速鍵 |
| --- | --- |
| 建立文字文件 | `Win+F` |
| 開啟記事本 | `Win+J` |
| 開關-建立文件 | 未設定 |
| 開關-開啟記事本 | 未設定 |

## 設定項目

- **功能開關**：建立文字文件、開啟記事本、開機自動啟動（與匣盤選單雙向即時同步）
- **快速鍵設定**：四行快速鍵綁定，逐行清除/重設，一鍵全部重設
- **語言**：20 種語言下拉，支援滾輪捲動與聚焦後 ↑ ↓ 切換
- **主題**：自動 / 淺色 / 深色，自動模式跟隨系統主題即時切換

核心功能為建立文字文件，開啟記事本為可選功能，預設為關閉狀態，需要可自行在設定中開啟。

## 注意事項

- 一直按住快速鍵會持續觸發（120ms 節流），可連續建立多個檔案或開啟多個記事本視窗
- 建立檔案失敗時不彈錯誤框，避免打斷目前操作

## 架構

整個程式是單一 Win32 視窗 + 一個匣盤圖示 + 一個全域鉤子，沒有子處理序，也沒有常駐的輔助執行緒（按鍵對應的動作按需臨時起執行緒，做完即結束）。

- **視窗模型**：`WS_POPUP` + `WS_THICKFRAME`，`WM_NCCALCSIZE` 回傳 0 讓客戶區鋪滿整個視窗，標題列、最小化/關閉按鈕全部自繪；`WM_NCHITTEST` 把頂部判為 `HTCAPTION`、四邊與四角判為縮放邊框，因此拖曳、貼邊、Snap Layouts 都是系統原生行為
- **視窗外觀**：`DwmSetWindowAttribute` 設定圓角偏好（`DWMWCP_ROUND`）、沉浸式深色模式與邊框色，失焦時也不會出現白邊
- **繪製**：`WM_PAINT` 中建立相容點陣圖，全部內容畫完後一次 `BitBlt` 到視窗，避免閃爍
- **動畫**：所有過渡量都是指數趨近（`v += (target - v) * (1 - exp(-speed * dt))`），由 15ms 計時器驅動，只在有動畫時存在，動畫結束立即 `KillTimer`，閒置 0% CPU
- **快速鍵動作**：命中快速鍵後只向視窗投遞訊息，實際動作（建檔案 / 啟動記事本）在臨時執行緒中執行，主執行緒不阻塞，因此連按不會排隊
- **狀態**：`App` 結構持有全部介面狀態，透過視窗使用者資料指標與視窗程序共享；設定變更時寫入登錄檔並即時同步鉤子與匣盤

## 渲染管線

- **技術棧**：純 GDI（`gdi32`），不使用 GDI+、Direct2D 或任何第三方 UI 函式庫
- **每幀流程**：`CreateCompatibleDC` + `CreateCompatibleBitmap`（依客戶區尺寸）→ 繪製 → `BitBlt` 到視窗 → `DeleteObject` + `DeleteDC`，緩衝不常駐，這也是記憶體能壓到幾 MB 的關鍵
- **圖元**：圓角矩形用 `RoundRect`（同時支援填色與 1px 描邊）、圓形用 `Ellipse`、線段用 `CreatePen` + `MoveToEx/LineTo`
- **漸層與陰影**：漸層按鈕在圓角路徑裁切區內繪製多條水平色帶（`BeginPath` / `EndPath` / `SelectClipPath`），陰影用數層逐漸變淡的圓角外擴模擬
- **透明度**：GDI 沒有 alpha 混合，所有半透明顏色在繪製前與目前背景色做一次逐通道混合（`blend`），既得到正確的視覺效果，又不引入額外依賴
- **文字**：`CreateFontW` + `DrawTextW`，開啟 ClearType 與 `DT_END_ELLIPSIS`；中文/日文/韓文用 `Microsoft YaHei UI`，其餘用 `Segoe UI`；文案過長時依可用寬度逐級縮小字級再省略
- **DPI**：Per-Monitor V2 感知，所有座標以邏輯像素為單位，繪製時統一乘以 `dpi / 96`；`WM_DPICHANGED` 與 `WM_DISPLAYCHANGE` 觸發版面重排與螢幕適配

## 全域快速鍵實作

- `SetWindowsHookExW(WH_KEYBOARD_LL)` 裝在獨立執行緒，執行緒優先順序 `THREAD_PRIORITY_TIME_CRITICAL`，安裝完成後主執行緒才繼續，確保任何程式裡的組合鍵都先被本程式處理
- 修飾鍵狀態自行維護：用原子位元遮罩記錄左右 Ctrl / Shift / Alt / Win 的按下情況，而不依賴 `GetAsyncKeyState`，避免 `Win` 按下時狀態不可靠
- `Win` 鍵被鉤子吞下後：命中自訂組合鍵就丟棄，單獨按 `Win` 時補發一次 `Win` 按下與放開（開始功能表照常開啟），其餘組合鍵（如 `Win+E`）則先補發 `Win` 再放行，因此不會把系統狀態弄亂
- 自身注入的按鍵帶有 `dwExtraInfo` 標記，鉤子中直接放行，避免自吞
- 命中後有 120ms 節流，長按不會連續刷出
- 建檔案時取前景視窗的 `HWND`：桌面（`Progman` / `WorkerW`）落到桌面目錄，檔案總管視窗透過 `IShellWindows` 列舉比對 `HWND` 並讀取目前路徑，取不到才回退桌面
- 開啟記事本用 `CreateProcessW` 啟動 `%SystemRoot%\System32\notepad.exe`（工作目錄留空，避免商店版記事本找不到相依 DLL），再短暫等待並用 `EnumWindows` 找到本次處理序的視窗置前

## 語言

- 20 種語言：簡體中文、繁體中文、English (US)、English (UK)、日本語、한국어、Français、Deutsch、Español、Italiano、Русский、Português (BR)、Polski、Nederlands、Türkçe、Українська、Tiếng Việt、Bahasa Indonesia、ไทย、हिन्दी
- 啟動時用 `GetUserDefaultLocaleName` 判斷系統區域（`zh-CN` 等 → 簡體中文，`zh-TW`/`zh-HK`/`zh-MO` → 繁體），再回退到 `GetUserDefaultUILanguage`；設定裡的 `language` 為 `auto` 時跟隨系統，手動選過語言後寫入對應代碼（例如 `zh_hk`），不再被系統語言覆蓋
- 版本資源內嵌 8 種語言的 `FileDescription`，工作管理員「處理程序」頁的程式名稱隨系統語言變化

## 效能與記憶體

| 指標                  | 實測                                       |
| ------------------- | ---------------------------------------- |
| 私有記憶體（PrivateBytes） | 約 1.5 MB                                 |
| 匣盤常駐、未開啟設定介面（靜止）    | 約 64–88 KB                              |
| 設定介面開啟但靜止           | 約 0.1–0.3 MB                            |
| 重繪峰值                | 約 1.5 MB（1060×620 的 32bpp 相容點陣圖，每幀建立後釋放） |
| 處理序數 / 子處理序         | 1 / 0                                    |
| 閒置 CPU              | 0%                                       |

## 建置

```
cargo build --release
```

產物：`target\release\text_document_creator.exe`

- 需要 Rust 工具鏈（MSVC 目標）與 Windows SDK（`build.rs` 自動尋找 `rc.exe` 編譯圖示與版本資源）
- `.cargo/config.toml` 啟用 `+crt-static`，配合 `opt-level="z"`、LTO、`panic="abort"`、strip 得到免依賴的單一 exe
- 圖示由 `assets/icon.ico`（16–256 共 9 個尺寸）編入 exe，工作列、檔案總管與工作管理員都顯示高解析度圖示

## 專案結構

| 路徑 | 作用 |
| --- | --- |
| `src/` | 全部 Rust 原始碼，零第三方相依，Win32 全部手寫 FFI |
| `src/main.rs` | 程式入口、視窗與訊息迴圈、匣盤圖示、單一實例、DPI 與主題變化、閒置記憶體回收 |
| `src/app.rs` | 設定介面：版面、命中測試、繪製、語言下拉與捲動、關於頁面、文字選取 |
| `src/render.rs` | 純 GDI 繪製層：文字、圓角、漸層、陰影、圖示、裁剪 |
| `src/theme.rs` | 深淺色調色盤、顏色混合、系統主題判定與主題轉場插值 |
| `src/anim.rs` | 指數逼近動畫與速度控制 |
| `src/i18n.rs` | 20 種語言文案、語言代碼表與系統語言偵測 |
| `src/config.rs` | `tdc_config.json` 讀寫、出廠預設值、開機自動啟動排程工作登記 |
| `src/hotkey.rs` | 全域鍵盤鉤子執行緒、按鍵擷取與綁定、建立文字文件、啟動記事本 |
| `src/ffi.rs` | Win32 常數與 user32 / kernel32 / shell32 等 API 宣告 |
| `src/sys.rs` | 結構定義與其餘 API 宣告（登錄檔、排程工作、COM、匣盤、處理程序） |
| `assets/` | 圖示資源 |
| `assets/icon.ico` | 編譯進 exe 的圖示，16 / 20 / 24 / 32 / 40 / 48 / 64 / 128 / 256 共 9 個尺寸 |
| `assets/icon_source.png` | 圖示的原始 256×256 PNG |
| `tools/` | 建置輔助腳本 |
| `tools/build_icon.mjs` | 從 `icon_source.png` 重新產生 `icon.ico`（Node 執行，無第三方相依） |
| `build.rs` | 建置腳本：尋找 Windows SDK 的 `rc.exe`，編譯 `app.rc` 並連結進 exe |
| `app.rc` | 圖示、資訊清單與 8 種語言的版本資訊資源 |
| `app.manifest` | `requireAdministrator`、Per-Monitor V2 DPI、Common Controls 6、longPathAware、UTF-8 代碼頁 |
| `.cargo/config.toml` | 靜態 CRT 與連結最佳化參數 |
| `Cargo.toml` / `Cargo.lock` | 套件資訊、release 最佳化設定與鎖定檔 |
| `README.md` / `README-EN.md` / `README-HK.md` | 簡體、英文、繁體說明 |
| `LICENSE` | 授權條款全文 |
## 資料位置

### 2.1.0 及之後

- 設定：exe 同目錄的 `tdc_config.json`（首次執行時寫出出廠設定，之後每次修改即時寫回；也可以直接編輯這個檔案，重新啟動程式後生效）

欄位說明：

| 欄位 | 含義 |
| --- | --- |
| `version` | 設定格式版本號，固定為 `1`；只在將來設定結構變化時用於遷移，平時不用改 |
| `enable_create` | 建立文字檔案開關，出廠 `true` |
| `enable_notepad` | 開啟記事本開關，出廠 `false` |
| `autostart` | 開機自動啟動開關，出廠 `false` |
| `language` | 介面語言，`auto` 表示跟隨系統 |
| `theme` | `0` 自動、`1` 淺色、`2` 深色 |
| `hotkey_create` / `hotkey_notepad` | 主功能快速鍵，如 `Win+F`、`Win+J` |
| `hotkey_toggle_create` / `hotkey_toggle_notepad` | 兩個開關快速鍵，留空表示未設定 |

`language` 可用的語言代碼：

| 代碼 | 語言 | 代碼 | 語言 |
| --- | --- | --- | --- |
| `auto` | 跟隨系統 | `ru_ru` | Русский |
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
- 開機自動啟動：排程工作 `\TextDocumentCreator`（`ONLOGON` + 最高權限，登入時靜默啟動、不彈 UAC；關閉開關會刪除該工作）

### 2.0.0 及更早

- 設定：登錄檔 `HKEY_CURRENT_USER\Software\TextDocumentCreator`，全部是 `REG_DWORD`：`EnableCreate`、`EnableNotepad`、`AutoStart`、`VkCreate`、`ModCreate`、`VkNotepad`、`ModNotepad`、`VkSwitchCreate`、`ModSwitchCreate`、`VkSwitchNotepad`、`ModSwitchNotepad`、`Language`、`LanguageAuto`、`ThemeMode`
- 開機自動啟動：排程工作 `TextDocumentCreator`；最早的兩版沒有排程工作，只寫 `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run` 下的 `Text_Document_Creator` 值（內容為 exe 完整路徑）
- 這些舊版沒有設定檔，升級到 2.1.0 後首次執行會重新寫出出廠設定，舊登錄檔項不再被讀取

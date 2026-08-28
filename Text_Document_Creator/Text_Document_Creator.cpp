#define WIN32_LEAN_AND_MEAN
#define _WIN32_WINNT 0x0601
#include "resource1.h"
#include <windows.h>
#include <shellapi.h>
#include <shlobj.h>
#include <shlwapi.h>
#include <commctrl.h>
#include <objbase.h>
#include <shlguid.h>
#include <exdisp.h>
#include <stdio.h>
#include <stdlib.h>

#pragma comment(lib, "shell32.lib")
#pragma comment(lib, "advapi32.lib")
#pragma comment(lib, "ole32.lib")
#pragma comment(lib, "oleaut32.lib")
#pragma comment(lib, "shlwapi.lib")
#pragma comment(lib, "comctl32.lib")

// ---------- 资源ID ----------
#define ICON_ID            IDI_ICON1
#define WM_TRAYICON        (WM_USER + 100)
#define ID_TRAY_SETTINGS   3004
#define ID_TRAY_ABOUT      3005
#define ID_TRAY_EXIT       3006

#define WM_CREATE_FILE     (WM_APP + 1)
#define WM_OPEN_NOTEPAD    (WM_APP + 2)

// ---------- 默认快捷键 ----------
#define DEF_VK_CREATE          VK_SPACE
#define DEF_MOD_CREATE         (MOD_CONTROL | MOD_SHIFT)
#define DEF_VK_NOTEPAD         'F'
#define DEF_MOD_NOTEPAD        (MOD_CONTROL | MOD_SHIFT)
#define DEF_VK_SWITCH_CREATE   0
#define DEF_MOD_SWITCH_CREATE  0
#define DEF_VK_SWITCH_NOTEPAD  0
#define DEF_MOD_SWITCH_NOTEPAD 0

// ---------- 语言常量 ----------
#define LANG_SIMPLIFIED_CHINESE  0
#define LANG_TRADITIONAL_CHINESE 1
#define LANG_ENGLISH_US          2

// ---------- 全局变量 ----------
HINSTANCE g_hInst;
HWND      g_hWnd;
HWND      g_hSettingsDlg = NULL;
HHOOK     g_hKeyboardHook = NULL;

static BOOL g_bAboutMode = FALSE;

// ---------- 配置结构 ----------
typedef struct {
    BOOL    bEnableCreate;
    BOOL    bEnableNotepad;
    BOOL    bAutoStart;
    UINT    vkCreate;
    UINT    modCreate;
    UINT    vkNotepad;
    UINT    modNotepad;
    UINT    vkSwitchCreate;
    UINT    modSwitchCreate;
    UINT    vkSwitchNotepad;
    UINT    modSwitchNotepad;
    int     language;
} Settings;

Settings g_Settings;

// ---------- 捕获按键状态 ----------
typedef struct {
    BOOL    bCapturing;
    HWND    hEdit;
    UINT    oldVk;
    UINT    oldMod;
} CaptureState;

static CaptureState g_capture = { FALSE, NULL, 0, 0 };

// ---------- 注册表配置键 ----------
#define REG_KEY_SETTINGS L"Software\\TextDocumentCreator"
#define REG_VAL_ENABLE_CREATE         L"EnableCreate"
#define REG_VAL_ENABLE_NOTEPAD        L"EnableNotepad"
#define REG_VAL_AUTOSTART             L"AutoStart"
#define REG_VAL_VK_CREATE             L"VkCreate"
#define REG_VAL_MOD_CREATE            L"ModCreate"
#define REG_VAL_VK_NOTEPAD            L"VkNotepad"
#define REG_VAL_MOD_NOTEPAD           L"ModNotepad"
#define REG_VAL_VK_SWITCH_CREATE      L"VkSwitchCreate"
#define REG_VAL_MOD_SWITCH_CREATE     L"ModSwitchCreate"
#define REG_VAL_VK_SWITCH_NOTEPAD     L"VkSwitchNotepad"
#define REG_VAL_MOD_SWITCH_NOTEPAD    L"ModSwitchNotepad"
#define REG_VAL_LANGUAGE              L"Language"

// ---------- 初始化配置 ----------
static void InitSettings() {
    g_Settings.bEnableCreate = TRUE;
    g_Settings.bEnableNotepad = TRUE;
    g_Settings.bAutoStart = FALSE;
    g_Settings.vkCreate = DEF_VK_CREATE;
    g_Settings.modCreate = DEF_MOD_CREATE;
    g_Settings.vkNotepad = DEF_VK_NOTEPAD;
    g_Settings.modNotepad = DEF_MOD_NOTEPAD;
    g_Settings.vkSwitchCreate = DEF_VK_SWITCH_CREATE;
    g_Settings.modSwitchCreate = DEF_MOD_SWITCH_CREATE;
    g_Settings.vkSwitchNotepad = DEF_VK_SWITCH_NOTEPAD;
    g_Settings.modSwitchNotepad = DEF_MOD_SWITCH_NOTEPAD;
    g_Settings.language = LANG_SIMPLIFIED_CHINESE;
}

// ---------- 读写配置 ----------
static void LoadSettings() {
    InitSettings();
    HKEY hKey;
    if (RegOpenKeyExW(HKEY_CURRENT_USER, REG_KEY_SETTINGS, 0, KEY_READ, &hKey) == ERROR_SUCCESS) {
        DWORD val, size = sizeof(DWORD);
        if (RegQueryValueExW(hKey, REG_VAL_ENABLE_CREATE, NULL, NULL, (BYTE*)&val, &size) == ERROR_SUCCESS)
            g_Settings.bEnableCreate = val ? TRUE : FALSE;
        if (RegQueryValueExW(hKey, REG_VAL_ENABLE_NOTEPAD, NULL, NULL, (BYTE*)&val, &size) == ERROR_SUCCESS)
            g_Settings.bEnableNotepad = val ? TRUE : FALSE;
        if (RegQueryValueExW(hKey, REG_VAL_AUTOSTART, NULL, NULL, (BYTE*)&val, &size) == ERROR_SUCCESS)
            g_Settings.bAutoStart = val ? TRUE : FALSE;
        if (RegQueryValueExW(hKey, REG_VAL_VK_CREATE, NULL, NULL, (BYTE*)&val, &size) == ERROR_SUCCESS)
            g_Settings.vkCreate = (UINT)val;
        if (RegQueryValueExW(hKey, REG_VAL_MOD_CREATE, NULL, NULL, (BYTE*)&val, &size) == ERROR_SUCCESS)
            g_Settings.modCreate = (UINT)val;
        if (RegQueryValueExW(hKey, REG_VAL_VK_NOTEPAD, NULL, NULL, (BYTE*)&val, &size) == ERROR_SUCCESS)
            g_Settings.vkNotepad = (UINT)val;
        if (RegQueryValueExW(hKey, REG_VAL_MOD_NOTEPAD, NULL, NULL, (BYTE*)&val, &size) == ERROR_SUCCESS)
            g_Settings.modNotepad = (UINT)val;
        if (RegQueryValueExW(hKey, REG_VAL_VK_SWITCH_CREATE, NULL, NULL, (BYTE*)&val, &size) == ERROR_SUCCESS)
            g_Settings.vkSwitchCreate = (UINT)val;
        if (RegQueryValueExW(hKey, REG_VAL_MOD_SWITCH_CREATE, NULL, NULL, (BYTE*)&val, &size) == ERROR_SUCCESS)
            g_Settings.modSwitchCreate = (UINT)val;
        if (RegQueryValueExW(hKey, REG_VAL_VK_SWITCH_NOTEPAD, NULL, NULL, (BYTE*)&val, &size) == ERROR_SUCCESS)
            g_Settings.vkSwitchNotepad = (UINT)val;
        if (RegQueryValueExW(hKey, REG_VAL_MOD_SWITCH_NOTEPAD, NULL, NULL, (BYTE*)&val, &size) == ERROR_SUCCESS)
            g_Settings.modSwitchNotepad = (UINT)val;
        if (RegQueryValueExW(hKey, REG_VAL_LANGUAGE, NULL, NULL, (BYTE*)&val, &size) == ERROR_SUCCESS)
            g_Settings.language = (int)val;
        RegCloseKey(hKey);
    }
}

static void SaveSettings() {
    HKEY hKey;
    if (RegCreateKeyExW(HKEY_CURRENT_USER, REG_KEY_SETTINGS, 0, NULL, 0, KEY_WRITE, NULL, &hKey, NULL) == ERROR_SUCCESS) {
        DWORD val = g_Settings.bEnableCreate ? 1 : 0;
        RegSetValueExW(hKey, REG_VAL_ENABLE_CREATE, 0, REG_DWORD, (BYTE*)&val, sizeof(DWORD));
        val = g_Settings.bEnableNotepad ? 1 : 0;
        RegSetValueExW(hKey, REG_VAL_ENABLE_NOTEPAD, 0, REG_DWORD, (BYTE*)&val, sizeof(DWORD));
        val = g_Settings.bAutoStart ? 1 : 0;
        RegSetValueExW(hKey, REG_VAL_AUTOSTART, 0, REG_DWORD, (BYTE*)&val, sizeof(DWORD));
        val = g_Settings.vkCreate;
        RegSetValueExW(hKey, REG_VAL_VK_CREATE, 0, REG_DWORD, (BYTE*)&val, sizeof(DWORD));
        val = g_Settings.modCreate;
        RegSetValueExW(hKey, REG_VAL_MOD_CREATE, 0, REG_DWORD, (BYTE*)&val, sizeof(DWORD));
        val = g_Settings.vkNotepad;
        RegSetValueExW(hKey, REG_VAL_VK_NOTEPAD, 0, REG_DWORD, (BYTE*)&val, sizeof(DWORD));
        val = g_Settings.modNotepad;
        RegSetValueExW(hKey, REG_VAL_MOD_NOTEPAD, 0, REG_DWORD, (BYTE*)&val, sizeof(DWORD));
        val = g_Settings.vkSwitchCreate;
        RegSetValueExW(hKey, REG_VAL_VK_SWITCH_CREATE, 0, REG_DWORD, (BYTE*)&val, sizeof(DWORD));
        val = g_Settings.modSwitchCreate;
        RegSetValueExW(hKey, REG_VAL_MOD_SWITCH_CREATE, 0, REG_DWORD, (BYTE*)&val, sizeof(DWORD));
        val = g_Settings.vkSwitchNotepad;
        RegSetValueExW(hKey, REG_VAL_VK_SWITCH_NOTEPAD, 0, REG_DWORD, (BYTE*)&val, sizeof(DWORD));
        val = g_Settings.modSwitchNotepad;
        RegSetValueExW(hKey, REG_VAL_MOD_SWITCH_NOTEPAD, 0, REG_DWORD, (BYTE*)&val, sizeof(DWORD));
        val = g_Settings.language;
        RegSetValueExW(hKey, REG_VAL_LANGUAGE, 0, REG_DWORD, (BYTE*)&val, sizeof(DWORD));
        RegCloseKey(hKey);
    }
}

// ---------- 开机自启动 ----------
static void ApplyAutoStart(BOOL enable) {
    HKEY hKey;
    if (RegOpenKeyExW(HKEY_CURRENT_USER, L"Software\\Microsoft\\Windows\\CurrentVersion\\Run", 0, KEY_SET_VALUE, &hKey) != ERROR_SUCCESS)
        return;
    if (enable) {
        wchar_t exePath[MAX_PATH];
        GetModuleFileNameW(NULL, exePath, MAX_PATH);
        RegSetValueExW(hKey, L"Text_Document_Creator", 0, REG_SZ, (BYTE*)exePath, (DWORD)(wcslen(exePath) + 1) * sizeof(wchar_t));
    }
    else {
        RegDeleteValueW(hKey, L"Text_Document_Creator");
    }
    RegCloseKey(hKey);
}

// ---------- 多语言字符串 ----------
static const wchar_t* GetString(int lang, const wchar_t* key) {
    static const wchar_t* strings_simplified[] = {
        L"设置", L"功能开关", L"创建文本文档", L"打开记事本", L"开机自启动",
        L"快捷键设置", L"创建文本文档", L"打开记事本", L"开关-创建文档", L"开关-打开记事本",
        L"清除", L"重置", L"重置全部", L"语言", L"设置", L"关于", L"退出",
        L"【文本文档创建器】\r\n一个可以用快捷键快速创建新建文本文档的自制小工具。\r\n也可以用快捷键快速打开记事本。\r\n\r\n可在本软件设置中自定义快捷键。\r\n\r\n版权：Torin Linsley\r\n开源协议：LGPL v2.1\r\n\r\n项目链接：https://github.com/TorinLinsley/Text_Document_Creator",
        L"返回"
    };
    static const wchar_t* strings_traditional[] = {
        L"設定", L"功能開關", L"建立文字文件", L"開啟記事本", L"開機自啟動",
        L"快速鍵設定", L"建立文字文件", L"開啟記事本", L"開關-建立文件", L"開關-開啟記事本",
        L"清除", L"重設", L"重設全部", L"語言", L"設定", L"關於", L"退出",
        L"【文字檔案建立工具】\r\n一個能用快速鍵快速建立新文字檔的自製小工具。\r\n也能用快速鍵快速開啟記事本。\r\n\r\n可在本軟體設定中自訂快速鍵。\r\n\r\n版權所有：Torin Linsley\r\n開放原始碼授權：LGPL v2.1\r\n\r\n項目鏈接：https://github.com/TorinLinsley/Text_Document_Creator",
        L"返回"
    };
    static const wchar_t* strings_english[] = {
        L"Settings", L"Function Switches", L"Create Text File", L"Open Notepad", L"Auto Start",
        L"Shortcut Keys", L"Create File", L"Open Notepad", L"Switch Create", L"Switch Notepad",
        L"Clear", L"Reset", L"Reset All", L"Language", L"Settings", L"About", L"Exit",
        L"【Text Document Creator】\r\nA custom utility that enables you to quickly create new text files using keyboard shortcuts. It can also be used to quickly open Notepad with a hotkey.\r\n\r\nKeyboard shortcuts can be customized in the software's settings.\r\n\r\nCopyright: Torin Linsley\r\nOpen Source License: LGPL v2.1\r\n\r\nProject repository link: https://github.com/TorinLinsley/Text_Document_Creator",
        L"Back"
    };

    const wchar_t** strings = NULL;
    if (lang == LANG_SIMPLIFIED_CHINESE) strings = strings_simplified;
    else if (lang == LANG_TRADITIONAL_CHINESE) strings = strings_traditional;
    else strings = strings_english;

    int index = -1;
    if (wcscmp(key, L"SETTINGS_TITLE") == 0) index = 0;
    else if (wcscmp(key, L"GROUP_SWITCHES") == 0) index = 1;
    else if (wcscmp(key, L"CHECK_CREATE") == 0) index = 2;
    else if (wcscmp(key, L"CHECK_NOTEPAD") == 0) index = 3;
    else if (wcscmp(key, L"CHECK_AUTOSTART") == 0) index = 4;
    else if (wcscmp(key, L"GROUP_SHORTCUTS") == 0) index = 5;
    else if (wcscmp(key, L"LABEL_CREATE") == 0) index = 6;
    else if (wcscmp(key, L"LABEL_NOTEPAD") == 0) index = 7;
    else if (wcscmp(key, L"LABEL_SWITCH_CREATE") == 0) index = 8;
    else if (wcscmp(key, L"LABEL_SWITCH_NOTEPAD") == 0) index = 9;
    else if (wcscmp(key, L"BTN_CLEAR") == 0) index = 10;
    else if (wcscmp(key, L"BTN_RESET") == 0) index = 11;
    else if (wcscmp(key, L"BTN_RESET_ALL") == 0) index = 12;
    else if (wcscmp(key, L"LANG_LABEL") == 0) index = 13;
    else if (wcscmp(key, L"TRAY_SETTINGS") == 0) index = 14;
    else if (wcscmp(key, L"TRAY_ABOUT") == 0) index = 15;
    else if (wcscmp(key, L"TRAY_EXIT") == 0) index = 16;
    else if (wcscmp(key, L"ABOUT_CONTENT") == 0) index = 17;
    else if (wcscmp(key, L"BACK_BUTTON") == 0) index = 18;

    if (index >= 0 && index <= 18) return strings[index];
    return L"";
}

// ---------- 键名转文本 ----------
static const wchar_t* GetKeyName(UINT vk) {
    if (vk == VK_SPACE) return L"Space";
    if (vk >= 'A' && vk <= 'Z') {
        static wchar_t buf[2] = { 0 };
        buf[0] = (wchar_t)vk;
        return buf;
    }
    if (vk >= '0' && vk <= '9') {
        static wchar_t buf[2] = { 0 };
        buf[0] = (wchar_t)vk;
        return buf;
    }
    if (vk >= VK_F1 && vk <= VK_F24) {
        static wchar_t buf[8];
        swprintf_s(buf, 8, L"F%d", vk - VK_F1 + 1);
        return buf;
    }
    switch (vk) {
    case VK_BACK: return L"Backspace";
    case VK_TAB: return L"Tab";
    case VK_CLEAR: return L"Clear";
    case VK_RETURN: return L"Enter";
    case VK_PAUSE: return L"Pause";
    case VK_CAPITAL: return L"Caps Lock";
    case VK_ESCAPE: return L"Esc";
    case VK_PRIOR: return L"PageUp";
    case VK_NEXT: return L"PageDown";
    case VK_END: return L"End";
    case VK_HOME: return L"Home";
    case VK_LEFT: return L"Left";
    case VK_UP: return L"Up";
    case VK_RIGHT: return L"Right";
    case VK_DOWN: return L"Down";
    case VK_SELECT: return L"Select";
    case VK_PRINT: return L"Print";
    case VK_EXECUTE: return L"Execute";
    case VK_SNAPSHOT: return L"PrintScreen";
    case VK_INSERT: return L"Insert";
    case VK_DELETE: return L"Delete";
    case VK_HELP: return L"Help";
    case VK_LWIN: return L"Win";
    case VK_RWIN: return L"Win";
    case VK_APPS: return L"Apps";
    case VK_SLEEP: return L"Sleep";
    case VK_NUMPAD0: return L"Num0";
    case VK_NUMPAD1: return L"Num1";
    case VK_NUMPAD2: return L"Num2";
    case VK_NUMPAD3: return L"Num3";
    case VK_NUMPAD4: return L"Num4";
    case VK_NUMPAD5: return L"Num5";
    case VK_NUMPAD6: return L"Num6";
    case VK_NUMPAD7: return L"Num7";
    case VK_NUMPAD8: return L"Num8";
    case VK_NUMPAD9: return L"Num9";
    case VK_MULTIPLY: return L"Num*";
    case VK_ADD: return L"Num+";
    case VK_SEPARATOR: return L"Separator";
    case VK_SUBTRACT: return L"Num-";
    case VK_DECIMAL: return L"Num.";
    case VK_DIVIDE: return L"Num/";
    case VK_NUMLOCK: return L"NumLock";
    case VK_SCROLL: return L"ScrollLock";
    case VK_BROWSER_BACK: return L"BrowserBack";
    case VK_BROWSER_FORWARD: return L"BrowserForward";
    case VK_BROWSER_REFRESH: return L"BrowserRefresh";
    case VK_BROWSER_STOP: return L"BrowserStop";
    case VK_BROWSER_SEARCH: return L"BrowserSearch";
    case VK_BROWSER_FAVORITES: return L"BrowserFavorites";
    case VK_BROWSER_HOME: return L"BrowserHome";
    case VK_VOLUME_MUTE: return L"VolumeMute";
    case VK_VOLUME_DOWN: return L"VolumeDown";
    case VK_VOLUME_UP: return L"VolumeUp";
    case VK_MEDIA_NEXT_TRACK: return L"NextTrack";
    case VK_MEDIA_PREV_TRACK: return L"PrevTrack";
    case VK_MEDIA_STOP: return L"MediaStop";
    case VK_MEDIA_PLAY_PAUSE: return L"PlayPause";
    case VK_LAUNCH_MAIL: return L"LaunchMail";
    case VK_LAUNCH_MEDIA_SELECT: return L"MediaSelect";
    case VK_LAUNCH_APP1: return L"App1";
    case VK_LAUNCH_APP2: return L"App2";
    case VK_OEM_1: return L";";
    case VK_OEM_PLUS: return L"+";
    case VK_OEM_COMMA: return L",";
    case VK_OEM_MINUS: return L"-";
    case VK_OEM_PERIOD: return L".";
    case VK_OEM_2: return L"/";
    case VK_OEM_3: return L"`";
    case VK_OEM_4: return L"[";
    case VK_OEM_5: return L"\\";
    case VK_OEM_6: return L"]";
    case VK_OEM_7: return L"'";
    case VK_OEM_102: return L"\\";
    default: {
        static wchar_t buf[16];
        swprintf_s(buf, 16, L"VK-%d", vk);
        return buf;
    }
    }
}

static void ShortcutToText(UINT vk, UINT mod, wchar_t* buf, int bufLen) {
    if (vk == 0) {
        wcscpy_s(buf, bufLen, L"");
        return;
    }
    wchar_t result[128] = { 0 };
    if (mod & MOD_CONTROL) wcscat_s(result, L"Ctrl+");
    if (mod & MOD_SHIFT)   wcscat_s(result, L"Shift+");
    if (mod & MOD_ALT)     wcscat_s(result, L"Alt+");
    if (mod & MOD_WIN)     wcscat_s(result, L"Win+");
    wcscat_s(result, GetKeyName(vk));
    wcscpy_s(buf, bufLen, result);
}

// ---------- 更新界面文字 ----------
static void UpdateDialogTexts(HWND hDlg) {
    int lang = g_Settings.language;
    SetWindowTextW(hDlg, GetString(lang, L"SETTINGS_TITLE"));
    SetDlgItemTextW(hDlg, IDC_LANG_LABEL, GetString(lang, L"LANG_LABEL"));
    SetDlgItemTextW(hDlg, IDC_STATIC_CREATE_LABEL, GetString(lang, L"LABEL_CREATE"));
    SetDlgItemTextW(hDlg, IDC_STATIC_NOTEPAD_LABEL, GetString(lang, L"LABEL_NOTEPAD"));
    SetDlgItemTextW(hDlg, IDC_STATIC_SWITCH_CREATE_LABEL, GetString(lang, L"LABEL_SWITCH_CREATE"));
    SetDlgItemTextW(hDlg, IDC_STATIC_SWITCH_NOTEPAD_LABEL, GetString(lang, L"LABEL_SWITCH_NOTEPAD"));
    SetDlgItemTextW(hDlg, IDC_BTN_CLEAR_CREATE, GetString(lang, L"BTN_CLEAR"));
    SetDlgItemTextW(hDlg, IDC_BTN_CLEAR_NOTEPAD, GetString(lang, L"BTN_CLEAR"));
    SetDlgItemTextW(hDlg, IDC_BTN_CLEAR_SWITCH_CREATE, GetString(lang, L"BTN_CLEAR"));
    SetDlgItemTextW(hDlg, IDC_BTN_CLEAR_SWITCH_NOTEPAD, GetString(lang, L"BTN_CLEAR"));
    SetDlgItemTextW(hDlg, IDC_BTN_RESET_CREATE, GetString(lang, L"BTN_RESET"));
    SetDlgItemTextW(hDlg, IDC_BTN_RESET_NOTEPAD, GetString(lang, L"BTN_RESET"));
    SetDlgItemTextW(hDlg, IDC_BTN_RESET_SWITCH_CREATE, GetString(lang, L"BTN_RESET"));
    SetDlgItemTextW(hDlg, IDC_BTN_RESET_SWITCH_NOTEPAD, GetString(lang, L"BTN_RESET"));
    SetDlgItemTextW(hDlg, IDC_BTN_RESET_ALL, GetString(lang, L"BTN_RESET_ALL"));
    SetDlgItemTextW(hDlg, IDC_CHECK_ENABLE_CREATE, GetString(lang, L"CHECK_CREATE"));
    SetDlgItemTextW(hDlg, IDC_CHECK_ENABLE_NOTEPAD, GetString(lang, L"CHECK_NOTEPAD"));
    SetDlgItemTextW(hDlg, IDC_CHECK_AUTOSTART, GetString(lang, L"CHECK_AUTOSTART"));
    SetDlgItemTextW(hDlg, IDC_GROUP_SWITCHES, GetString(lang, L"GROUP_SWITCHES"));
    SetDlgItemTextW(hDlg, IDC_GROUP_SHORTCUTS, GetString(lang, L"GROUP_SHORTCUTS"));
    // 关于按钮：根据模式显示"关于"或"返回"
    if (g_bAboutMode) {
        SetDlgItemTextW(hDlg, IDC_BTN_ABOUT, GetString(lang, L"BACK_BUTTON"));
    }
    else {
        SetDlgItemTextW(hDlg, IDC_BTN_ABOUT, GetString(lang, L"TRAY_ABOUT"));
    }
    // 如果处于关于模式，更新关于文本框内容
    if (g_bAboutMode) {
        SetDlgItemTextW(hDlg, IDC_EDIT_ABOUT, GetString(lang, L"ABOUT_CONTENT"));
    }
}

// ---------- 切换关于模式 ----------
static void ToggleAboutMode(HWND hDlg) {
    g_bAboutMode = !g_bAboutMode;
    int lang = g_Settings.language;

    // 获取所有需要显示/隐藏的控件
    HWND hLangLabel = GetDlgItem(hDlg, IDC_LANG_LABEL);
    HWND hLangCombo = GetDlgItem(hDlg, IDC_COMBO_LANGUAGE);
    HWND hGroupSwitches = GetDlgItem(hDlg, IDC_GROUP_SWITCHES);
    HWND hCheckCreate = GetDlgItem(hDlg, IDC_CHECK_ENABLE_CREATE);
    HWND hCheckNotepad = GetDlgItem(hDlg, IDC_CHECK_ENABLE_NOTEPAD);
    HWND hCheckAutostart = GetDlgItem(hDlg, IDC_CHECK_AUTOSTART);
    HWND hGroupShortcuts = GetDlgItem(hDlg, IDC_GROUP_SHORTCUTS);
    HWND hStaticCreate = GetDlgItem(hDlg, IDC_STATIC_CREATE_LABEL);
    HWND hEditCreate = GetDlgItem(hDlg, IDC_EDIT_CREATE_KEY);
    HWND hBtnClearCreate = GetDlgItem(hDlg, IDC_BTN_CLEAR_CREATE);
    HWND hBtnResetCreate = GetDlgItem(hDlg, IDC_BTN_RESET_CREATE);
    HWND hStaticNotepad = GetDlgItem(hDlg, IDC_STATIC_NOTEPAD_LABEL);
    HWND hEditNotepad = GetDlgItem(hDlg, IDC_EDIT_NOTEPAD_KEY);
    HWND hBtnClearNotepad = GetDlgItem(hDlg, IDC_BTN_CLEAR_NOTEPAD);
    HWND hBtnResetNotepad = GetDlgItem(hDlg, IDC_BTN_RESET_NOTEPAD);
    HWND hStaticSwitchCreate = GetDlgItem(hDlg, IDC_STATIC_SWITCH_CREATE_LABEL);
    HWND hEditSwitchCreate = GetDlgItem(hDlg, IDC_EDIT_SWITCH_CREATE_KEY);
    HWND hBtnClearSwitchCreate = GetDlgItem(hDlg, IDC_BTN_CLEAR_SWITCH_CREATE);
    HWND hBtnResetSwitchCreate = GetDlgItem(hDlg, IDC_BTN_RESET_SWITCH_CREATE);
    HWND hStaticSwitchNotepad = GetDlgItem(hDlg, IDC_STATIC_SWITCH_NOTEPAD_LABEL);
    HWND hEditSwitchNotepad = GetDlgItem(hDlg, IDC_EDIT_SWITCH_NOTEPAD_KEY);
    HWND hBtnClearSwitchNotepad = GetDlgItem(hDlg, IDC_BTN_CLEAR_SWITCH_NOTEPAD);
    HWND hBtnResetSwitchNotepad = GetDlgItem(hDlg, IDC_BTN_RESET_SWITCH_NOTEPAD);
    HWND hBtnResetAll = GetDlgItem(hDlg, IDC_BTN_RESET_ALL);
    HWND hStaticLine = GetDlgItem(hDlg, IDC_STATIC);       // 水平分割线
    HWND hEditAbout = GetDlgItem(hDlg, IDC_EDIT_ABOUT);
    // 关于按钮不隐藏，但文字会在 UpdateDialogTexts 中更新

    // 根据模式显示/隐藏（语言标签和下拉框始终显示）
    BOOL showNormal = !g_bAboutMode;
    // 语言标签和下拉框始终显示，不隐藏
    ShowWindow(hLangLabel, SW_SHOW);
    ShowWindow(hLangCombo, SW_SHOW);

    // 其他控件按模式切换
    ShowWindow(hGroupSwitches, showNormal ? SW_SHOW : SW_HIDE);
    ShowWindow(hCheckCreate, showNormal ? SW_SHOW : SW_HIDE);
    ShowWindow(hCheckNotepad, showNormal ? SW_SHOW : SW_HIDE);
    ShowWindow(hCheckAutostart, showNormal ? SW_SHOW : SW_HIDE);
    ShowWindow(hGroupShortcuts, showNormal ? SW_SHOW : SW_HIDE);
    ShowWindow(hStaticCreate, showNormal ? SW_SHOW : SW_HIDE);
    ShowWindow(hEditCreate, showNormal ? SW_SHOW : SW_HIDE);
    ShowWindow(hBtnClearCreate, showNormal ? SW_SHOW : SW_HIDE);
    ShowWindow(hBtnResetCreate, showNormal ? SW_SHOW : SW_HIDE);
    ShowWindow(hStaticNotepad, showNormal ? SW_SHOW : SW_HIDE);
    ShowWindow(hEditNotepad, showNormal ? SW_SHOW : SW_HIDE);
    ShowWindow(hBtnClearNotepad, showNormal ? SW_SHOW : SW_HIDE);
    ShowWindow(hBtnResetNotepad, showNormal ? SW_SHOW : SW_HIDE);
    ShowWindow(hStaticSwitchCreate, showNormal ? SW_SHOW : SW_HIDE);
    ShowWindow(hEditSwitchCreate, showNormal ? SW_SHOW : SW_HIDE);
    ShowWindow(hBtnClearSwitchCreate, showNormal ? SW_SHOW : SW_HIDE);
    ShowWindow(hBtnResetSwitchCreate, showNormal ? SW_SHOW : SW_HIDE);
    ShowWindow(hStaticSwitchNotepad, showNormal ? SW_SHOW : SW_HIDE);
    ShowWindow(hEditSwitchNotepad, showNormal ? SW_SHOW : SW_HIDE);
    ShowWindow(hBtnClearSwitchNotepad, showNormal ? SW_SHOW : SW_HIDE);
    ShowWindow(hBtnResetSwitchNotepad, showNormal ? SW_SHOW : SW_HIDE);
    ShowWindow(hBtnResetAll, showNormal ? SW_SHOW : SW_HIDE);
    ShowWindow(hStaticLine, showNormal ? SW_SHOW : SW_HIDE);
    ShowWindow(hEditAbout, g_bAboutMode ? SW_SHOW : SW_HIDE);

    // 更新关于按钮文字和窗口标题（由 UpdateDialogTexts 统一处理）
    UpdateDialogTexts(hDlg);
}

// ---------- 窗口居中 ----------
static void CenterWindow(HWND hWnd) {
    int screenWidth = GetSystemMetrics(SM_CXSCREEN);
    int screenHeight = GetSystemMetrics(SM_CYSCREEN);
    RECT rc;
    GetWindowRect(hWnd, &rc);
    int winWidth = rc.right - rc.left;
    int winHeight = rc.bottom - rc.top;
    int x = (screenWidth - winWidth) / 2;
    int y = (screenHeight - winHeight) / 2;
    SetWindowPos(hWnd, NULL, x, y, 0, 0, SWP_NOSIZE | SWP_NOZORDER);
}

// ---------- 编辑框子类过程 ----------
static LRESULT CALLBACK EditSubclassProc(HWND hEdit, UINT msg, WPARAM wParam, LPARAM lParam, UINT_PTR uIdSubclass, DWORD_PTR dwRefData) {
    if (msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN) {
        if (g_capture.bCapturing && g_capture.hEdit == hEdit) {
            UINT vk = (UINT)wParam;
            if (vk == VK_CONTROL || vk == VK_SHIFT || vk == VK_MENU || vk == VK_LWIN || vk == VK_RWIN)
                return 0;
            if (vk == VK_ESCAPE) {
                UINT* pVk = NULL, * pMod = NULL;
                if (hEdit == GetDlgItem(g_hSettingsDlg, IDC_EDIT_CREATE_KEY)) {
                    pVk = &g_Settings.vkCreate; pMod = &g_Settings.modCreate;
                }
                else if (hEdit == GetDlgItem(g_hSettingsDlg, IDC_EDIT_NOTEPAD_KEY)) {
                    pVk = &g_Settings.vkNotepad; pMod = &g_Settings.modNotepad;
                }
                else if (hEdit == GetDlgItem(g_hSettingsDlg, IDC_EDIT_SWITCH_CREATE_KEY)) {
                    pVk = &g_Settings.vkSwitchCreate; pMod = &g_Settings.modSwitchCreate;
                }
                else if (hEdit == GetDlgItem(g_hSettingsDlg, IDC_EDIT_SWITCH_NOTEPAD_KEY)) {
                    pVk = &g_Settings.vkSwitchNotepad; pMod = &g_Settings.modSwitchNotepad;
                }
                if (pVk) {
                    *pVk = g_capture.oldVk;
                    *pMod = g_capture.oldMod;
                    wchar_t buf[64];
                    ShortcutToText(*pVk, *pMod, buf, 64);
                    SetWindowTextW(hEdit, buf);
                }
                g_capture.bCapturing = FALSE;
                g_capture.hEdit = NULL;
                SetFocus(g_hSettingsDlg);
                return 0;
            }
            UINT mod = 0;
            if (GetAsyncKeyState(VK_CONTROL) & 0x8000) mod |= MOD_CONTROL;
            if (GetAsyncKeyState(VK_SHIFT) & 0x8000) mod |= MOD_SHIFT;
            if (GetAsyncKeyState(VK_MENU) & 0x8000) mod |= MOD_ALT;
            if (GetAsyncKeyState(VK_LWIN) & 0x8000 || GetAsyncKeyState(VK_RWIN) & 0x8000) mod |= MOD_WIN;

            UINT* pVk = NULL, * pMod = NULL;
            if (hEdit == GetDlgItem(g_hSettingsDlg, IDC_EDIT_CREATE_KEY)) {
                pVk = &g_Settings.vkCreate; pMod = &g_Settings.modCreate;
            }
            else if (hEdit == GetDlgItem(g_hSettingsDlg, IDC_EDIT_NOTEPAD_KEY)) {
                pVk = &g_Settings.vkNotepad; pMod = &g_Settings.modNotepad;
            }
            else if (hEdit == GetDlgItem(g_hSettingsDlg, IDC_EDIT_SWITCH_CREATE_KEY)) {
                pVk = &g_Settings.vkSwitchCreate; pMod = &g_Settings.modSwitchCreate;
            }
            else if (hEdit == GetDlgItem(g_hSettingsDlg, IDC_EDIT_SWITCH_NOTEPAD_KEY)) {
                pVk = &g_Settings.vkSwitchNotepad; pMod = &g_Settings.modSwitchNotepad;
            }
            if (pVk) {
                *pVk = vk;
                *pMod = mod;
                wchar_t buf[64];
                ShortcutToText(*pVk, *pMod, buf, 64);
                SetWindowTextW(hEdit, buf);
                SaveSettings();
            }
            g_capture.bCapturing = FALSE;
            g_capture.hEdit = NULL;
            SetFocus(g_hSettingsDlg);
            return 0;
        }
    }
    if (msg == WM_SETFOCUS) {
        HideCaret(hEdit);
        if (!g_capture.bCapturing) {
            g_capture.bCapturing = TRUE;
            g_capture.hEdit = hEdit;
            UINT* pVk = NULL, * pMod = NULL;
            if (hEdit == GetDlgItem(g_hSettingsDlg, IDC_EDIT_CREATE_KEY)) {
                pVk = &g_Settings.vkCreate; pMod = &g_Settings.modCreate;
            }
            else if (hEdit == GetDlgItem(g_hSettingsDlg, IDC_EDIT_NOTEPAD_KEY)) {
                pVk = &g_Settings.vkNotepad; pMod = &g_Settings.modNotepad;
            }
            else if (hEdit == GetDlgItem(g_hSettingsDlg, IDC_EDIT_SWITCH_CREATE_KEY)) {
                pVk = &g_Settings.vkSwitchCreate; pMod = &g_Settings.modSwitchCreate;
            }
            else if (hEdit == GetDlgItem(g_hSettingsDlg, IDC_EDIT_SWITCH_NOTEPAD_KEY)) {
                pVk = &g_Settings.vkSwitchNotepad; pMod = &g_Settings.modSwitchNotepad;
            }
            if (pVk) {
                g_capture.oldVk = *pVk;
                g_capture.oldMod = *pMod;
            }
            SetWindowTextW(hEdit, L"按下快捷键...");
        }
        return DefSubclassProc(hEdit, msg, wParam, lParam);
    }
    if (msg == WM_KILLFOCUS) {
        ShowCaret(hEdit);
        if (g_capture.bCapturing && g_capture.hEdit == hEdit) {
            UINT* pVk = NULL, * pMod = NULL;
            if (hEdit == GetDlgItem(g_hSettingsDlg, IDC_EDIT_CREATE_KEY)) {
                pVk = &g_Settings.vkCreate; pMod = &g_Settings.modCreate;
            }
            else if (hEdit == GetDlgItem(g_hSettingsDlg, IDC_EDIT_NOTEPAD_KEY)) {
                pVk = &g_Settings.vkNotepad; pMod = &g_Settings.modNotepad;
            }
            else if (hEdit == GetDlgItem(g_hSettingsDlg, IDC_EDIT_SWITCH_CREATE_KEY)) {
                pVk = &g_Settings.vkSwitchCreate; pMod = &g_Settings.modSwitchCreate;
            }
            else if (hEdit == GetDlgItem(g_hSettingsDlg, IDC_EDIT_SWITCH_NOTEPAD_KEY)) {
                pVk = &g_Settings.vkSwitchNotepad; pMod = &g_Settings.modSwitchNotepad;
            }
            if (pVk) {
                *pVk = g_capture.oldVk;
                *pMod = g_capture.oldMod;
                wchar_t buf[64];
                ShortcutToText(*pVk, *pMod, buf, 64);
                SetWindowTextW(hEdit, buf);
            }
            g_capture.bCapturing = FALSE;
            g_capture.hEdit = NULL;
        }
        return DefSubclassProc(hEdit, msg, wParam, lParam);
    }
    return DefSubclassProc(hEdit, msg, wParam, lParam);
}

// ---------- 设置对话框过程 ----------
static INT_PTR CALLBACK SettingsDlgProc(HWND hDlg, UINT msg, WPARAM wParam, LPARAM lParam) {
    wchar_t buf[64];

    switch (msg) {
    case WM_INITDIALOG: {
        SetWindowLongPtr(hDlg, GWL_EXSTYLE, GetWindowLongPtr(hDlg, GWL_EXSTYLE) | WS_EX_APPWINDOW);
        HICON hIcon = LoadIconW(g_hInst, MAKEINTRESOURCE(ICON_ID));
        SendMessage(hDlg, WM_SETICON, ICON_SMALL, (LPARAM)hIcon);
        SendMessage(hDlg, WM_SETICON, ICON_BIG, (LPARAM)hIcon);

        HWND hCombo = GetDlgItem(hDlg, IDC_COMBO_LANGUAGE);
        SendMessageW(hCombo, CB_ADDSTRING, 0, (LPARAM)L"简体中文");
        SendMessageW(hCombo, CB_ADDSTRING, 0, (LPARAM)L"繁體中文");
        SendMessageW(hCombo, CB_ADDSTRING, 0, (LPARAM)L"English");
        SendMessageW(hCombo, CB_SETCURSEL, g_Settings.language, 0);

        CheckDlgButton(hDlg, IDC_CHECK_ENABLE_CREATE, g_Settings.bEnableCreate ? BST_CHECKED : BST_UNCHECKED);
        CheckDlgButton(hDlg, IDC_CHECK_ENABLE_NOTEPAD, g_Settings.bEnableNotepad ? BST_CHECKED : BST_UNCHECKED);
        CheckDlgButton(hDlg, IDC_CHECK_AUTOSTART, g_Settings.bAutoStart ? BST_CHECKED : BST_UNCHECKED);

        ShortcutToText(g_Settings.vkCreate, g_Settings.modCreate, buf, 64);
        SetDlgItemTextW(hDlg, IDC_EDIT_CREATE_KEY, buf);
        ShortcutToText(g_Settings.vkNotepad, g_Settings.modNotepad, buf, 64);
        SetDlgItemTextW(hDlg, IDC_EDIT_NOTEPAD_KEY, buf);
        ShortcutToText(g_Settings.vkSwitchCreate, g_Settings.modSwitchCreate, buf, 64);
        SetDlgItemTextW(hDlg, IDC_EDIT_SWITCH_CREATE_KEY, buf);
        ShortcutToText(g_Settings.vkSwitchNotepad, g_Settings.modSwitchNotepad, buf, 64);
        SetDlgItemTextW(hDlg, IDC_EDIT_SWITCH_NOTEPAD_KEY, buf);

        SetWindowSubclass(GetDlgItem(hDlg, IDC_EDIT_CREATE_KEY), EditSubclassProc, 0, 0);
        SetWindowSubclass(GetDlgItem(hDlg, IDC_EDIT_NOTEPAD_KEY), EditSubclassProc, 0, 0);
        SetWindowSubclass(GetDlgItem(hDlg, IDC_EDIT_SWITCH_CREATE_KEY), EditSubclassProc, 0, 0);
        SetWindowSubclass(GetDlgItem(hDlg, IDC_EDIT_SWITCH_NOTEPAD_KEY), EditSubclassProc, 0, 0);

        g_bAboutMode = FALSE;
        ShowWindow(GetDlgItem(hDlg, IDC_EDIT_ABOUT), SW_HIDE);

        UpdateDialogTexts(hDlg);
        CenterWindow(hDlg);

        g_hSettingsDlg = hDlg;
        SetFocus(hDlg);
        return FALSE;
    }

    case WM_COMMAND: {
        switch (LOWORD(wParam)) {
        case IDC_BTN_ABOUT:
            ToggleAboutMode(hDlg);
            break;

        case IDC_COMBO_LANGUAGE: {
            if (HIWORD(wParam) == CBN_SELCHANGE) {
                int sel = (int)SendMessageW((HWND)lParam, CB_GETCURSEL, 0, 0);
                if (sel != CB_ERR) {
                    g_Settings.language = sel;
                    SaveSettings();
                    UpdateDialogTexts(hDlg);
                    if (g_bAboutMode) {
                        SetDlgItemTextW(hDlg, IDC_EDIT_ABOUT, GetString(sel, L"ABOUT_CONTENT"));
                    }
                }
            }
            break;
        }

        case IDC_CHECK_ENABLE_CREATE:
        case IDC_CHECK_ENABLE_NOTEPAD:
        case IDC_CHECK_AUTOSTART: {
            g_Settings.bEnableCreate = (IsDlgButtonChecked(hDlg, IDC_CHECK_ENABLE_CREATE) == BST_CHECKED);
            g_Settings.bEnableNotepad = (IsDlgButtonChecked(hDlg, IDC_CHECK_ENABLE_NOTEPAD) == BST_CHECKED);
            g_Settings.bAutoStart = (IsDlgButtonChecked(hDlg, IDC_CHECK_AUTOSTART) == BST_CHECKED);
            ApplyAutoStart(g_Settings.bAutoStart);
            SaveSettings();
            break;
        }

        case IDC_BTN_CLEAR_CREATE:
            g_Settings.vkCreate = 0; g_Settings.modCreate = 0;
            SetDlgItemTextW(hDlg, IDC_EDIT_CREATE_KEY, L"");
            SaveSettings();
            SetFocus(hDlg);
            break;
        case IDC_BTN_CLEAR_NOTEPAD:
            g_Settings.vkNotepad = 0; g_Settings.modNotepad = 0;
            SetDlgItemTextW(hDlg, IDC_EDIT_NOTEPAD_KEY, L"");
            SaveSettings();
            SetFocus(hDlg);
            break;
        case IDC_BTN_CLEAR_SWITCH_CREATE:
            g_Settings.vkSwitchCreate = 0; g_Settings.modSwitchCreate = 0;
            SetDlgItemTextW(hDlg, IDC_EDIT_SWITCH_CREATE_KEY, L"");
            SaveSettings();
            SetFocus(hDlg);
            break;
        case IDC_BTN_CLEAR_SWITCH_NOTEPAD:
            g_Settings.vkSwitchNotepad = 0; g_Settings.modSwitchNotepad = 0;
            SetDlgItemTextW(hDlg, IDC_EDIT_SWITCH_NOTEPAD_KEY, L"");
            SaveSettings();
            SetFocus(hDlg);
            break;

        case IDC_BTN_RESET_CREATE:
            g_Settings.vkCreate = DEF_VK_CREATE; g_Settings.modCreate = DEF_MOD_CREATE;
            ShortcutToText(g_Settings.vkCreate, g_Settings.modCreate, buf, 64);
            SetDlgItemTextW(hDlg, IDC_EDIT_CREATE_KEY, buf);
            SaveSettings();
            SetFocus(hDlg);
            break;
        case IDC_BTN_RESET_NOTEPAD:
            g_Settings.vkNotepad = DEF_VK_NOTEPAD; g_Settings.modNotepad = DEF_MOD_NOTEPAD;
            ShortcutToText(g_Settings.vkNotepad, g_Settings.modNotepad, buf, 64);
            SetDlgItemTextW(hDlg, IDC_EDIT_NOTEPAD_KEY, buf);
            SaveSettings();
            SetFocus(hDlg);
            break;
        case IDC_BTN_RESET_SWITCH_CREATE:
            g_Settings.vkSwitchCreate = DEF_VK_SWITCH_CREATE; g_Settings.modSwitchCreate = DEF_MOD_SWITCH_CREATE;
            SetDlgItemTextW(hDlg, IDC_EDIT_SWITCH_CREATE_KEY, L"");
            SaveSettings();
            SetFocus(hDlg);
            break;
        case IDC_BTN_RESET_SWITCH_NOTEPAD:
            g_Settings.vkSwitchNotepad = DEF_VK_SWITCH_NOTEPAD; g_Settings.modSwitchNotepad = DEF_MOD_SWITCH_NOTEPAD;
            SetDlgItemTextW(hDlg, IDC_EDIT_SWITCH_NOTEPAD_KEY, L"");
            SaveSettings();
            SetFocus(hDlg);
            break;

        case IDC_BTN_RESET_ALL: {
            g_Settings.vkCreate = DEF_VK_CREATE; g_Settings.modCreate = DEF_MOD_CREATE;
            g_Settings.vkNotepad = DEF_VK_NOTEPAD; g_Settings.modNotepad = DEF_MOD_NOTEPAD;
            g_Settings.vkSwitchCreate = DEF_VK_SWITCH_CREATE; g_Settings.modSwitchCreate = DEF_MOD_SWITCH_CREATE;
            g_Settings.vkSwitchNotepad = DEF_VK_SWITCH_NOTEPAD; g_Settings.modSwitchNotepad = DEF_MOD_SWITCH_NOTEPAD;
            ShortcutToText(g_Settings.vkCreate, g_Settings.modCreate, buf, 64);
            SetDlgItemTextW(hDlg, IDC_EDIT_CREATE_KEY, buf);
            ShortcutToText(g_Settings.vkNotepad, g_Settings.modNotepad, buf, 64);
            SetDlgItemTextW(hDlg, IDC_EDIT_NOTEPAD_KEY, buf);
            SetDlgItemTextW(hDlg, IDC_EDIT_SWITCH_CREATE_KEY, L"");
            SetDlgItemTextW(hDlg, IDC_EDIT_SWITCH_NOTEPAD_KEY, L"");
            SaveSettings();
            SetFocus(hDlg);
            break;
        }
        }
        break;
    }

    case WM_CLOSE:
        DestroyWindow(hDlg);
        break;

    case WM_DESTROY:
        g_hSettingsDlg = NULL;
        break;

    default:
        return FALSE;
    }
    return TRUE;
}

// ---------- 打开设置窗口 ----------
static void OpenSettingsDialog() {
    if (g_hSettingsDlg && IsWindow(g_hSettingsDlg)) {
        SetForegroundWindow(g_hSettingsDlg);
        return;
    }
    g_hSettingsDlg = CreateDialogW(g_hInst, MAKEINTRESOURCE(IDD_SETTINGS), g_hWnd, SettingsDlgProc);
    ShowWindow(g_hSettingsDlg, SW_SHOW);
}

// ---------- 获取桌面路径等辅助函数 ----------
static void GetDesktopPathW(wchar_t* buffer) {
    if (FAILED(SHGetFolderPathW(NULL, CSIDL_DESKTOP, NULL, 0, buffer)))
        wcscpy_s(buffer, MAX_PATH, L"C:\\");
}

static void UrlToLocalPathW(wchar_t* path) {
    const wchar_t* prefix = L"file:///";
    size_t plen = wcslen(prefix);
    if (wcsncmp(path, prefix, plen) == 0) {
        memmove(path, path + plen, (wcslen(path) - plen + 1) * sizeof(wchar_t));
        for (wchar_t* p = path; *p; ++p) {
            if (*p == L'/') *p = L'\\';
        }
        if (path[0] == L'\\') {
            memmove(path, path + 1, (wcslen(path) + 1) * sizeof(wchar_t));
        }
    }
}

static void GetCurrentFolderPathW(wchar_t* buffer) {
    HWND hwndFore = GetForegroundWindow();
    if (!hwndFore) {
        GetDesktopPathW(buffer);
        return;
    }

    wchar_t className[256];
    GetClassNameW(hwndFore, className, 256);

    if (wcscmp(className, L"Progman") == 0 || wcscmp(className, L"WorkerW") == 0) {
        GetDesktopPathW(buffer);
        return;
    }

    if (wcscmp(className, L"CabinetWClass") == 0 || wcscmp(className, L"ExploreWClass") == 0) {
        CoInitialize(NULL);
        IShellWindows* pShellWin = NULL;
        HRESULT hr = CoCreateInstance(CLSID_ShellWindows, NULL, CLSCTX_ALL, IID_IShellWindows, (void**)&pShellWin);
        if (SUCCEEDED(hr) && pShellWin) {
            long count = 0;
            pShellWin->get_Count(&count);
            for (long i = 0; i < count; ++i) {
                VARIANT varIndex;
                VariantInit(&varIndex);
                varIndex.vt = VT_I4;
                varIndex.lVal = i;

                IDispatch* pDisp = NULL;
                hr = pShellWin->Item(varIndex, &pDisp);
                VariantClear(&varIndex);
                if (FAILED(hr) || !pDisp) continue;

                IWebBrowser2* pBrowser = NULL;
                pDisp->QueryInterface(IID_IWebBrowser2, (void**)&pBrowser);
                pDisp->Release();
                if (!pBrowser) continue;

                HWND hwndBrowser = NULL;
                pBrowser->get_HWND((SHANDLE_PTR*)&hwndBrowser);
                if (hwndBrowser == hwndFore) {
                    BSTR bstrURL = NULL;
                    hr = pBrowser->get_LocationURL(&bstrURL);
                    if (SUCCEEDED(hr) && bstrURL) {
                        wcscpy_s(buffer, MAX_PATH, bstrURL);
                        SysFreeString(bstrURL);
                        UrlToLocalPathW(buffer);
                        if (PathFileExistsW(buffer)) {
                            pBrowser->Release();
                            pShellWin->Release();
                            CoUninitialize();
                            return;
                        }
                    }
                }
                pBrowser->Release();
            }
            pShellWin->Release();
        }
        CoUninitialize();
    }

    GetDesktopPathW(buffer);
}

static BOOL CreateTextFileW(const wchar_t* folder) {
    wchar_t fullPath[MAX_PATH];
    wchar_t numbered[64];
    int index = 0;

    swprintf_s(fullPath, MAX_PATH, L"%s\\新建文本文档.txt", folder);

    while (GetFileAttributesW(fullPath) != INVALID_FILE_ATTRIBUTES) {
        ++index;
        swprintf_s(numbered, 64, L"_%d.txt", index);
        swprintf_s(fullPath, MAX_PATH, L"%s\\新建文本文档%s", folder, numbered);
    }

    HANDLE hFile = CreateFileW(fullPath, GENERIC_WRITE, 0, NULL, CREATE_NEW, FILE_ATTRIBUTE_NORMAL, NULL);
    if (hFile == INVALID_HANDLE_VALUE) return FALSE;
    CloseHandle(hFile);
    return TRUE;
}

// ======================== 键盘钩子回调 ========================
LRESULT CALLBACK LowLevelKeyboardProc(int nCode, WPARAM wParam, LPARAM lParam) {
    if (nCode == HC_ACTION) {
        KBDLLHOOKSTRUCT* p = (KBDLLHOOKSTRUCT*)lParam;

        if (wParam == WM_KEYDOWN || wParam == WM_SYSKEYDOWN) {
            if (g_capture.bCapturing) {
                return CallNextHookEx(g_hKeyboardHook, nCode, wParam, lParam);
            }

            UINT vk = p->vkCode;
            if (vk == VK_CONTROL || vk == VK_SHIFT || vk == VK_MENU || vk == VK_LWIN || vk == VK_RWIN)
                return CallNextHookEx(g_hKeyboardHook, nCode, wParam, lParam);

            UINT mod = 0;
            if (GetAsyncKeyState(VK_CONTROL) & 0x8000) mod |= MOD_CONTROL;
            if (GetAsyncKeyState(VK_SHIFT) & 0x8000) mod |= MOD_SHIFT;
            if (GetAsyncKeyState(VK_MENU) & 0x8000) mod |= MOD_ALT;
            if (GetAsyncKeyState(VK_LWIN) & 0x8000 || GetAsyncKeyState(VK_RWIN) & 0x8000) mod |= MOD_WIN;

            BOOL handled = FALSE;

            if (g_Settings.bEnableCreate && g_Settings.vkCreate != 0) {
                if (vk == g_Settings.vkCreate && mod == g_Settings.modCreate) {
                    wchar_t folder[MAX_PATH];
                    GetCurrentFolderPathW(folder);
                    if (PathFileExistsW(folder)) {
                        PostMessage(g_hWnd, WM_CREATE_FILE, 0, 0);
                        handled = TRUE;
                    }
                }
            }
            if (!handled && g_Settings.bEnableNotepad && g_Settings.vkNotepad != 0) {
                if (vk == g_Settings.vkNotepad && mod == g_Settings.modNotepad) {
                    PostMessage(g_hWnd, WM_OPEN_NOTEPAD, 0, 0);
                    handled = TRUE;
                }
            }
            if (!handled && g_Settings.vkSwitchCreate != 0) {
                if (vk == g_Settings.vkSwitchCreate && mod == g_Settings.modSwitchCreate) {
                    g_Settings.bEnableCreate = !g_Settings.bEnableCreate;
                    SaveSettings();
                    if (g_hSettingsDlg && IsWindow(g_hSettingsDlg)) {
                        CheckDlgButton(g_hSettingsDlg, IDC_CHECK_ENABLE_CREATE, g_Settings.bEnableCreate ? BST_CHECKED : BST_UNCHECKED);
                    }
                    handled = TRUE;
                }
            }
            if (!handled && g_Settings.vkSwitchNotepad != 0) {
                if (vk == g_Settings.vkSwitchNotepad && mod == g_Settings.modSwitchNotepad) {
                    g_Settings.bEnableNotepad = !g_Settings.bEnableNotepad;
                    SaveSettings();
                    if (g_hSettingsDlg && IsWindow(g_hSettingsDlg)) {
                        CheckDlgButton(g_hSettingsDlg, IDC_CHECK_ENABLE_NOTEPAD, g_Settings.bEnableNotepad ? BST_CHECKED : BST_UNCHECKED);
                    }
                    handled = TRUE;
                }
            }

            if (handled) {
                return 1;
            }
        }
    }
    return CallNextHookEx(g_hKeyboardHook, nCode, wParam, lParam);
}

// ======================== 主窗口过程 ========================
LRESULT CALLBACK WndProc(HWND hWnd, UINT msg, WPARAM wParam, LPARAM lParam) {
    switch (msg) {
    case WM_CREATE: {
        NOTIFYICONDATAW nid = { sizeof(nid) };
        nid.hWnd = hWnd;
        nid.uID = 1;
        nid.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
        nid.uCallbackMessage = WM_TRAYICON;
        nid.hIcon = LoadIconW(g_hInst, MAKEINTRESOURCE(ICON_ID));
        wcscpy_s(nid.szTip, L"Text Document Creator");
        Shell_NotifyIconW(NIM_ADD, &nid);

        g_hKeyboardHook = SetWindowsHookExW(WH_KEYBOARD_LL, LowLevelKeyboardProc, g_hInst, 0);
        if (!g_hKeyboardHook) {
            MessageBoxW(hWnd, L"安装键盘钩子失败，请以管理员身份运行", L"错误", MB_ICONERROR);
        }

        LoadSettings();
        ApplyAutoStart(g_Settings.bAutoStart);
        break;
    }

    case WM_CREATE_FILE: {
        wchar_t folder[MAX_PATH];
        GetCurrentFolderPathW(folder);
        if (!CreateTextFileW(folder))
            MessageBoxW(hWnd, L"创建文件失败", L"错误", MB_ICONERROR);
        break;
    }

    case WM_OPEN_NOTEPAD: {
        ShellExecuteW(NULL, L"open", L"notepad.exe", NULL, NULL, SW_SHOWNORMAL);
        break;
    }

    case WM_TRAYICON: {
        if (lParam == WM_RBUTTONUP) {
            HMENU hMenu = CreatePopupMenu();
            AppendMenuW(hMenu, MF_STRING, ID_TRAY_SETTINGS, GetString(g_Settings.language, L"TRAY_SETTINGS"));
            AppendMenuW(hMenu, MF_SEPARATOR, 0, NULL);
            AppendMenuW(hMenu, MF_STRING, ID_TRAY_EXIT, GetString(g_Settings.language, L"TRAY_EXIT"));

            POINT pt;
            GetCursorPos(&pt);
            SetForegroundWindow(hWnd);
            int cmd = TrackPopupMenu(hMenu, TPM_RIGHTBUTTON | TPM_RETURNCMD, pt.x, pt.y, 0, hWnd, NULL);
            PostMessage(hWnd, WM_NULL, 0, 0);

            switch (cmd) {
            case ID_TRAY_SETTINGS:
                OpenSettingsDialog();
                break;
            case ID_TRAY_EXIT:
                DestroyWindow(hWnd);
                break;
            }
            DestroyMenu(hMenu);
        }
        break;
    }

    case WM_DESTROY: {
        if (g_hKeyboardHook) {
            UnhookWindowsHookEx(g_hKeyboardHook);
            g_hKeyboardHook = NULL;
        }

        NOTIFYICONDATAW nid = { sizeof(nid) };
        nid.hWnd = hWnd;
        nid.uID = 1;
        Shell_NotifyIconW(NIM_DELETE, &nid);

        PostQuitMessage(0);
        break;
    }

    default:
        return DefWindowProcW(hWnd, msg, wParam, lParam);
    }
    return 0;
}

BOOL IsElevated() {
    BOOL fRet = FALSE;
    HANDLE hToken = NULL;
    if (OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &hToken)) {
        TOKEN_ELEVATION Elevation;
        DWORD cbSize = sizeof(TOKEN_ELEVATION);
        if (GetTokenInformation(hToken, TokenElevation, &Elevation, cbSize, &cbSize))
            fRet = Elevation.TokenIsElevated;
        CloseHandle(hToken);
    }
    return fRet;
}

// ======================== 程序入口 ========================
int WINAPI WinMain(HINSTANCE hInstance, HINSTANCE hPrevInstance, LPSTR lpCmdLine, int nCmdShow) {
    g_hInst = hInstance;

    if (!IsElevated()) {
        wchar_t exePath[MAX_PATH];
        GetModuleFileNameW(NULL, exePath, MAX_PATH);
        SHELLEXECUTEINFOW sei = { sizeof(sei) };
        sei.lpVerb = L"runas";
        sei.lpFile = exePath;
        sei.lpParameters = GetCommandLineW();
        sei.nShow = SW_SHOWNORMAL;
        if (ShellExecuteExW(&sei))
            return 0;
        else {
            MessageBoxW(NULL, L"无法获取管理员权限，打开记事本快捷键可能无法正常工作。", L"提示", MB_ICONWARNING);
        }
    }

    WNDCLASSEXW wc = { sizeof(wc) };
    wc.lpfnWndProc = WndProc;
    wc.hInstance = hInstance;
    wc.lpszClassName = L"TextDocCreatorClass";
    wc.hIcon = LoadIconW(hInstance, MAKEINTRESOURCE(ICON_ID));
    if (!RegisterClassExW(&wc)) return 1;

    g_hWnd = CreateWindowExW(0, L"TextDocCreatorClass", L"Text Document Creator",
        WS_OVERLAPPEDWINDOW, 0, 0, 0, 0,
        NULL, NULL, hInstance, NULL);
    if (!g_hWnd) return 1;

    ShowWindow(g_hWnd, SW_HIDE);

    MSG msg;
    while (GetMessage(&msg, NULL, 0, 0)) {
        TranslateMessage(&msg);
        DispatchMessage(&msg);
    }

    return (int)msg.wParam;
}
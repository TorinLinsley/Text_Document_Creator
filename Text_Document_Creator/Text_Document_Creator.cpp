#define WIN32_LEAN_AND_MEAN
#include "resource.h"           // 确保定义 IDI_ICON1
#include <windows.h>
#include <shellapi.h>
#include <shlobj.h>
#include <shlwapi.h>
#include <commctrl.h>
#include <objbase.h>
#include <shlguid.h>
#include <exdisp.h>
#include <stdio.h>              // 提供 swprintf_s

#pragma comment(lib, "shell32.lib")
#pragma comment(lib, "advapi32.lib")
#pragma comment(lib, "ole32.lib")
#pragma comment(lib, "oleaut32.lib")
#pragma comment(lib, "shlwapi.lib")

// 图标资源 ID（resource.h 中定义 IDI_ICON1）
#define ICON_ID            IDI_ICON1

#define WM_TRAYICON        (WM_USER + 100)
#define ID_TRAY_EXIT       3001
#define ID_TRAY_AUTO       3002
#define ID_TRAY_ABOUT      3003

#define WM_CREATE_FILE     (WM_APP + 1)
#define WM_OPEN_NOTEPAD    (WM_APP + 2)

// 全局变量
HINSTANCE g_hInst;
HWND      g_hWnd;
BOOL      g_bAutoStart = FALSE;
HHOOK     g_hKeyboardHook = NULL;
volatile LONG g_winPressed = 0;

// ------------------------------------------------------------
// 获取桌面路径
static void GetDesktopPathW(wchar_t* buffer) {
    if (FAILED(SHGetFolderPathW(NULL, CSIDL_DESKTOP, NULL, 0, buffer)))
        wcscpy_s(buffer, MAX_PATH, L"C:\\");
}

// ------------------------------------------------------------
// 将 file:/// URL 转为本地路径（直接修改原字符串）
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

// ------------------------------------------------------------
// 获取当前聚焦的资源管理器或桌面路径
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

// ------------------------------------------------------------
// 创建文本文档（自动加下划线序号）
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

// ------------------------------------------------------------
// 注册表自启动
static void SetAutoStart(BOOL enable) {
    HKEY hKey;
    if (RegOpenKeyExW(HKEY_CURRENT_USER, L"Software\\Microsoft\\Windows\\CurrentVersion\\Run", 0, KEY_SET_VALUE, &hKey) != ERROR_SUCCESS)
        return;

    if (enable) {
        wchar_t exePath[MAX_PATH];
        GetModuleFileNameW(NULL, exePath, MAX_PATH);
        RegSetValueExW(hKey, L"Text_Document_Creator", 0, REG_SZ, (BYTE*)exePath, (DWORD)(wcslen(exePath) + 1) * sizeof(wchar_t));
    } else {
        RegDeleteValueW(hKey, L"Text_Document_Creator");
    }
    RegCloseKey(hKey);
}

static BOOL GetAutoStartState(void) {
    HKEY hKey;
    if (RegOpenKeyExW(HKEY_CURRENT_USER, L"Software\\Microsoft\\Windows\\CurrentVersion\\Run", 0, KEY_READ, &hKey) != ERROR_SUCCESS)
        return FALSE;
    wchar_t buffer[MAX_PATH];
    DWORD size = sizeof(buffer);
    DWORD type = 0;
    LONG ret = RegQueryValueExW(hKey, L"Text_Document_Creator", NULL, &type, (BYTE*)buffer, &size);
    RegCloseKey(hKey);
    return (ret == ERROR_SUCCESS);
}

// ------------------------------------------------------------
static void UpdateTrayMenu(HMENU hMenu) {
    CheckMenuItem(hMenu, ID_TRAY_AUTO, g_bAutoStart ? MF_CHECKED : MF_UNCHECKED);
}

// ======================== 键盘钩子回调 ========================
LRESULT CALLBACK LowLevelKeyboardProc(int nCode, WPARAM wParam, LPARAM lParam) {
    if (nCode == HC_ACTION) {
        KBDLLHOOKSTRUCT* p = (KBDLLHOOKSTRUCT*)lParam;

        if (p->vkCode == VK_LWIN || p->vkCode == VK_RWIN) {
            if (wParam == WM_KEYDOWN || wParam == WM_SYSKEYDOWN) {
                InterlockedExchange(&g_winPressed, 1);
            } else if (wParam == WM_KEYUP || wParam == WM_SYSKEYUP) {
                InterlockedExchange(&g_winPressed, 0);
            }
        }

        if (wParam == WM_KEYDOWN || wParam == WM_SYSKEYDOWN) {
            if (p->vkCode == 'F' && g_winPressed) {
                PostMessage(g_hWnd, WM_CREATE_FILE, 0, 0);
                return 1;
            }
            if (p->vkCode == 'J' && g_winPressed) {
                PostMessage(g_hWnd, WM_OPEN_NOTEPAD, 0, 0);
                return 1;
            }
        }
    }
    return CallNextHookEx(g_hKeyboardHook, nCode, wParam, lParam);
}

// ------------------------------------------------------------
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

        g_bAutoStart = GetAutoStartState();
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
            AppendMenuW(hMenu, MF_STRING, ID_TRAY_AUTO, L"开机自启动 | Auto Start");
            AppendMenuW(hMenu, MF_STRING, ID_TRAY_ABOUT, L"关于 | About");
            AppendMenuW(hMenu, MF_SEPARATOR, 0, NULL);
            AppendMenuW(hMenu, MF_STRING, ID_TRAY_EXIT, L"退出 | Exit");

            UpdateTrayMenu(hMenu);

            POINT pt;
            GetCursorPos(&pt);
            SetForegroundWindow(hWnd);
            int cmd = TrackPopupMenu(hMenu, TPM_RIGHTBUTTON | TPM_RETURNCMD, pt.x, pt.y, 0, hWnd, NULL);
            PostMessage(hWnd, WM_NULL, 0, 0);

            switch (cmd) {
            case ID_TRAY_AUTO:
                g_bAutoStart = !g_bAutoStart;
                SetAutoStart(g_bAutoStart);
                break;
            case ID_TRAY_ABOUT:
                MessageBoxW(hWnd,
                    L"【文本文档创建器】\n"
                    L"为创建 新建文本文档的操作绑定了快捷键，\n"
                    L"以及打开（系统自带）记事本的快捷键。\n"
                    L"\n"
                    L"[Win]+[F] - 创建文本文档\n"
                    L"[Win]+[J] - 打开记事本\n"
                    L"\n"
                    L"版权所有： Torin Linsley\n"
                    L"开源协议：LGPL v2.1\n"
                    L"\n"
                    L"-----------\n"
                    L"\n"
                    L"【Text Document Creator】\n"
                    L"I've assigned keyboard shortcuts for creating a new text file\n"
                    L"and for opening the built-in Notepad.\n"
                    L"\n"
                    L"[Win]+[F] - create new text document\n"
                    L"[Win]+[J] - open notepad\n"
                    L"\n"
                    L"Copyright @ Torin Linsley\n"
                    L"Open source license :  LGPL v2.1\n",
                    L"About | 关于",
                    MB_OK | MB_ICONINFORMATION);
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

// ------------------------------------------------------------
int WINAPI WinMain(HINSTANCE hInstance, HINSTANCE hPrevInstance, LPSTR lpCmdLine, int nCmdShow) {
    g_hInst = hInstance;

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
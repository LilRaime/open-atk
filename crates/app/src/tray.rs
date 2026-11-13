/*
 * System Tray Integration for ATK Control
 * Provides persistent background notification tray icon with live battery tooltip,
 * restore-on-click, and minimize-to-tray handling.
 */

#![allow(dead_code, non_snake_case, non_camel_case_types)]

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

static RESTORE_REQUESTED: AtomicBool = AtomicBool::new(false);
static EXIT_REQUESTED: AtomicBool = AtomicBool::new(false);
static EGUI_CTX: OnceLock<eframe::egui::Context> = OnceLock::new();

/* Registers the egui Context for immediate UI thread wakeups and viewport restores */
pub fn register_egui_ctx(ctx: eframe::egui::Context) {
    let _ = EGUI_CTX.set(ctx);
}

/* Checks and clears the flag indicating the user clicked the tray icon to restore the GUI */
pub fn should_restore_window() -> bool {
    RESTORE_REQUESTED.swap(false, Ordering::SeqCst)
}

/* Request restoring window visibility */
pub fn request_restore_window() {
    RESTORE_REQUESTED.store(true, Ordering::SeqCst);

    /* 1. Directly invoke Win32 ShowWindow and SetForegroundWindow from caller thread */
    #[cfg(windows)]
    unsafe {
        windows_tray::restore_main_window_win32();
    }

    /* 2. Dispatch viewport commands and request immediate repaint on egui context */
    if let Some(ctx) = EGUI_CTX.get() {
        ctx.send_viewport_cmd(eframe::egui::ViewportCommand::Visible(true));
        ctx.send_viewport_cmd(eframe::egui::ViewportCommand::Minimized(false));
        ctx.send_viewport_cmd(eframe::egui::ViewportCommand::Focus);
        ctx.request_repaint();
    }
}

/* Checks if user selected Exit from system tray menu */
pub fn should_exit() -> bool {
    EXIT_REQUESTED.load(Ordering::SeqCst)
}

/* Request application exit from system tray */
pub fn request_exit() {
    EXIT_REQUESTED.store(true, Ordering::SeqCst);
    remove_tray();
    /* Spawn a fallback exit thread to ensure process termination even if UI is suspended */
    std::thread::spawn(|| {
        std::thread::sleep(std::time::Duration::from_millis(150));
        std::process::exit(0);
    });
}

#[cfg(not(windows))]
pub fn init_tray() {}

#[cfg(not(windows))]
pub fn update_tray_tooltip(_tooltip: &str) {}

#[cfg(not(windows))]
pub fn remove_tray() {}

#[cfg(windows)]
pub fn enable_dark_mode_for_app() {
    unsafe {
        windows_tray::enable_dark_mode_for_tray(std::ptr::null_mut());
    }
}

#[cfg(not(windows))]
pub fn enable_dark_mode_for_app() {}

/* Minimizes RAM footprint by requesting the Windows OS memory manager to trim the process working set */
#[cfg(windows)]
pub fn trim_working_set() {
    unsafe {
        windows_tray::trim_process_memory();
    }
}

#[cfg(not(windows))]
pub fn trim_working_set() {}

#[cfg(windows)]
pub use windows_tray::*;

#[cfg(windows)]
mod windows_tray {
    use super::*;
    use std::sync::Mutex;

    const WM_APP: u32 = 0x8000;
    const WM_TRAY_CALLBACK: u32 = WM_APP + 101;
    const TRAY_ICON_ID: u32 = 1001;

    const NIM_ADD: u32 = 0x00000000;
    const NIM_MODIFY: u32 = 0x00000001;
    const NIM_DELETE: u32 = 0x00000002;

    const NIF_MESSAGE: u32 = 0x00000001;
    const NIF_ICON: u32 = 0x00000002;
    const NIF_TIP: u32 = 0x00000004;

    const WM_LBUTTONUP: usize = 0x0202;
    const WM_LBUTTONDBLCLK: usize = 0x0203;
    const WM_RBUTTONUP: usize = 0x0205;

    const ID_TRAY_OPEN: usize = 2001;
    const ID_TRAY_EXIT: usize = 2002;

    const MF_STRING: u32 = 0x00000000;
    const MF_SEPARATOR: u32 = 0x00000800;
    const TPM_RETURNCMD: u32 = 0x0100;
    const TPM_RIGHTBUTTON: u32 = 0x0002;

    const SM_CXSMICON: i32 = 49;
    const SM_CYSMICON: i32 = 50;

    const IMAGE_ICON: u32 = 1;
    const LR_DEFAULTSIZE: u32 = 0x0040;
    const IDI_APPLICATION: usize = 32512;

    const SW_HIDE: i32 = 0;
    const SW_SHOWNORMAL: i32 = 1;
    const SW_SHOW: i32 = 5;
    const SW_RESTORE: i32 = 9;

    #[repr(C)]
    struct POINT {
        x: i32,
        y: i32,
    }

    #[repr(C)]
    struct NOTIFYICONDATAW {
        cbSize: u32,
        hWnd: *mut std::ffi::c_void,
        uID: u32,
        uFlags: u32,
        uCallbackMessage: u32,
        hIcon: *mut std::ffi::c_void,
        szTip: [u16; 128],
        dwState: u32,
        dwStateMask: u32,
        szInfo: [u16; 256],
        uTimeoutOrVersion: u32,
        szInfoTitle: [u16; 64],
        dwInfoFlags: u32,
        guidItem: [u8; 16],
        hBalloonIcon: *mut std::ffi::c_void,
    }

    #[repr(C)]
    struct WNDCLASSW {
        style: u32,
        lpfnWndProc: Option<unsafe extern "system" fn(*mut std::ffi::c_void, u32, usize, isize) -> isize>,
        cbClsExtra: i32,
        cbWndExtra: i32,
        hInstance: *mut std::ffi::c_void,
        hIcon: *mut std::ffi::c_void,
        hCursor: *mut std::ffi::c_void,
        hbrBackground: *mut std::ffi::c_void,
        lpszMenuName: *const u16,
        lpszClassName: *const u16,
    }

    #[repr(C)]
    struct MSG {
        hwnd: *mut std::ffi::c_void,
        message: u32,
        wParam: usize,
        lParam: isize,
        time: u32,
        pt: [i32; 2],
    }

    extern "system" {
        fn Shell_NotifyIconW(dwMessage: u32, lpData: *const NOTIFYICONDATAW) -> i32;
        fn RegisterClassW(lpWndClass: *const WNDCLASSW) -> u16;
        fn CreateWindowExW(
            dwExStyle: u32,
            lpClassName: *const u16,
            lpWindowName: *const u16,
            dwStyle: u32,
            X: i32,
            Y: i32,
            nWidth: i32,
            nHeight: i32,
            hWndParent: *mut std::ffi::c_void,
            hMenu: *mut std::ffi::c_void,
            hInstance: *mut std::ffi::c_void,
            lpParam: *mut std::ffi::c_void,
        ) -> *mut std::ffi::c_void;
        fn DefWindowProcW(
            hWnd: *mut std::ffi::c_void,
            Msg: u32,
            wParam: usize,
            lParam: isize,
        ) -> isize;
        fn GetModuleHandleW(lpModuleName: *const u16) -> *mut std::ffi::c_void;
        fn LoadIconW(hInstance: *mut std::ffi::c_void, lpIconName: *const u16) -> *mut std::ffi::c_void;
        fn LoadImageW(
            hInst: *mut std::ffi::c_void,
            name: *const u16,
            type_: u32,
            cx: i32,
            cy: i32,
            fuLoad: u32,
        ) -> *mut std::ffi::c_void;
        fn GetSystemMetrics(nIndex: i32) -> i32;
        fn CreateIconFromResourceEx(
            pbIconBits: *const u8,
            cbIconBits: u32,
            fIcon: i32,
            dwVersion: u32,
            cxDesired: i32,
            cyDesired: i32,
            uFlags: u32,
        ) -> *mut std::ffi::c_void;
        fn CreatePopupMenu() -> *mut std::ffi::c_void;
        fn AppendMenuW(
            hMenu: *mut std::ffi::c_void,
            uFlags: u32,
            uIDNewItem: usize,
            lpNewItem: *const u16,
        ) -> i32;
        fn TrackPopupMenu(
            hMenu: *mut std::ffi::c_void,
            uFlags: u32,
            x: i32,
            y: i32,
            nReserved: i32,
            hWnd: *mut std::ffi::c_void,
            prcRect: *const std::ffi::c_void,
        ) -> i32;
        fn DestroyMenu(hMenu: *mut std::ffi::c_void) -> i32;
        fn GetCursorPos(lpPoint: *mut POINT) -> i32;
        fn SetForegroundWindow(hWnd: *mut std::ffi::c_void) -> i32;
        fn PostMessageW(
            hWnd: *mut std::ffi::c_void,
            Msg: u32,
            wParam: usize,
            lParam: isize,
        ) -> i32;
        fn GetMessageW(
            lpMsg: *mut std::ffi::c_void,
            hWnd: *mut std::ffi::c_void,
            wMsgFilterMin: u32,
            wMsgFilterMax: u32,
        ) -> i32;
        fn TranslateMessage(lpMsg: *const std::ffi::c_void) -> i32;
        fn DispatchMessageW(lpMsg: *const std::ffi::c_void) -> isize;
        fn DestroyWindow(hWnd: *mut std::ffi::c_void) -> i32;
        fn LoadLibraryW(lpLibFileName: *const u16) -> *mut std::ffi::c_void;
        fn GetProcAddress(hModule: *mut std::ffi::c_void, lpProcName: *const u8) -> *mut std::ffi::c_void;
        fn ShowWindow(hWnd: *mut std::ffi::c_void, nCmdShow: i32) -> i32;
        fn BringWindowToTop(hWnd: *mut std::ffi::c_void) -> i32;
        fn SetFocus(hWnd: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
        fn FindWindowW(lpClassName: *const u16, lpWindowName: *const u16) -> *mut std::ffi::c_void;
        fn GetCurrentProcessId() -> u32;
        fn GetWindowThreadProcessId(hWnd: *mut std::ffi::c_void, lpdwProcessId: *mut u32) -> u32;
        fn EnumWindows(
            lpEnumFunc: unsafe extern "system" fn(*mut std::ffi::c_void, isize) -> i32,
            lParam: isize,
        ) -> i32;
        fn GetCurrentProcess() -> *mut std::ffi::c_void;
        fn SetProcessWorkingSetSize(
            hProcess: *mut std::ffi::c_void,
            dwMinimumWorkingSetSize: usize,
            dwMaximumWorkingSetSize: usize,
        ) -> i32;
    }

    /* Flushes inactive pages from process working set to free up RAM while idle/minimized */
    pub unsafe fn trim_process_memory() {
        SetProcessWorkingSetSize(GetCurrentProcess(), usize::MAX, usize::MAX);
    }

    static TRAY_HWND: OnceLock<Mutex<Option<usize>>> = OnceLock::new();

    fn get_tray_hwnd() -> &'static Mutex<Option<usize>> {
        TRAY_HWND.get_or_init(|| Mutex::new(None))
    }

    /* Callback for EnumWindows to locate top-level window belonging to current process */
    unsafe extern "system" fn enum_windows_find_main(hwnd: *mut std::ffi::c_void, lparam: isize) -> i32 {
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, &mut pid);
        if pid == GetCurrentProcessId() {
            let tray_h = get_tray_hwnd().lock().ok().and_then(|g| *g).unwrap_or(0);
            if (hwnd as usize) != tray_h {
                *(lparam as *mut usize) = hwnd as usize;
                return 0; /* Stop enumeration once found */
            }
        }
        1 /* Continue enumeration */
    }

    /* Locate main OpenATK window handle */
    pub unsafe fn find_main_window_hwnd() -> Option<*mut std::ffi::c_void> {
        let current_pid = GetCurrentProcessId();

        /* 1. Try finding by title "OpenATK" */
        let title: Vec<u16> = "OpenATK\0".encode_utf16().collect();
        let hwnd = FindWindowW(std::ptr::null(), title.as_ptr());
        if !hwnd.is_null() {
            let mut pid = 0u32;
            GetWindowThreadProcessId(hwnd, &mut pid);
            if pid == current_pid {
                return Some(hwnd);
            }
        }

        /* 2. Fallback: Enumerate all top-level windows of current process */
        let mut target_hwnd: usize = 0;
        EnumWindows(enum_windows_find_main, &mut target_hwnd as *mut _ as isize);

        if target_hwnd != 0 {
            Some(target_hwnd as *mut std::ffi::c_void)
        } else {
            None
        }
    }

    /* Restore and focus main OpenATK GUI window using native Win32 calls */
    pub unsafe fn restore_main_window_win32() {
        if let Some(hwnd) = find_main_window_hwnd() {
            ShowWindow(hwnd, SW_RESTORE);
            ShowWindow(hwnd, SW_SHOW);
            BringWindowToTop(hwnd);
            SetForegroundWindow(hwnd);
            SetFocus(hwnd);
        }
    }

    /* Resolves the best available ATK/VXE application icon for the system tray */
    unsafe fn get_app_tray_icon(hinstance: *mut std::ffi::c_void) -> *mut std::ffi::c_void {
        let cx = GetSystemMetrics(SM_CXSMICON);
        let cy = GetSystemMetrics(SM_CYSMICON);
        let cx = if cx <= 0 { 16 } else { cx };
        let cy = if cy <= 0 { 16 } else { cy };

        /* 1. Try loading icon resource #1 from the executable module */
        let h_icon = LoadImageW(
            hinstance,
            1 as *const u16,
            IMAGE_ICON,
            cx,
            cy,
            0,
        );
        if !h_icon.is_null() {
            return h_icon;
        }

        let h_icon = LoadIconW(hinstance, 1 as *const u16);
        if !h_icon.is_null() {
            return h_icon;
        }

        /* 2. Fallback: Parse embedded ICO bytes directly */
        const ICO_BYTES: &[u8] = include_bytes!("../assets/atk_control.ico");
        if ICO_BYTES.len() >= 22 && ICO_BYTES[2] == 1 && ICO_BYTES[3] == 0 {
            let count = u16::from_le_bytes([ICO_BYTES[4], ICO_BYTES[5]]) as usize;
            let mut best_offset = 0;
            let mut best_size = 0;
            let mut best_diff = i32::MAX;

            for i in 0..count {
                let entry_off = 6 + i * 16;
                if entry_off + 16 <= ICO_BYTES.len() {
                    let w = ICO_BYTES[entry_off] as i32;
                    let w = if w == 0 { 256 } else { w };
                    let diff = (w - cx).abs();
                    let size = u32::from_le_bytes([
                        ICO_BYTES[entry_off + 8],
                        ICO_BYTES[entry_off + 9],
                        ICO_BYTES[entry_off + 10],
                        ICO_BYTES[entry_off + 11],
                    ]) as usize;
                    let off = u32::from_le_bytes([
                        ICO_BYTES[entry_off + 12],
                        ICO_BYTES[entry_off + 13],
                        ICO_BYTES[entry_off + 14],
                        ICO_BYTES[entry_off + 15],
                    ]) as usize;

                    if diff < best_diff && off + size <= ICO_BYTES.len() {
                        best_diff = diff;
                        best_offset = off;
                        best_size = size;
                    }
                }
            }

            if best_size > 0 {
                let icon = CreateIconFromResourceEx(
                    ICO_BYTES[best_offset..].as_ptr(),
                    best_size as u32,
                    1, /* 1 = icon */
                    0x00030000,
                    cx,
                    cy,
                    0,
                );
                if !icon.is_null() {
                    return icon;
                }
            }
        }

        /* 3. System application icon as last resort */
        LoadIconW(std::ptr::null_mut(), IDI_APPLICATION as *const u16)
    }

    /* Enables native Windows 10/11 Dark Mode for Win32 menus and windows */
    pub unsafe fn enable_dark_mode_for_tray(hwnd: *mut std::ffi::c_void) {
        let uxtheme_str: Vec<u16> = "uxtheme.dll\0".encode_utf16().collect();
        let h_uxtheme = LoadLibraryW(uxtheme_str.as_ptr());
        if !h_uxtheme.is_null() {
            /* Ordinal 135: SetPreferredAppMode (1903+) or AllowDarkModeForApp (1809) */
            /* 2 = ForceDark (Forces dark popup menus and controls) */
            let set_preferred_app_mode: Option<unsafe extern "system" fn(i32) -> i32> =
                std::mem::transmute(GetProcAddress(h_uxtheme, 135 as *const u8));
            if let Some(set_mode) = set_preferred_app_mode {
                set_mode(2);
            }

            /* Ordinal 136: FlushMenuThemes (forces refresh of cached menu themes) */
            let flush_menu_themes: Option<unsafe extern "system" fn()> =
                std::mem::transmute(GetProcAddress(h_uxtheme, 136 as *const u8));
            if let Some(flush) = flush_menu_themes {
                flush();
            }

            if !hwnd.is_null() {
                /* Ordinal 133: AllowDarkModeForWindow */
                let allow_dark_mode_for_window: Option<unsafe extern "system" fn(*mut std::ffi::c_void, bool) -> bool> =
                    std::mem::transmute(GetProcAddress(h_uxtheme, 133 as *const u8));
                if let Some(allow_window) = allow_dark_mode_for_window {
                    allow_window(hwnd, true);
                }

                /* Apply DarkMode_Explorer theme to the window */
                let theme_name: Vec<u16> = "DarkMode_Explorer\0".encode_utf16().collect();
                let set_window_theme: Option<unsafe extern "system" fn(*mut std::ffi::c_void, *const u16, *const u16) -> i32> =
                    std::mem::transmute(GetProcAddress(h_uxtheme, b"SetWindowTheme\0".as_ptr()));
                if let Some(set_theme) = set_window_theme {
                    let _ = set_theme(hwnd, theme_name.as_ptr(), std::ptr::null());
                }
            }
        }

        if !hwnd.is_null() {
            /* Enable DWM Dark Mode attribute on Windows 10/11 */
            let dwmapi_str: Vec<u16> = "dwmapi.dll\0".encode_utf16().collect();
            let h_dwmapi = LoadLibraryW(dwmapi_str.as_ptr());
            if !h_dwmapi.is_null() {
                let dwm_set_window_attribute: Option<
                    unsafe extern "system" fn(*mut std::ffi::c_void, u32, *const i32, u32) -> i32,
                > = std::mem::transmute(GetProcAddress(h_dwmapi, b"DwmSetWindowAttribute\0".as_ptr()));
                if let Some(set_attr) = dwm_set_window_attribute {
                    let dark_true: i32 = 1;
                    /* 20 = DWMWA_USE_IMMERSIVE_DARK_MODE (Windows 11 / 10 20H1+) */
                    let _ = set_attr(hwnd, 20, &dark_true, 4);
                    /* 19 = DWMWA_USE_IMMERSIVE_DARK_MODE (Windows 10 1809-1909) */
                    let _ = set_attr(hwnd, 19, &dark_true, 4);
                }
            }
        }
    }

    unsafe extern "system" fn tray_wnd_proc(
        hWnd: *mut std::ffi::c_void,
        Msg: u32,
        wParam: usize,
        lParam: isize,
    ) -> isize {
        if Msg == WM_TRAY_CALLBACK {
            let event = lParam as usize;
            if event == WM_LBUTTONUP || event == WM_LBUTTONDBLCLK {
                request_restore_window();
                return 0;
            }
            if event == WM_RBUTTONUP {
                enable_dark_mode_for_tray(hWnd);
                let mut pt = POINT { x: 0, y: 0 };
                GetCursorPos(&mut pt);
                let hmenu = CreatePopupMenu();
                if !hmenu.is_null() {
                    let open_text: Vec<u16> = "Open OpenATK\0".encode_utf16().collect();
                    let exit_text: Vec<u16> = "Exit\0".encode_utf16().collect();

                    AppendMenuW(hmenu, MF_STRING, ID_TRAY_OPEN, open_text.as_ptr());
                    AppendMenuW(hmenu, MF_SEPARATOR, 0, std::ptr::null());
                    AppendMenuW(hmenu, MF_STRING, ID_TRAY_EXIT, exit_text.as_ptr());

                    /* SetForegroundWindow ensures the menu dismisses on outside clicks */
                    SetForegroundWindow(hWnd);
                    let cmd = TrackPopupMenu(
                        hmenu,
                        TPM_RETURNCMD | TPM_RIGHTBUTTON,
                        pt.x,
                        pt.y,
                        0,
                        hWnd,
                        std::ptr::null(),
                    );
                    PostMessageW(hWnd, 0, 0, 0);
                    DestroyMenu(hmenu);

                    if cmd as usize == ID_TRAY_OPEN {
                        request_restore_window();
                    } else if cmd as usize == ID_TRAY_EXIT {
                        request_exit();
                    }
                }
                return 0;
            }
            return 0;
        }
        DefWindowProcW(hWnd, Msg, wParam, lParam)
    }

    /* Initialize background Windows system tray icon and message loop */
    pub fn init_tray() {
        std::thread::spawn(|| unsafe {
            let hinstance = GetModuleHandleW(std::ptr::null());
            let class_name: Vec<u16> = "AtkControlTrayClass\0".encode_utf16().collect();
            let window_name: Vec<u16> = "AtkControlTrayWindow\0".encode_utf16().collect();

            let wnd_class = WNDCLASSW {
                style: 0,
                lpfnWndProc: Some(tray_wnd_proc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: hinstance,
                hIcon: std::ptr::null_mut(),
                hCursor: std::ptr::null_mut(),
                hbrBackground: std::ptr::null_mut(),
                lpszMenuName: std::ptr::null(),
                lpszClassName: class_name.as_ptr(),
            };

            let _ = RegisterClassW(&wnd_class);

            let hwnd = CreateWindowExW(
                0,
                class_name.as_ptr(),
                window_name.as_ptr(),
                0,
                0,
                0,
                0,
                0,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                hinstance,
                std::ptr::null_mut(),
            );

            if hwnd.is_null() {
                return;
            }

            enable_dark_mode_for_tray(hwnd);

            if let Ok(mut lock) = get_tray_hwnd().lock() {
                *lock = Some(hwnd as usize);
            }

            let h_icon = get_app_tray_icon(hinstance);

            let mut nid: NOTIFYICONDATAW = std::mem::zeroed();
            nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
            nid.hWnd = hwnd;
            nid.uID = TRAY_ICON_ID;
            nid.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
            nid.uCallbackMessage = WM_TRAY_CALLBACK;
            nid.hIcon = h_icon;

            let tip: Vec<u16> = "OpenATK\0".encode_utf16().collect();
            for (i, c) in tip.iter().take(127).enumerate() {
                nid.szTip[i] = *c;
            }

            Shell_NotifyIconW(NIM_ADD, &nid);

            let mut msg = std::mem::zeroed();
            while GetMessageW(&mut msg as *mut _ as *mut std::ffi::c_void, hwnd, 0, 0) > 0 {
                TranslateMessage(&msg as *const _ as *const std::ffi::c_void);
                DispatchMessageW(&msg as *const _ as *const std::ffi::c_void);
            }

            Shell_NotifyIconW(NIM_DELETE, &nid);
            DestroyWindow(hwnd);
        });
    }

    /* Update tray icon tooltip string (e.g. live battery status) only when text changes */
    pub fn update_tray_tooltip(tooltip: &str) {
        static LAST_TOOLTIP: Mutex<String> = Mutex::new(String::new());
        if let Ok(mut last) = LAST_TOOLTIP.lock() {
            if *last == tooltip {
                return;
            }
            *last = tooltip.to_string();
        }

        let hwnd_opt = get_tray_hwnd().lock().ok().and_then(|g| *g);
        if let Some(hwnd_val) = hwnd_opt {
            unsafe {
                let mut nid: NOTIFYICONDATAW = std::mem::zeroed();
                nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
                nid.hWnd = hwnd_val as *mut std::ffi::c_void;
                nid.uID = TRAY_ICON_ID;
                nid.uFlags = NIF_TIP;

                let mut encoded: Vec<u16> = tooltip.encode_utf16().collect();
                encoded.push(0);
                for (i, c) in encoded.iter().take(127).enumerate() {
                    nid.szTip[i] = *c;
                }

                Shell_NotifyIconW(NIM_MODIFY, &nid);
            }
        }
    }

    /* Cleanly remove tray icon */
    pub fn remove_tray() {
        let hwnd_opt = get_tray_hwnd().lock().ok().and_then(|g| *g);
        if let Some(hwnd_val) = hwnd_opt {
            unsafe {
                let mut nid: NOTIFYICONDATAW = std::mem::zeroed();
                nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
                nid.hWnd = hwnd_val as *mut std::ffi::c_void;
                nid.uID = TRAY_ICON_ID;
                Shell_NotifyIconW(NIM_DELETE, &nid);
            }
        }
    }
}

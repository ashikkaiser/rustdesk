use super::{PrivacyMode, INVALID_PRIVACY_MODE_CONN_ID};
use crate::privacy_mode::PrivacyModeState;

pub const PRIVACY_MODE_IMPL: &str = crate::privacy_mode::PRIVACY_MODE_IMPL_WIN_DIRECT_OVERLAY;

use hbb_common::{allow_err, bail, log, ResultType};
use hbb_common::platform::windows::is_windows_version_or_greater;
use enigo::ENIGO_INPUT_EXTRA_VALUE;
use std::ffi::{c_void, OsStr};
use std::mem;
use std::os::windows::ffi::OsStrExt;
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use winapi::{
    ctypes::c_void as WinCvoid,
    shared::{
    minwindef::{ATOM, BOOL, DWORD, HINSTANCE, LPARAM, LRESULT, UINT, WPARAM, LOBYTE},
        windef::{HBITMAP, HCURSOR, HHOOK, HMENU, HWND, POINT, RECT},
    },
    um::{
        dwmapi::DwmSetWindowAttribute,
        errhandlingapi::GetLastError,
        libloaderapi::{FreeLibrary, GetModuleHandleW, GetProcAddress, LoadLibraryW},
        processthreadsapi::GetCurrentThreadId,
        wingdi::{CreateBitmap, CreateSolidBrush, DeleteObject, RGB},
        winuser::{
            BeginPaint, CallNextHookEx, ClientToScreen, CreateIconIndirect, CreateWindowExW, DefWindowProcW, DestroyWindow,
            DispatchMessageW, EndPaint, FillRect, GetClientRect, GetCursorPos, GetForegroundWindow,
            GetMessageW, GetSystemMetrics, GetWindowLongPtrW, GetWindowLongW, PostMessageW, PostQuitMessage, 
            PostThreadMessageW, RegisterClassExW, ScreenToClient, SetCursor, SetCursorPos, 
            SetLayeredWindowAttributes, SetSystemCursor, SetWindowDisplayAffinity, SetWindowLongPtrW, SetWindowLongW, SetWindowPos, 
            SetWindowsHookExW, ShowCursor, ShowWindow, TranslateMessage, UnhookWindowsHookEx, 
            UpdateWindow, WindowFromPoint, CS_HREDRAW, CS_VREDRAW, GWL_EXSTYLE, HC_ACTION, 
            HWND_TOPMOST, IDC_ARROW, LWA_ALPHA, MSG, PAINTSTRUCT, SM_CXVIRTUALSCREEN, 
            SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN, SW_HIDE, SW_SHOW, 
            SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_SHOWWINDOW, WM_APP, WM_CHAR, WM_CLOSE, 
            WM_DESTROY, WM_KEYDOWN, WM_KEYUP, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN, 
            WM_MBUTTONUP, WM_MOUSEHWHEEL, WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_PAINT, WM_RBUTTONDOWN, 
            WM_RBUTTONUP, WM_SETCURSOR, WM_SYSKEYDOWN, WM_SYSKEYUP, WM_WINDOWPOSCHANGING, 
            WNDCLASSEXW, WINDOWPOS, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, 
            WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_POPUP, WH_KEYBOARD_LL, WH_MOUSE_LL, 
            KBDLLHOOKSTRUCT, MSLLHOOKSTRUCT, GetKeyState, ICONINFO, SPI_SETCURSORS, SystemParametersInfoW,
            VK_CONTROL, WM_NCHITTEST, HTTRANSPARENT,
        },
    },
};

pub(super) const PRIVACY_MODE_IMPL_LOCAL: &str = "privacy_mode_impl_direct";
pub(super) const PRIVACY_WINDOW_CLASS: &str = "CloudyDeskDirectPrivacyWindow";
pub(super) const PRIVACY_WINDOW_TITLE: &str = "CloudyDesk Privacy Overlay";

// Windows Display Affinity constants (not in winapi crate)
const WDA_NONE: u32 = 0x0000_0000;
const WDA_EXCLUDEFROMCAPTURE: u32 = 0x0000_0011;
const ZBID_ABOVELOCK_UX: u32 = 18;
const DWMWA_CLOAK: u32 = 13;

// Define OCR_* constants (IDs for system cursors) as they may not be exposed in our winapi version
const OCR_NORMAL: u32 = 32512;      // Arrow
const OCR_IBEAM: u32 = 32513;       // Text I-beam
const OCR_WAIT: u32 = 32514;        // Hourglass
const OCR_CROSS: u32 = 32515;       // Crosshair
const OCR_UP: u32 = 32516;          // Up arrow
const OCR_SIZE: u32 = 32640;        // Obsolete size (unused)
const OCR_ICON: u32 = 32641;        // Obsolete icon (unused)
const OCR_SIZENWSE: u32 = 32642;    // NW/SE resize
const OCR_SIZENESW: u32 = 32643;    // NE/SW resize
const OCR_SIZEWE: u32 = 32644;      // W/E resize
const OCR_SIZENS: u32 = 32645;      // N/S resize
const OCR_SIZEALL: u32 = 32646;     // Move
const OCR_NO: u32 = 32648;          // No/Unavailable
const OCR_HAND: u32 = 32649;        // Hand (link)
const OCR_APPSTARTING: u32 = 32650; // App starting (unused)
const OCR_HELP: u32 = 32651;        // Help

const WM_PRIVACY_SHOW: UINT = WM_APP + 0x101;
const WM_PRIVACY_HIDE: UINT = WM_APP + 0x102;
const WM_PRIVACY_SHUTDOWN: UINT = WM_APP + 0x103;

static PRIVACY_ACTIVE: AtomicBool = AtomicBool::new(false);
static CURSOR_HIDDEN: AtomicBool = AtomicBool::new(false);
static CURSOR_ENFORCER_RUNNING: AtomicBool = AtomicBool::new(false);
static HOOKS_INSTALLED: AtomicBool = AtomicBool::new(false);
static CURSOR_SYSTEM_REPLACED: AtomicBool = AtomicBool::new(false);
static OVERLAY_CONTROLLER: Mutex<Option<Arc<OverlayController>>> = Mutex::new(None);

static mut KEYBOARD_HOOK: HHOOK = ptr::null_mut();
static mut MOUSE_HOOK: HHOOK = ptr::null_mut();

#[derive(Clone, Copy)]
struct OverlayThreadState {
    thread_id: u32,
}

struct OverlayController {
    state: Mutex<Option<OverlayThreadState>>,
    join: Mutex<Option<JoinHandle<()>>>,
}

impl OverlayController {
    fn new() -> Self {
        Self {
            state: Mutex::new(None),
            join: Mutex::new(None),
        }
    }

    fn set_join_handle(&self, handle: JoinHandle<()>) {
        *self.join.lock().unwrap() = Some(handle);
    }

    fn set_state(&self, state: OverlayThreadState) {
        *self.state.lock().unwrap() = Some(state);
    }

    fn clear_state(&self) {
        *self.state.lock().unwrap() = None;
    }

    fn thread_id(&self) -> Option<u32> {
        self.state.lock().unwrap().as_ref().map(|s| s.thread_id)
    }

    fn post_message(&self, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> ResultType<()> {
        if let Some(thread_id) = self.thread_id() {
            let result = unsafe { PostThreadMessageW(thread_id, msg, wparam, lparam) };
            if result == 0 {
                let error = unsafe { GetLastError() };
                bail!("Failed to post message {}, error {}", msg, error);
            }
            Ok(())
        } else {
            bail!("Overlay thread is not ready");
        }
    }

    fn join_thread(&self) {
        if let Some(handle) = self.join.lock().unwrap().take() {
            if let Err(err) = handle.join() {
                log::warn!("Overlay thread join failed: {:?}", err);
            }
        }
    }
}

fn to_wide_null(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
}

fn start_overlay_thread() -> ResultType<Arc<OverlayController>> {
    if let Some(existing) = OVERLAY_CONTROLLER.lock().unwrap().as_ref() {
        return Ok(existing.clone());
    }

    let controller = Arc::new(OverlayController::new());
    let thread_controller = controller.clone();
    let (ready_tx, ready_rx) = mpsc::channel();

    let handle = match thread::Builder::new()
        .name("privacy-overlay".into())
        .spawn(move || overlay_thread_main(thread_controller, ready_tx))
    {
        Ok(handle) => handle,
        Err(e) => bail!("Failed to spawn overlay thread: {}", e),
    };

    controller.set_join_handle(handle);

    match ready_rx.recv() {
        Ok(Ok(state)) => {
            controller.set_state(state);
            let mut guard = OVERLAY_CONTROLLER.lock().unwrap();
            *guard = Some(controller.clone());
            Ok(controller)
        }
        Ok(Err(msg)) => {
            controller.join_thread();
            bail!("Overlay thread initialization failed: {}", msg);
        }
        Err(_) => {
            controller.join_thread();
            bail!("Overlay thread initialization channel closed unexpectedly");
        }
    }
}

fn overlay_thread_main(
    controller: Arc<OverlayController>,
    ready_tx: mpsc::Sender<Result<OverlayThreadState, String>>,
) {
    unsafe {
        let thread_id = GetCurrentThreadId();
        let class_name = to_wide_null(PRIVACY_WINDOW_CLASS);
        let window_title = to_wide_null(PRIVACY_WINDOW_TITLE);
        let hinstance = GetModuleHandleW(ptr::null_mut());

        let brush = CreateSolidBrush(RGB(0, 0, 0));

        let wnd_class = WNDCLASSEXW {
            cbSize: mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(window_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinstance,
            hIcon: ptr::null_mut(),
            hCursor: winapi::um::winuser::LoadCursorW(ptr::null_mut(), IDC_ARROW),
            hbrBackground: brush,
            lpszMenuName: ptr::null(),
            lpszClassName: class_name.as_ptr(),
            hIconSm: ptr::null_mut(),
        };

        let class_atom = RegisterClassExW(&wnd_class);
        if class_atom == 0 {
            let error = GetLastError();
            if error != 1410 {
                let _ = ready_tx.send(Err(format!(
                    "Failed to register overlay window class, error {}",
                    error
                )));
                return;
            }
        }

        let hwnd_result = create_overlay_window(
            class_atom as ATOM,
            class_name.as_ptr(),
            window_title.as_ptr(),
            hinstance,
        );

        let hwnd = match hwnd_result {
            Ok(hwnd) => hwnd,
            Err(err) => {
                let _ = ready_tx.send(Err(err));
                return;
            }
        };

        configure_overlay_window(hwnd);
        cloak_window(hwnd, true);

        if ready_tx
            .send(Ok(OverlayThreadState { thread_id }))
            .is_err()
        {
            DestroyWindow(hwnd);
            return;
        }

        let mut msg: MSG = mem::zeroed();
        loop {
            let result = GetMessageW(&mut msg, ptr::null_mut(), 0, 0);
            if result == 0 {
                break;
            } else if result == -1 {
                let error = GetLastError();
                log::error!("GetMessageW failed: {}", error);
                break;
            }

            match msg.message {
                WM_PRIVACY_SHOW => {
                    show_overlay(hwnd);
                }
                WM_PRIVACY_HIDE => {
                    hide_overlay(hwnd);
                }
                WM_PRIVACY_SHUTDOWN => {
                    hide_overlay(hwnd);
                    cloak_window(hwnd, false);
                    set_capture_exclusion(hwnd, false);
                    DestroyWindow(hwnd);
                }
                _ => {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }
        }

        hide_overlay(hwnd);
        set_capture_exclusion(hwnd, false);
        controller.clear_state();
        remove_input_hooks();
        show_cursor_restore();
        PRIVACY_ACTIVE.store(false, Ordering::SeqCst);
        CURSOR_ENFORCER_RUNNING.store(false, Ordering::SeqCst);
        CURSOR_HIDDEN.store(false, Ordering::SeqCst);
        HOOKS_INSTALLED.store(false, Ordering::SeqCst);
    }
}

unsafe fn create_overlay_window(
    class_atom: ATOM,
    class_name: *const u16,
    window_title: *const u16,
    hinstance: HINSTANCE,
) -> Result<HWND, String> {
    if let Some(hwnd) = try_create_window_in_band(class_atom, window_title, hinstance) {
        if hwnd.is_null() {
            return Err("CreateWindowInBand returned null".into());
        }
        return Ok(hwnd);
    }

    // Get screen dimensions for initial window creation
    let vx = GetSystemMetrics(SM_XVIRTUALSCREEN);
    let vy = GetSystemMetrics(SM_YVIRTUALSCREEN);
    let vw = GetSystemMetrics(SM_CXVIRTUALSCREEN);
    let vh = GetSystemMetrics(SM_CYVIRTUALSCREEN);
    
    let hwnd = CreateWindowExW(
        WS_EX_TOPMOST | WS_EX_NOACTIVATE | WS_EX_TRANSPARENT,
        class_name,
        window_title,
        WS_POPUP,
        vx,
        vy,
        vw,
        vh,
        ptr::null_mut(),
        ptr::null_mut(),
        hinstance,
        ptr::null_mut(),
    );

    if hwnd.is_null() {
        return Err(format!(
            "CreateWindowExW failed, error {}",
            GetLastError()
        ));
    }
     SetWindowPos(
        hwnd,
        HWND_TOPMOST,
        vx,
        vy,
        vw,
        vh,
        SWP_NOACTIVATE | SWP_SHOWWINDOW
    );

    Ok(hwnd)
}

unsafe fn try_create_window_in_band(
    class_atom: ATOM,
    window_title: *const u16,
    hinstance: HINSTANCE,
) -> Option<HWND> {
    let user32 = to_wide_null("user32.dll");
    let module = LoadLibraryW(user32.as_ptr());
    if module.is_null() {
        return None;
    }

    let proc = GetProcAddress(module, b"CreateWindowInBand\0".as_ptr() as *const i8);
    if proc.is_null() {
        FreeLibrary(module);
        return None;
    }

    type CreateWindowInBandFn = unsafe extern "system" fn(
        DWORD,
        ATOM,
        *const u16,
        DWORD,
        i32,
        i32,
        i32,
        i32,
        HWND,
        HMENU,
        HINSTANCE,
        *mut c_void,
        DWORD,
    ) -> HWND;

    let create: CreateWindowInBandFn = mem::transmute(proc);

    let hwnd = create(
        WS_EX_TOPMOST | WS_EX_NOACTIVATE | WS_EX_TRANSPARENT,
        class_atom,
        window_title,
        WS_POPUP,
        0,
        0,
        0,
        0,
        ptr::null_mut(),
        ptr::null_mut(),
        hinstance,
        class_atom as usize as *mut c_void,
        ZBID_ABOVELOCK_UX,
    );

    FreeLibrary(module);

    if hwnd.is_null() {
        None
    } else {
        Some(hwnd)
    }
}

fn supports_capture_exclusion() -> bool {
    is_windows_version_or_greater(10, 0, 19041, 0, 0)
}

unsafe fn set_capture_exclusion(hwnd: HWND, enabled: bool) {
    if !supports_capture_exclusion() {
        return;
    }

    let affinity = if enabled { WDA_EXCLUDEFROMCAPTURE } else { WDA_NONE };
    if SetWindowDisplayAffinity(hwnd, affinity) == 0 {
        log::warn!(
            "SetWindowDisplayAffinity({:#x}) failed: {}",
            affinity,
            GetLastError()
        );
    } else {
        log::info!(
            "Successfully set window display affinity to {:#x} for overlay window",
            affinity
        );
    }
}

unsafe fn cloak_window(hwnd: HWND, cloak: bool) {
    let value: BOOL = if cloak { 1 } else { 0 };
    let hr = DwmSetWindowAttribute(
        hwnd,
        DWMWA_CLOAK,
        &value as *const BOOL as *const WinCvoid,
        mem::size_of::<BOOL>() as u32,
    );
    if hr < 0 {
        log::warn!(
            "DwmSetWindowAttribute(DWMWA_CLOAK, {}) failed: 0x{:08x}",
            cloak,
            hr as u32
        );
    }
}

unsafe fn configure_overlay_window(hwnd: HWND) {
    // Start without transparency to make sure window is visible when shown
    let current_style = GetWindowLongW(hwnd, GWL_EXSTYLE);
    let new_style = current_style
        | (WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_TOPMOST | WS_EX_NOACTIVATE | WS_EX_TRANSPARENT) as i32;
    SetWindowLongW(hwnd, GWL_EXSTYLE, new_style);

    // Set fully opaque 
    SetLayeredWindowAttributes(hwnd, 0, 255, LWA_ALPHA);
    ShowWindow(hwnd, SW_HIDE);
    UpdateWindow(hwnd);
}

unsafe fn show_overlay(hwnd: HWND) {
    log::info!("show_overlay called for hwnd {:?}", hwnd);
    if PRIVACY_ACTIVE.swap(true, Ordering::SeqCst) {
        log::warn!("Privacy mode already active, skipping show_overlay");
        return;
    }

    log::info!("Uncloaking overlay window");
    cloak_window(hwnd, false);

    let vx = GetSystemMetrics(SM_XVIRTUALSCREEN);
    let vy = GetSystemMetrics(SM_YVIRTUALSCREEN);
    let vw = GetSystemMetrics(SM_CXVIRTUALSCREEN);
    let vh = GetSystemMetrics(SM_CYVIRTUALSCREEN);

    log::info!("Setting overlay position: {}x{} at ({}, {})", vw, vh, vx, vy);

   
    // Force window repaint to ensure visibility
    winapi::um::winuser::InvalidateRect(hwnd, ptr::null(), 1);

    // Exclude only the overlay window from screen capture
    // This allows RustDesk to capture the desktop underneath while keeping the overlay visible to the user
    set_capture_exclusion(hwnd, true);

    hide_cursor_aggressive();
    apply_system_blank_cursors();
    start_cursor_enforcer();

    if let Err(err) = install_input_hooks() {
        log::error!("Failed to install input hooks: {}", err);
    }

    log::info!("Showing overlay window");
    ShowWindow(hwnd, SW_SHOW);
    UpdateWindow(hwnd);
    log::info!("Overlay window shown and updated");
}

unsafe fn hide_overlay(hwnd: HWND) {
    if !PRIVACY_ACTIVE.swap(false, Ordering::SeqCst) {
        return;
    }

    ShowWindow(hwnd, SW_HIDE);
    cloak_window(hwnd, true);
    set_capture_exclusion(hwnd, false);

    remove_input_hooks();
    restore_system_cursors();
    show_cursor_restore();
}

unsafe fn install_input_hooks() -> ResultType<()> {
    log::info!("install_input_hooks called");
    
    if HOOKS_INSTALLED.load(Ordering::SeqCst) {
        log::info!("Input hooks already installed, skipping");
        return Ok(());
    }

    let hinstance = GetModuleHandleW(ptr::null_mut());

    log::info!("Installing keyboard hook...");
    let keyboard_hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook_proc), hinstance, 0);
    if keyboard_hook.is_null() {
        let error = GetLastError();
        bail!("Failed to install keyboard hook, error {}", error);
    }

    log::info!("Installing mouse hook...");
    let mouse_hook = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_hook_proc), hinstance, 0);
    if mouse_hook.is_null() {
        let error = GetLastError();
        UnhookWindowsHookEx(keyboard_hook);
        bail!("Failed to install mouse hook, error {}", error);
    }

    KEYBOARD_HOOK = keyboard_hook;
    MOUSE_HOOK = mouse_hook;
    HOOKS_INSTALLED.store(true, Ordering::SeqCst);
    
    log::info!("Input hooks installed successfully! Keyboard hook: {:?}, Mouse hook: {:?}", keyboard_hook, mouse_hook);
    Ok(())
}

unsafe fn remove_input_hooks() {
    if !HOOKS_INSTALLED.swap(false, Ordering::SeqCst) {
        return;
    }

    if !KEYBOARD_HOOK.is_null() {
        UnhookWindowsHookEx(KEYBOARD_HOOK);
        KEYBOARD_HOOK = ptr::null_mut();
    }

    if !MOUSE_HOOK.is_null() {
        UnhookWindowsHookEx(MOUSE_HOOK);
        MOUSE_HOOK = ptr::null_mut();
    }
}

unsafe fn hide_cursor_aggressive() {
    if CURSOR_HIDDEN.swap(true, Ordering::SeqCst) {
        return;
    }

    SetCursor(ptr::null_mut());
    let mut count = ShowCursor(0);
    while count >= 0 {
        count = ShowCursor(0);
    }
}

unsafe fn show_cursor_restore() {
    let mut count = ShowCursor(1);
    while count < 0 {
        count = ShowCursor(1);
    }

    let arrow = winapi::um::winuser::LoadCursorW(ptr::null_mut(), IDC_ARROW);
    SetCursor(arrow);

    let mut point = POINT { x: 0, y: 0 };
    if GetCursorPos(&mut point) != 0 {
        SetCursorPos(point.x, point.y);
    }

    CURSOR_HIDDEN.store(false, Ordering::SeqCst);
}

fn start_cursor_enforcer() {
    if CURSOR_ENFORCER_RUNNING
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return;
    }

    thread::spawn(|| {
        while PRIVACY_ACTIVE.load(Ordering::SeqCst) {
            unsafe {
                SetCursor(ptr::null_mut());
                let mut count = ShowCursor(0);
                while count >= 0 {
                    count = ShowCursor(0);
                }
            }
            thread::sleep(Duration::from_millis(50));
        }
        CURSOR_ENFORCER_RUNNING.store(false, Ordering::SeqCst);
    });
}

// Create a fully transparent cursor
unsafe fn create_invisible_cursor() -> Option<HCURSOR> {
    // 1x1 transparent bitmaps
    let hbm_mask: HBITMAP = CreateBitmap(1, 1, 1, 1, ptr::null());
    let hbm_color: HBITMAP = CreateBitmap(1, 1, 1, 32, ptr::null());
    if hbm_mask.is_null() || hbm_color.is_null() {
        if !hbm_mask.is_null() { DeleteObject(hbm_mask as _); }
        if !hbm_color.is_null() { DeleteObject(hbm_color as _); }
        log::warn!("Failed to create bitmaps for invisible cursor");
        return None;
    }

    let mut ii: ICONINFO = mem::zeroed();
    ii.fIcon = 0; // cursor, not icon
    ii.xHotspot = 0;
    ii.yHotspot = 0;
    ii.hbmMask = hbm_mask;
    ii.hbmColor = hbm_color;
    let hcursor = CreateIconIndirect(&mut ii);

    // We can delete the bitmaps after creating the cursor
    DeleteObject(hbm_mask as _);
    DeleteObject(hbm_color as _);

    if hcursor.is_null() {
        log::warn!("CreateIconIndirect failed for invisible cursor");
        None
    } else {
        Some(hcursor)
    }
}

unsafe fn apply_system_blank_cursors() {
    if CURSOR_SYSTEM_REPLACED.swap(true, Ordering::SeqCst) {
        return;
    }

    let cursor_ids = [
        OCR_NORMAL, OCR_IBEAM, OCR_CROSS, OCR_HAND, OCR_HELP, OCR_NO, OCR_SIZEALL,
        OCR_SIZENESW, OCR_SIZENS, OCR_SIZENWSE, OCR_SIZEWE, OCR_UP, OCR_WAIT,
    ];

    for id in cursor_ids.iter() {
        if let Some(cur) = create_invisible_cursor() {
            let ok = SetSystemCursor(cur, *id);
            if ok == 0 {
                log::warn!("SetSystemCursor failed for id {}: {}", id, GetLastError());
            }
            // SetSystemCursor takes ownership of HCURSOR, no need to destroy here
        }
    }
}

unsafe fn restore_system_cursors() {
    if !CURSOR_SYSTEM_REPLACED.swap(false, Ordering::SeqCst) {
        return;
    }
    // Reload default system cursors
    let ok = SystemParametersInfoW(SPI_SETCURSORS, 0, ptr::null_mut(), 0);
    if ok == 0 {
        log::warn!("SystemParametersInfoW(SPI_SETCURSORS) failed: {}", GetLastError());
    }
}

unsafe extern "system" fn keyboard_hook_proc(
    code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> isize {
    if code == HC_ACTION {
        let info = &*(lparam as *const KBDLLHOOKSTRUCT);
        
        // Log all keyboard events to debug input sources
        log::debug!("🔵 Keyboard hook: vkCode={}, flags=0x{:x}, dwExtraInfo=0x{:x}, wparam={}", 
                   info.vkCode, info.flags, info.dwExtraInfo, wparam);
        
        // Allow agent input (ENIGO) to pass through
        if info.dwExtraInfo == ENIGO_INPUT_EXTRA_VALUE {
            log::info!("✅ Allowing ENIGO keyboard input to pass through");
            
           
            
            return CallNextHookEx(KEYBOARD_HOOK, code, wparam, lparam);
        }
        
        // Allow injected input (from agent) to pass through
        const LLKHF_INJECTED: u32 = 0x01;
        const LLKHF_LOWER_IL_INJECTED: u32 = 0x02;
        if (info.flags & (LLKHF_INJECTED | LLKHF_LOWER_IL_INJECTED)) != 0 {
            log::info!("✅ Allowing injected keyboard input to pass through (flags=0x{:x})", info.flags);
            
           
            
            return CallNextHookEx(KEYBOARD_HOOK, code, wparam, lparam);
        }
        
        
        
        log::info!("🚫 Blocking user keyboard input");
        return 1;
        
        // Block user input, but allow Ctrl+P to exit privacy mode
        /*let wparam_uint = wparam as UINT;
        if wparam_uint == WM_KEYDOWN {
            let ctrl_down = (GetKeyState(VK_CONTROL) as u16) & 0x8000 != 0;
            let key = LOBYTE(info.vkCode as _);
            if ctrl_down && (key == b'p' || key == b'P') {
                if let Some(Err(e)) = super::turn_off_privacy(
                    super::INVALID_PRIVACY_MODE_CONN_ID,
                    Some(super::PrivacyModeState::OffByPeer),
                ) {
                    log::error!("Failed to off_privacy {}", e);
                }
            }
        }
        
        // Block all other user keyboard input
        return 1;*/
    }

    CallNextHookEx(KEYBOARD_HOOK, code, wparam, lparam)
}

unsafe extern "system" fn mouse_hook_proc(
    code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> isize {
    if code < 0 {
        // Always pass through if code is negative
        return CallNextHookEx(MOUSE_HOOK, code, wparam, lparam);
    }
    
    if code == HC_ACTION {
        let info = &*(lparam as *const MSLLHOOKSTRUCT);
        const LLMHF_INJECTED: u32 = 0x01;
        const LLMHF_LOWER_IL_INJECTED: u32 = 0x02;
        
        // Always hide cursor during privacy mode
        SetCursor(ptr::null_mut());

        
        // Log all mouse events to debug input sources
        log::debug!("🔴 Mouse hook: x={}, y={}, flags=0x{:x}, dwExtraInfo=0x{:x}, wparam={}", 
                   info.pt.x, info.pt.y, info.flags, info.dwExtraInfo, wparam);
        
        // Allow agent input (ENIGO) to pass through
        if info.dwExtraInfo == ENIGO_INPUT_EXTRA_VALUE {
            log::info!("✅ Allowing ENIGO mouse input to pass through");
         
            // Don't block this input - pass it to the next hook
            return CallNextHookEx(MOUSE_HOOK, code, wparam, lparam);
        }
        
        // Allow injected input (from agent) to pass through
        if (info.flags & (LLMHF_INJECTED | LLMHF_LOWER_IL_INJECTED)) != 0 {
            log::info!("✅ Allowing injected mouse input to pass through (flags=0x{:x})", info.flags);
            
           
            // Don't block this input - pass it to the next hook
            return CallNextHookEx(MOUSE_HOOK, code, wparam, lparam);
        }
         // Block all user mouse input
        
        log::info!("🚫 Blocking user mouse input - returning 1");
        return 1; // Block input
    }

    // For other codes, pass through normally
    CallNextHookEx(MOUSE_HOOK, code, wparam, lparam)
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {

        WM_PAINT => {
            let mut ps: PAINTSTRUCT = mem::zeroed();
            let hdc = BeginPaint(hwnd, &mut ps);
            if !hdc.is_null() {
                let mut rect = RECT {
                    left: 0,
                    top: 0,
                    right: 0,
                    bottom: 0,
                };
                GetClientRect(hwnd, &mut rect);
                
                // Fill with solid black first (like DLL does)
                let black_brush = CreateSolidBrush(RGB(0, 0, 0)); 
                FillRect(hdc, &rect, black_brush);
                DeleteObject(black_brush as _);
                
                // Add very visible red indicators at corners to confirm it's working
                let red_brush = CreateSolidBrush(RGB(255, 0, 0));
                let corner_size = 100;
                
                // Top-left corner
                let tl_rect = RECT {
                    left: rect.left,
                    top: rect.top,
                    right: rect.left + corner_size,
                    bottom: rect.top + corner_size,
                };
                FillRect(hdc, &tl_rect, red_brush);
                
                // Top-right corner  
                let tr_rect = RECT {
                    left: rect.right - corner_size,
                    top: rect.top,
                    right: rect.right,
                    bottom: rect.top + corner_size,
                };
                FillRect(hdc, &tr_rect, red_brush);
                
                // Bottom-left corner
                let bl_rect = RECT {
                    left: rect.left,
                    top: rect.bottom - corner_size,
                    right: rect.left + corner_size,
                    bottom: rect.bottom,
                };
                FillRect(hdc, &bl_rect, red_brush);
                
                // Bottom-right corner
                let br_rect = RECT {
                    left: rect.right - corner_size,
                    top: rect.bottom - corner_size,
                    right: rect.right,
                    bottom: rect.bottom,
                };
                FillRect(hdc, &br_rect, red_brush);
                
                DeleteObject(red_brush as _);
                EndPaint(hwnd, &mut ps);
            }
            0
        }
        WM_SETCURSOR => {
            SetCursor(ptr::null_mut());
            1
        }
        // Make the overlay window hit-test transparent so mouse events pass to underlying apps
        WM_NCHITTEST => {
            HTTRANSPARENT as LRESULT
        }
        WM_WINDOWPOSCHANGING => {
            let window_pos = lparam as *mut WINDOWPOS;
            if !window_pos.is_null() {
                (*window_pos).flags |= SWP_NOMOVE | SWP_NOSIZE;
            }
            0
        }
        WM_CLOSE => {
            hide_overlay(hwnd);
            DestroyWindow(hwnd);
            0
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

pub fn is_supported() -> bool {
    true
}

pub fn init_cleanup() -> ResultType<()> {
    log::info!("Initializing direct overlay privacy cleanup");

    std::panic::set_hook(Box::new(|info| {
        log::error!("PANIC in privacy mode: {:?}", info);
        if let Err(err) = cleanup_on_session_close() {
            log::error!("Cleanup after panic failed: {}", err);
        }
    }));

    cleanup_on_session_close()
}

pub fn emergency_cleanup() {
    log::error!("Emergency privacy cleanup invoked");
    if let Err(err) = cleanup_on_session_close() {
        log::error!("Emergency cleanup failed: {}", err);
    }
}

pub fn cleanup_on_session_close() -> ResultType<()> {
    let controller = OVERLAY_CONTROLLER.lock().unwrap().take();

    if let Some(controller) = controller {
        let _ = controller.post_message(WM_PRIVACY_SHUTDOWN, 0, 0);
        controller.join_thread();
    }

    unsafe {
        remove_input_hooks();
        restore_system_cursors();
        show_cursor_restore();
    }

    PRIVACY_ACTIVE.store(false, Ordering::SeqCst);
    CURSOR_ENFORCER_RUNNING.store(false, Ordering::SeqCst);
    CURSOR_HIDDEN.store(false, Ordering::SeqCst);
    HOOKS_INSTALLED.store(false, Ordering::SeqCst);

    Ok(())
}

pub struct DirectOverlayPrivacyMode {
    impl_key: String,
    conn_id: i32,
    controller: Option<Arc<OverlayController>>,
}

impl PrivacyMode for DirectOverlayPrivacyMode {
    fn is_async_privacy_mode(&self) -> bool {
        false
    }

    fn init(&self) -> ResultType<()> {
        start_overlay_thread().map(|_| ())
    }

    fn clear(&mut self) {
        allow_err!(self.turn_off_privacy(self.conn_id, None));
    }

    fn turn_on_privacy(&mut self, conn_id: i32) -> ResultType<bool> {
        log::info!("turn_on_privacy called for connection {}", conn_id);
        if self.check_on_conn_id(conn_id)? {
            log::debug!("Privacy mode for connection {} already active", conn_id);
            return Ok(true);
        }

        log::info!("Starting overlay thread...");
        let controller = start_overlay_thread()?;
        log::info!("Posting WM_PRIVACY_SHOW message...");
        controller.post_message(WM_PRIVACY_SHOW, 0, 0)?;

        self.controller = Some(controller);
        self.conn_id = conn_id;
        log::info!("Direct overlay privacy mode enabled for connection {}", conn_id);
        Ok(true)
    }

    fn turn_off_privacy(
        &mut self,
        conn_id: i32,
        state: Option<PrivacyModeState>,
    ) -> ResultType<()> {
        self.check_off_conn_id(conn_id)?;

        if let Some(controller) = &self.controller {
            let _ = controller.post_message(WM_PRIVACY_HIDE, 0, 0);
        }

        if let Some(state) = state {
            allow_err!(super::set_privacy_mode_state(
                conn_id,
                state,
                PRIVACY_MODE_IMPL.to_string(),
                1_000
            ));
        }

        self.conn_id = INVALID_PRIVACY_MODE_CONN_ID;
        self.controller = None;
        log::info!("Direct overlay privacy mode disabled for connection {}", conn_id);
        Ok(())
    }

    #[inline]
    fn pre_conn_id(&self) -> i32 {
        self.conn_id
    }

    #[inline]
    fn get_impl_key(&self) -> &str {
        &self.impl_key
    }
}

impl DirectOverlayPrivacyMode {
    pub fn new(impl_key: &str) -> Self {
        Self {
            impl_key: impl_key.to_owned(),
            conn_id: INVALID_PRIVACY_MODE_CONN_ID,
            controller: None,
        }
    }

    pub fn is_available() -> bool {
        true
    }
}

impl Drop for DirectOverlayPrivacyMode {
    fn drop(&mut self) {
        if let Err(err) = cleanup_on_session_close() {
            log::error!("Cleanup during drop failed: {}", err);
        }
    }
}
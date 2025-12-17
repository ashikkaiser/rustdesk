// Windows Separate Desktop Privacy Mode
// Creates an isolated desktop for remote agent while keeping user's desktop untouched
// This is how enterprise RMM tools provide secure, invisible remote access

use super::{PrivacyMode, INVALID_PRIVACY_MODE_CONN_ID};
use crate::privacy_mode::PrivacyModeState;

pub const PRIVACY_MODE_IMPL: &str = "privacy_mode_impl_separate_desktop";

use hbb_common::{allow_err, bail, log, ResultType};
use std::ffi::OsStr;
use std::mem;
use std::os::windows::ffi::OsStrExt;
use std::ptr;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use winapi::{
    shared::{
        minwindef::{BOOL, DWORD, FALSE, TRUE},
        ntdef::HANDLE,
        windef::{HDESK, HWND},
    },
    um::{
        handleapi::CloseHandle,
        processthreadsapi::{
            CreateProcessW, GetCurrentProcess, GetCurrentProcessId, 
            OpenProcess, TerminateProcess, PROCESS_INFORMATION, STARTUPINFOW,
        },
        winbase::{CREATE_NEW_CONSOLE, CREATE_UNICODE_ENVIRONMENT, STARTF_USESHOWWINDOW},
        winnt::{PROCESS_QUERY_INFORMATION, PROCESS_TERMINATE},
        winuser::{
            CloseDesktop, CreateDesktopW, GetThreadDesktop, OpenDesktopW, 
            SetThreadDesktop, SwitchDesktop, DESKTOP_CREATEWINDOW, DESKTOP_CREATEMENU,
            DESKTOP_ENUMERATE, DESKTOP_HOOKCONTROL, DESKTOP_JOURNALPLAYBACK,
            DESKTOP_JOURNALRECORD, DESKTOP_READOBJECTS, DESKTOP_SWITCHDESKTOP,
            DESKTOP_WRITEOBJECTS, SW_SHOW,
        },
        wincon::GetConsoleWindow,
    },
};

// Desktop access flags for full functionality
const DESKTOP_ALL_ACCESS: DWORD = DESKTOP_READOBJECTS
    | DESKTOP_CREATEWINDOW
    | DESKTOP_CREATEMENU
    | DESKTOP_HOOKCONTROL
    | DESKTOP_JOURNALRECORD
    | DESKTOP_JOURNALPLAYBACK
    | DESKTOP_ENUMERATE
    | DESKTOP_WRITEOBJECTS
    | DESKTOP_SWITCHDESKTOP;

// Store just the desktop name and process info (thread-safe)
struct DesktopInfo {
    desktop_name: String,
    explorer_pid: Option<DWORD>,
}

unsafe impl Send for DesktopInfo {}
unsafe impl Sync for DesktopInfo {}

static AGENT_DESKTOP_INFO: Mutex<Option<DesktopInfo>> = Mutex::new(None);

fn to_wide_null(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
}

/// Represents an isolated desktop for remote agent operations
struct AgentDesktop {
    desktop_name: String,
    desktop_handle: HDESK,
    explorer_process: Option<ProcessHandle>,
    original_desktop: HDESK,
}

/// Safe wrapper for Windows process handle
struct ProcessHandle {
    handle: HANDLE,
    pid: DWORD,
}

impl ProcessHandle {
    fn new(handle: HANDLE, pid: DWORD) -> Self {
        Self { handle, pid }
    }
}

impl Drop for ProcessHandle {
    fn drop(&mut self) {
        unsafe {
            if !self.handle.is_null() {
                // Try graceful termination first
                TerminateProcess(self.handle, 0);
                CloseHandle(self.handle);
                log::info!("Terminated process {}", self.pid);
            }
        }
    }
}

impl AgentDesktop {
    /// Create a new isolated desktop
    fn create(desktop_name: &str) -> ResultType<Self> {
        unsafe {
            // Save current desktop to restore later
            let original_desktop = GetThreadDesktop(winapi::um::processthreadsapi::GetCurrentThreadId());
            if original_desktop.is_null() {
                bail!("Failed to get current desktop");
            }

            log::info!("Creating isolated desktop: {}", desktop_name);
            
            let desktop_name_wide = to_wide_null(desktop_name);
            let desktop_handle = CreateDesktopW(
                desktop_name_wide.as_ptr(),
                ptr::null_mut(),
                ptr::null_mut(),
                0, // Don't switch to this desktop automatically
                DESKTOP_ALL_ACCESS,
                ptr::null_mut(),
            );

            if desktop_handle.is_null() {
                let error = winapi::um::errhandlingapi::GetLastError();
                bail!("Failed to create desktop '{}': error {}", desktop_name, error);
            }

            log::info!("Desktop '{}' created successfully (handle: {:?})", desktop_name, desktop_handle);

            Ok(Self {
                desktop_name: desktop_name.to_string(),
                desktop_handle,
                explorer_process: None,
                original_desktop,
            })
        }
    }

    /// Start Explorer on the isolated desktop (gives Start Menu, taskbar, etc.)
    fn start_explorer(&mut self) -> ResultType<()> {
        unsafe {
            log::info!("Starting Explorer on desktop '{}'...", self.desktop_name);

            // Build desktop path for CreateProcess
            let desktop_path = format!("winsta0\\{}", self.desktop_name);
            let desktop_path_wide = to_wide_null(&desktop_path);

            // Locate explorer.exe
            let explorer_path = std::env::var("WINDIR")
                .unwrap_or_else(|_| "C:\\Windows".to_string())
                + "\\explorer.exe";
            let explorer_path_wide = to_wide_null(&explorer_path);

            let mut startup_info: STARTUPINFOW = mem::zeroed();
            startup_info.cb = mem::size_of::<STARTUPINFOW>() as u32;
            startup_info.lpDesktop = desktop_path_wide.as_ptr() as *mut u16;
            startup_info.dwFlags = STARTF_USESHOWWINDOW;
            startup_info.wShowWindow = SW_SHOW as u16;

            let mut process_info: PROCESS_INFORMATION = mem::zeroed();

            let success = CreateProcessW(
                explorer_path_wide.as_ptr(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                FALSE,
                CREATE_NEW_CONSOLE | CREATE_UNICODE_ENVIRONMENT,
                ptr::null_mut(),
                ptr::null_mut(),
                &mut startup_info,
                &mut process_info,
            );

            if success == 0 {
                let error = winapi::um::errhandlingapi::GetLastError();
                bail!("Failed to start Explorer on desktop '{}': error {}", self.desktop_name, error);
            }

            // Close thread handle (we don't need it)
            CloseHandle(process_info.hThread);

            log::info!(
                "Explorer started on desktop '{}' (PID: {})",
                self.desktop_name,
                process_info.dwProcessId
            );

            self.explorer_process = Some(ProcessHandle::new(
                process_info.hProcess,
                process_info.dwProcessId,
            ));

            // Give Explorer time to initialize
            thread::sleep(Duration::from_millis(2000));

            Ok(())
        }
    }

    /// Get the desktop handle for use by other processes
    fn handle(&self) -> HDESK {
        self.desktop_handle
    }

    /// Get the desktop name (e.g., "CloudyDeskAgent")
    fn name(&self) -> &str {
        &self.desktop_name
    }

    /// Cleanup: close desktop and terminate processes
    fn cleanup(&mut self) {
        unsafe {
            log::info!("Cleaning up desktop '{}'...", self.desktop_name);

            // Restore original desktop for this thread
            if !self.original_desktop.is_null() {
                SetThreadDesktop(self.original_desktop);
            }

            // Terminate Explorer (ProcessHandle Drop will handle this)
            self.explorer_process = None;

            // Close desktop handle
            if !self.desktop_handle.is_null() {
                CloseDesktop(self.desktop_handle);
                self.desktop_handle = ptr::null_mut();
                log::info!("Desktop '{}' closed", self.desktop_name);
            }
        }
    }
}

impl Drop for AgentDesktop {
    fn drop(&mut self) {
        self.cleanup();
    }
}

/// Create and initialize the agent desktop
pub fn create_agent_desktop() -> ResultType<()> {
    let mut guard = AGENT_DESKTOP_INFO.lock().unwrap();
    
    if guard.is_some() {
        log::warn!("Agent desktop already exists, skipping creation");
        return Ok(());
    }

    let desktop_name = "CloudyDeskAgent";
    unsafe {
        // Save current desktop
        let thread_id = winapi::um::processthreadsapi::GetCurrentThreadId();
        log::debug!("Current thread ID: {}", thread_id);
        
        let original_desktop = GetThreadDesktop(thread_id);
        if original_desktop.is_null() {
            let error = winapi::um::errhandlingapi::GetLastError();
            bail!("Failed to get current desktop for thread {}: error code {}", thread_id, error);
        }

        log::info!("Current desktop handle: {:?}, creating isolated desktop: {}", original_desktop, desktop_name);
        
        let desktop_name_wide = to_wide_null(desktop_name);
        let desktop_handle = CreateDesktopW(
            desktop_name_wide.as_ptr(),
            ptr::null_mut(),
            ptr::null_mut(),
            0,
            DESKTOP_ALL_ACCESS,
            ptr::null_mut(),
        );

        if desktop_handle.is_null() {
            let error = winapi::um::errhandlingapi::GetLastError();
            bail!("Failed to create desktop '{}': error {}", desktop_name, error);
        }

        log::info!("Desktop '{}' created successfully", desktop_name);

        // Start Explorer on the desktop
        let desktop_path = format!("winsta0\\{}", desktop_name);
        let desktop_path_wide = to_wide_null(&desktop_path);

        let explorer_path = std::env::var("WINDIR")
            .unwrap_or_else(|_| "C:\\Windows".to_string())
            + "\\explorer.exe";
        let explorer_path_wide = to_wide_null(&explorer_path);

        let mut startup_info: STARTUPINFOW = mem::zeroed();
        startup_info.cb = mem::size_of::<STARTUPINFOW>() as u32;
        startup_info.lpDesktop = desktop_path_wide.as_ptr() as *mut u16;
        startup_info.dwFlags = STARTF_USESHOWWINDOW;
        startup_info.wShowWindow = SW_SHOW as u16;

        let mut process_info: PROCESS_INFORMATION = mem::zeroed();

        let success = CreateProcessW(
            explorer_path_wide.as_ptr(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            FALSE,
            CREATE_NEW_CONSOLE | CREATE_UNICODE_ENVIRONMENT,
            ptr::null_mut(),
            ptr::null_mut(),
            &mut startup_info,
            &mut process_info,
        );

        if success == 0 {
            let error = winapi::um::errhandlingapi::GetLastError();
            CloseDesktop(desktop_handle);
            bail!("Failed to start Explorer on desktop '{}': error {}", desktop_name, error);
        }

        CloseHandle(process_info.hThread);

        log::info!(
            "Explorer started on desktop '{}' (PID: {})",
            desktop_name,
            process_info.dwProcessId
        );

        // Store info
        *guard = Some(DesktopInfo {
            desktop_name: desktop_name.to_string(),
            explorer_pid: Some(process_info.dwProcessId),
        });

        // Close handles (we don't need to keep them)
        CloseHandle(process_info.hProcess);
        CloseDesktop(desktop_handle);
    }

    // Give Explorer time to initialize
    thread::sleep(Duration::from_millis(2000));

    Ok(())
}

/// Destroy the agent desktop
pub fn destroy_agent_desktop() -> ResultType<()> {
    let mut guard = AGENT_DESKTOP_INFO.lock().unwrap();
    
    if let Some(info) = guard.take() {
        unsafe {
            log::info!("Destroying agent desktop '{}'...", info.desktop_name);

            // Terminate Explorer process
            if let Some(pid) = info.explorer_pid {
                let process = OpenProcess(PROCESS_TERMINATE, FALSE, pid);
                if !process.is_null() {
                    TerminateProcess(process, 0);
                    CloseHandle(process);
                    log::info!("Terminated Explorer process {}", pid);
                }
            }

            // Open and close the desktop to clean it up
            let desktop_name_wide = to_wide_null(&info.desktop_name);
            let desktop = OpenDesktopW(
                desktop_name_wide.as_ptr(),
                0,
                FALSE,
                DESKTOP_ALL_ACCESS,
            );
            
            if !desktop.is_null() {
                CloseDesktop(desktop);
                log::info!("Desktop '{}' closed", info.desktop_name);
            }
        }
    }
    
    Ok(())
}

/// Switch current thread to agent desktop (for screen capture)
pub fn switch_to_agent_desktop() -> ResultType<()> {
    unsafe {
        let guard = AGENT_DESKTOP_INFO.lock().unwrap();
        
        if let Some(info) = guard.as_ref() {
            let desktop_name_wide = to_wide_null(&info.desktop_name);
            let desktop = OpenDesktopW(
                desktop_name_wide.as_ptr(),
                0,
                FALSE,
                DESKTOP_ALL_ACCESS,
            );

            if desktop.is_null() {
                let error = winapi::um::errhandlingapi::GetLastError();
                bail!("Failed to open agent desktop: error {}", error);
            }

            let success = SetThreadDesktop(desktop);
            if success == 0 {
                let error = winapi::um::errhandlingapi::GetLastError();
                CloseDesktop(desktop);
                bail!("Failed to switch to agent desktop: error {}", error);
            }
            
            // Don't close desktop - thread is using it
            log::info!("Thread switched to agent desktop '{}'", info.desktop_name);
            Ok(())
        } else {
            bail!("Agent desktop not initialized");
        }
    }
}

/// Switch current thread back to original desktop
pub fn switch_to_original_desktop() -> ResultType<()> {
    unsafe {
        // Get the default desktop
        let desktop_name_wide = to_wide_null("Default");
        let desktop = OpenDesktopW(
            desktop_name_wide.as_ptr(),
            0,
            FALSE,
            DESKTOP_ALL_ACCESS,
        );

        if desktop.is_null() {
            let error = winapi::um::errhandlingapi::GetLastError();
            bail!("Failed to open default desktop: error {}", error);
        }

        let success = SetThreadDesktop(desktop);
        if success == 0 {
            let error = winapi::um::errhandlingapi::GetLastError();
            CloseDesktop(desktop);
            bail!("Failed to switch back to default desktop: error {}", error);
        }
        
        log::info!("Thread switched back to default desktop");
        Ok(())
    }
}

/// Get the desktop handle for launching processes on agent desktop
pub fn get_agent_desktop_name() -> ResultType<String> {
    let guard = AGENT_DESKTOP_INFO.lock().unwrap();
    
    if let Some(info) = guard.as_ref() {
        Ok(format!("winsta0\\{}", info.desktop_name))
    } else {
        bail!("Agent desktop not initialized");
    }
}

pub fn is_supported() -> bool {
    // Separate desktop works on all Windows versions
    true
}

pub fn init_cleanup() -> ResultType<()> {
    log::info!("Initializing separate desktop privacy mode cleanup");
    
    std::panic::set_hook(Box::new(|info| {
        log::error!("PANIC in separate desktop mode: {:?}", info);
        if let Err(err) = destroy_agent_desktop() {
            log::error!("Cleanup after panic failed: {}", err);
        }
    }));

    // Clean up any leftover desktops from previous crashes
    destroy_agent_desktop()
}

pub fn emergency_cleanup() {
    log::error!("Emergency separate desktop cleanup invoked");
    if let Err(err) = destroy_agent_desktop() {
        log::error!("Emergency cleanup failed: {}", err);
    }
}

pub struct SeparateDesktopPrivacyMode {
    impl_key: String,
    conn_id: i32,
}

impl PrivacyMode for SeparateDesktopPrivacyMode {
    fn is_async_privacy_mode(&self) -> bool {
        false
    }

    fn init(&self) -> ResultType<()> {
        // Desktop creation happens in turn_on_privacy
        Ok(())
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

        log::info!("Creating agent desktop...");
        create_agent_desktop()?;

        log::info!("Switching capture thread to agent desktop...");
        switch_to_agent_desktop()?;

        self.conn_id = conn_id;
        log::info!("Separate desktop privacy mode enabled for connection {}", conn_id);
        Ok(true)
    }

    fn turn_off_privacy(
        &mut self,
        conn_id: i32,
        state: Option<PrivacyModeState>,
    ) -> ResultType<()> {
        self.check_off_conn_id(conn_id)?;

        log::info!("Switching back to original desktop...");
        allow_err!(switch_to_original_desktop());

        log::info!("Destroying agent desktop...");
        destroy_agent_desktop()?;

        if let Some(state) = state {
            allow_err!(super::set_privacy_mode_state(
                conn_id,
                state,
                PRIVACY_MODE_IMPL.to_string(),
                1_000
            ));
        }

        self.conn_id = INVALID_PRIVACY_MODE_CONN_ID;
        log::info!("Separate desktop privacy mode disabled for connection {}", conn_id);
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

impl SeparateDesktopPrivacyMode {
    pub fn new(impl_key: &str) -> Self {
        log::info!("Creating SeparateDesktopPrivacyMode with impl_key: '{}'", impl_key);
        Self {
            impl_key: impl_key.to_owned(),
            conn_id: INVALID_PRIVACY_MODE_CONN_ID,
        }
    }

    pub fn is_available() -> bool {
        // Works on all Windows versions
        true
    }
}

impl Drop for SeparateDesktopPrivacyMode {
    fn drop(&mut self) {
        if let Err(err) = destroy_agent_desktop() {
            log::error!("Cleanup during drop failed: {}", err);
        }
    }
}

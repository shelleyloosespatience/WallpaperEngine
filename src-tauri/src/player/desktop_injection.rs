use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use windows::core::{BOOL, PCWSTR};
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, FindWindowExW, FindWindowW, GetWindowLongPtrW, SendMessageTimeoutW,
    SetLayeredWindowAttributes, SetParent, SetWindowLongPtrW, SetWindowPos, ShowWindow, GWL_STYLE,
    HWND_BOTTOM, LWA_ALPHA, SMTO_NORMAL, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
    SW_SHOWNA, WS_CHILD, WS_DISABLED, WS_POPUP,
};

lazy_static::lazy_static! {
    static ref WATCHDOG_HANDLE: Arc<Mutex<Option<thread::JoinHandle<()>>>> = Arc::new(Mutex::new(None));
    static ref WATCHDOG_STOP_FLAG: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
    static ref CURRENT_HWND: Arc<Mutex<Option<isize>>> = Arc::new(Mutex::new(None));
    static ref WINDOW_BOUNDS: Arc<Mutex<(i32, i32, i32, i32)>> = Arc::new(Mutex::new((0, 0, 1920, 1080)));
    static ref WORKERW_HANDLE: Arc<Mutex<Option<isize>>> = Arc::new(Mutex::new(None));
    static ref SHELLVIEW_HANDLE: Arc<Mutex<Option<isize>>> = Arc::new(Mutex::new(None));
    static ref PROGMAN_HANDLE: Arc<Mutex<Option<isize>>> = Arc::new(Mutex::new(None));
        static ref IS_WINDOWS_11: Arc<Mutex<Option<bool>>> = Arc::new(Mutex::new(None));
}

/// Check if running on Windows 11 (any version, build >= 22000)
/// Windows 11 introduced the raised desktop compositor architecture, i read it from thier release notes, refer to them ig
/// that requires different injection method than Windows 10
fn is_windows_11_or_later() -> bool {
    {
        let cached = IS_WINDOWS_11.lock().unwrap();
        if let Some(result) = *cached {
            return result;
        }
    }

    let build_number = get_windows_build_number();
    // Windows 11 starts at build 22000, refer to their release notes
    // All Windows 11 versions use the new compositor architecture (assumption lmao)
    let is_windows11 = build_number >= 22000;
    *IS_WINDOWS_11.lock().unwrap() = Some(is_windows11);
    is_windows11
}

fn get_windows_build_number() -> u32 {
    unsafe {
        use windows::Win32::System::SystemInformation::OSVERSIONINFOEXW;

        let mut version_info: OSVERSIONINFOEXW = std::mem::zeroed();
        version_info.dwOSVersionInfoSize = std::mem::size_of::<OSVERSIONINFOEXW>() as u32;

        type RtlGetVersion = unsafe extern "system" fn(*mut OSVERSIONINFOEXW) -> i32;

        match windows::Win32::System::LibraryLoader::LoadLibraryW(windows::core::w!("ntdll.dll")) {
            Ok(ntdll) => {
                match windows::Win32::System::LibraryLoader::GetProcAddress(
                    ntdll,
                    windows::core::s!("RtlGetVersion"),
                ) {
                    Some(proc_addr) => {
                        let rtl_get_version: RtlGetVersion = std::mem::transmute(proc_addr);
                        let status = rtl_get_version(&mut version_info as *mut OSVERSIONINFOEXW);

                        if status == 0 {
                            return version_info.dwBuildNumber;
                        }
                    }
                    None => {}
                }
            }
            Err(_) => {}
        }

        0
    }
}

pub fn inject_behind_desktop(
    hwnd: HWND,
    _x: i32,
    _y: i32,
    _width: i32,
    _height: i32,
) -> Result<(), String> {
    stop_watchdog();
    thread::sleep(Duration::from_millis(300));

    unsafe {
        let progman = FindWindowW(
            PCWSTR(windows::core::w!("Progman").as_ptr()),
            PCWSTR(windows::core::w!("Program Manager").as_ptr()),
        )
        .map_err(|e| format!("FindWindowW failed: {}", e))?;

        use windows::Win32::UI::WindowsAndMessaging::{
            GetSystemMetrics, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN,
            SM_YVIRTUALSCREEN,
        };

        let virtual_x = GetSystemMetrics(SM_XVIRTUALSCREEN);
        let virtual_y = GetSystemMetrics(SM_YVIRTUALSCREEN);
        let virtual_width = GetSystemMetrics(SM_CXVIRTUALSCREEN);
        let virtual_height = GetSystemMetrics(SM_CYVIRTUALSCREEN);

        *CURRENT_HWND.lock().unwrap() = Some(hwnd.0 as isize);
        *WINDOW_BOUNDS.lock().unwrap() = (virtual_x, virtual_y, virtual_width, virtual_height);
        *PROGMAN_HANDLE.lock().unwrap() = Some(progman.0 as isize);

        // Windows 11 (all versions, build >= 22000) uses the new compositor architecture
        // with "raised desktop" that requires different injection method
        if is_windows_11_or_later() {
            inject_windows_11(
                hwnd,
                progman,
                virtual_x,
                virtual_y,
                virtual_width,
                virtual_height,
            )?;
        } else {
            // Windows 10 and earlier use the legacy WorkerW method
            inject_legacy_workerw(
                hwnd,
                progman,
                virtual_x,
                virtual_y,
                virtual_width,
                virtual_height,
            )?;
        }
    }

    start_watchdog();
    Ok(())
}

unsafe fn inject_windows_11(
    hwnd: HWND,
    progman: HWND,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> Result<(), String> {
    // ===================================================================
    // ACHIEVMENT :3 WINDOWS 11 24H2 FIX
    // ===================================================================
    // Windows 11 introduced the "raised desktop" architecture with new compositor:
    //
    // Windows 11 hierarchy:
    //   Progman ─┐
    //            ├─ ShellDLL_DefView  // Desktop icons (interactive, visible, WS_EX_LAYERED)
    //            ├─ Our Window        // Our layered window goes here (WS_EX_LAYERED child)
    //            └─ WorkerW           // Static wallpaper (beneath everything)
    //
    // Key changes from Windows 10:
    // - Progman has WS_EX_NOREDIRECTIONBITMAP (no GDI content)
    // - DefView is a WS_EX_LAYERED child window
    // - WorkerW is z-ordered under DefView
    // - Our window must be a WS_EX_LAYERED child of Progman
    // - Window coordinates are relative to Progman, not screen!
    //
    // Solution:
    //   1. Send 0x052C to Progman to raise desktop hierarchy
    //   2. Find ShellDLL_DefView and WorkerW as children of Progman
    //   3. Parent our window to Progman (make it a child)
    //   4. Position our window below ShellDLL_DefView but above WorkerW
    //   5. Make window fully opaque with SetLayeredWindowAttributes
    //   6. Use (0, 0) coordinates relative to Progman, not virtual screen coords
    // ===================================================================

    println!(
        "[desktop_injection] [Windows11] Virtual screen: x={}, y={}, width={}, height={}",
        x, y, width, height
    );
    println!(
        "[desktop_injection] [Windows11] HWND: {:?}, Progman: {:?}",
        hwnd, progman
    );

    let _ = SendMessageTimeoutW(
        progman,
        0x052C,
        WPARAM(0),
        LPARAM(0),
        SMTO_NORMAL,
        1000,
        None,
    );
    thread::sleep(Duration::from_millis(500));

    let shell_view = FindWindowExW(
        Some(progman),
        None,
        PCWSTR(windows::core::w!("SHELLDLL_DefView").as_ptr()),
        PCWSTR::null(),
    )
    .map_err(|e| format!("[Windows11] ShellDLL_DefView not found: {}", e))?;

    let workerw = FindWindowExW(
        Some(progman),
        None,
        PCWSTR(windows::core::w!("WorkerW").as_ptr()),
        PCWSTR::null(),
    )
    .map_err(|e| format!("[Windows11] WorkerW not found as child of Progman: {}", e))?;

    *WORKERW_HANDLE.lock().unwrap() = Some(workerw.0 as isize);
    *SHELLVIEW_HANDLE.lock().unwrap() = Some(shell_view.0 as isize);

    println!(
        "[desktop_injection] [Windows11] Found ShellDLL_DefView: {:?}, WorkerW: {:?}",
        shell_view, workerw
    );

    // Get Progman's actual size to ensure our window covers the full desktop
    use windows::Win32::UI::WindowsAndMessaging::GetWindowRect;
    use windows::Win32::Foundation::RECT;
    let mut progman_rect = RECT::default();
    GetWindowRect(progman, &mut progman_rect)
        .map_err(|e| format!("GetWindowRect(Progman) failed: {}", e))?;
    
    let progman_width = progman_rect.right - progman_rect.left;
    let progman_height = progman_rect.bottom - progman_rect.top;
    
    println!(
        "[desktop_injection] [Windows11] Progman size: {}x{}",
        progman_width, progman_height
    );

    let mut style = GetWindowLongPtrW(hwnd, GWL_STYLE);
    println!(
        "[desktop_injection] [Windows11] Original window style: 0x{:X}",
        style
    );
    style &= !(WS_POPUP.0 as isize);
    style &= !(WS_DISABLED.0 as isize);
    style |= WS_CHILD.0 as isize;
    SetWindowLongPtrW(hwnd, GWL_STYLE, style);
    println!("[desktop_injection] [Windows11] New window style: 0x{:X}", style);

    SetParent(hwnd, Some(progman)).map_err(|e| format!("SetParent failed: {}", e))?;
    println!("[desktop_injection] [Windows11] SetParent to Progman: OK");

    SetLayeredWindowAttributes(hwnd, COLORREF(0), 255, LWA_ALPHA)
        .map_err(|e| format!("SetLayeredWindowAttributes failed: {}", e))?;
    println!("[desktop_injection] [Windows11] SetLayeredWindowAttributes alpha=255: OK");

    // CRITICAL: Child window coordinates are relative to parent (Progman), not screen!
    // Use (0, 0) to cover the entire parent window, not (x, y) virtual screen coords
    // Use Progman's actual size, not virtual screen size, to ensure full coverage
    // SWP_FRAMECHANGED is REQUIRED after SetWindowLongPtrW style change per MSDN!
    println!(
        "[desktop_injection] [Windows11] Calling SetWindowPos(0, 0, {}, {}) with SWP_FRAMECHANGED",
        progman_width, progman_height
    );
    SetWindowPos(
        hwnd,
        Some(shell_view),
        0,
        0,
        progman_width,
        progman_height,
        SWP_NOACTIVATE | SWP_FRAMECHANGED,
    )
    .map_err(|e| format!("SetWindowPos (below ShellView) failed: {}", e))?;

    // Verify the window rect after positioning
    let mut rect: windows::Win32::Foundation::RECT = std::mem::zeroed();
    if GetWindowRect(hwnd, &mut rect).is_ok() {
        println!("[desktop_injection] [Windows11] Window rect after SetWindowPos: left={}, top={}, right={}, bottom={} (size: {}x{})",
            rect.left, rect.top, rect.right, rect.bottom,
            rect.right - rect.left, rect.bottom - rect.top);
    }

    SetWindowPos(
        workerw,
        Some(hwnd),
        0,
        0,
        0,
        0,
        SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
    )
    .map_err(|e| format!("SetWindowPos (WorkerW behind) failed: {}", e))?;

    let _ = ShowWindow(hwnd, SW_SHOWNA);
    println!("[desktop_injection] [Windows11] ShowWindow: OK");

    Ok(())
}

unsafe fn inject_legacy_workerw(
    hwnd: HWND,
    progman: HWND,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> Result<(), String> {
    let workerw = spawn_workerw_with_retry(progman)?;
    *WORKERW_HANDLE.lock().unwrap() = Some(workerw.0 as isize);
    parent_to_workerw(hwnd, workerw, x, y, width, height)?;
    Ok(())
}

unsafe fn spawn_workerw_with_retry(progman: HWND) -> Result<HWND, String> {
    for attempt in 0..10 {
        let _ = SendMessageTimeoutW(
            progman,
            0x052C,
            WPARAM(0),
            LPARAM(0),
            SMTO_NORMAL,
            2000,
            None,
        );
        thread::sleep(Duration::from_millis(200 + (attempt * 100)));

        if let Some(workerw) = find_workerw() {
            return Ok(workerw);
        }
    }
    Err("WorkerW spawn failed after 10 attempts".into())
}

unsafe fn find_workerw() -> Option<HWND> {
    let mut result = HWND(std::ptr::null_mut());

    extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        unsafe {
            let target = lparam.0 as *mut HWND;
            let def_view = FindWindowExW(
                Some(hwnd),
                None,
                PCWSTR(windows::core::w!("SHELLDLL_DefView").as_ptr()),
                PCWSTR::null(),
            );

            if let Ok(dv) = def_view {
                if !dv.0.is_null() {
                    let workerw = FindWindowExW(
                        None,
                        Some(hwnd),
                        PCWSTR(windows::core::w!("WorkerW").as_ptr()),
                        PCWSTR::null(),
                    );

                    if let Ok(worker) = workerw {
                        if !worker.0.is_null() {
                            *target = worker;
                            return BOOL(0);
                        }
                    }
                }
            }
        }
        BOOL(1)
    }

    let _ = EnumWindows(Some(enum_proc), LPARAM(&mut result as *mut _ as isize));

    if result.0 != std::ptr::null_mut() {
        Some(result)
    } else {
        None
    }
}

unsafe fn parent_to_workerw(
    hwnd: HWND,
    workerw: HWND,
    _x: i32,
    _y: i32,
    width: i32,
    height: i32,
) -> Result<(), String> {
    let mut style = GetWindowLongPtrW(hwnd, GWL_STYLE);
    style &= !(WS_POPUP.0 as isize);
    style &= !(WS_DISABLED.0 as isize);
    style |= WS_CHILD.0 as isize;
    SetWindowLongPtrW(hwnd, GWL_STYLE, style);

    SetParent(hwnd, Some(workerw)).map_err(|e| format!("SetParent failed: {}", e))?;

    // CRITICAL: Child window coordinates are relative to parent (WorkerW), not screen!
    // Use (0, 0) to cover the entire parent window
    // SWP_FRAMECHANGED is REQUIRED after SetWindowLongPtrW style change per MSDN!
    SetWindowPos(
        hwnd,
        Some(HWND_BOTTOM),
        0,
        0,
        width,
        height,
        SWP_NOACTIVATE | SWP_FRAMECHANGED,
    )
    .map_err(|e| format!("SetWindowPos failed: {}", e))?;
    let _ = ShowWindow(hwnd, SW_SHOWNA);

    Ok(())
}

fn start_watchdog() {
    *WATCHDOG_STOP_FLAG.lock().unwrap() = false;
    let mut handle_lock = WATCHDOG_HANDLE.lock().unwrap();
    if let Some(old_handle) = handle_lock.take() {
        drop(handle_lock);
        let _ = old_handle;
    } else {
        drop(handle_lock);
    }

    let handle = thread::spawn(|| {
        let mut check_count = 0;
        let is_windows11 = is_windows_11_or_later();

        loop {
            if *WATCHDOG_STOP_FLAG.lock().unwrap() {
                break;
            }

            thread::sleep(if check_count < 12 {
                Duration::from_secs(2)
            } else {
                Duration::from_secs(5)
            });
            check_count += 1;

            let hwnd_opt = *CURRENT_HWND.lock().unwrap();
            if let Some(handle_ptr) = hwnd_opt {
                unsafe {
                    let hwnd = HWND(handle_ptr as *mut _);

                    if !is_window_valid(hwnd) {
                        break;
                    }

                    if is_windows11 {
                        verify_windows11_zorder(hwnd);
                    } else {
                        if find_workerw().is_none() {
                            let (_x, _y, width, height) = *WINDOW_BOUNDS.lock().unwrap();

                            match FindWindowW(
                                PCWSTR(windows::core::w!("Progman").as_ptr()),
                                PCWSTR(windows::core::w!("Program Manager").as_ptr()),
                            ) {
                                Ok(progman) => {
                                    if let Ok(workerw) = spawn_workerw_with_retry(progman) {
                                        *WORKERW_HANDLE.lock().unwrap() = Some(workerw.0 as isize);
                                        // Child window coordinates are relative to parent, use (0, 0)
                                        let _ =
                                            parent_to_workerw(hwnd, workerw, 0, 0, width, height);
                                    }
                                }
                                Err(_) => continue,
                            }
                        }
                    }
                }
            } else {
                break;
            }
        }
    });

    *WATCHDOG_HANDLE.lock().unwrap() = Some(handle);
}

unsafe fn verify_windows11_zorder(hwnd: HWND) {
    let progman_opt = *PROGMAN_HANDLE.lock().unwrap();
    let shellview_opt = *SHELLVIEW_HANDLE.lock().unwrap();
    let workerw_opt = *WORKERW_HANDLE.lock().unwrap();

    if let (Some(progman_ptr), Some(shellview_ptr), Some(workerw_ptr)) =
        (progman_opt, shellview_opt, workerw_opt)
    {
        let progman = HWND(progman_ptr as *mut _);
        let shellview = HWND(shellview_ptr as *mut _);
        let workerw = HWND(workerw_ptr as *mut _);

        if !is_window_valid(workerw) {
            if let Ok(new_workerw) = FindWindowExW(
                Some(progman),
                None,
                PCWSTR(windows::core::w!("WorkerW").as_ptr()),
                PCWSTR::null(),
            ) {
                *WORKERW_HANDLE.lock().unwrap() = Some(new_workerw.0 as isize);

                let _ = SetWindowPos(
                    hwnd,
                    Some(shellview),
                    0,
                    0,
                    0,
                    0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
                );

                let _ = SetWindowPos(
                    new_workerw,
                    Some(hwnd),
                    0,
                    0,
                    0,
                    0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
                );
            }
        }
    }
}

unsafe fn is_window_valid(hwnd: HWND) -> bool {
    use windows::Win32::UI::WindowsAndMessaging::IsWindow;
    IsWindow(Some(hwnd)).as_bool()
}

pub fn stop_watchdog() {
    *WATCHDOG_STOP_FLAG.lock().unwrap() = true;
    *CURRENT_HWND.lock().unwrap() = None;
    *WORKERW_HANDLE.lock().unwrap() = None;
    *SHELLVIEW_HANDLE.lock().unwrap() = None;
    *PROGMAN_HANDLE.lock().unwrap() = None;
    let mut handle_lock = WATCHDOG_HANDLE.lock().unwrap();
    if let Some(handle) = handle_lock.take() {
        drop(handle_lock);
        thread::sleep(Duration::from_millis(100));
        let _ = handle;
    }
}

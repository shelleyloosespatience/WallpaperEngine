use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use windows::core::{BOOL, PCWSTR};
use windows::Win32::Foundation::COLORREF;
use windows::Win32::Foundation::{HWND, LPARAM, RECT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumChildWindows, EnumWindows, FindWindowExW, FindWindowW, GetWindowLongPtrW, GetWindowRect,
    SendMessageTimeoutW, SetLayeredWindowAttributes, SetParent, SetWindowLongPtrW, SetWindowPos,
    ShowWindow, GWL_EXSTYLE, GWL_STYLE, HWND_BOTTOM, LWA_ALPHA, SMTO_NORMAL, SWP_NOACTIVATE,
    SWP_NOMOVE, SWP_NOSIZE, SW_SHOWNA, WS_CHILD, WS_DISABLED, WS_EX_LAYERED, WS_EX_NOACTIVATE,
    WS_EX_TOOLWINDOW, WS_POPUP,
};

lazy_static::lazy_static! {
    static ref WATCHDOG_HANDLE: Arc<Mutex<Option<thread::JoinHandle<()>>>> = Arc::new(Mutex::new(None));
    static ref WATCHDOG_STOP_FLAG: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
    static ref CURRENT_HWND: Arc<Mutex<Option<isize>>> = Arc::new(Mutex::new(None));
    static ref WINDOW_BOUNDS: Arc<Mutex<(i32, i32, i32, i32)>> = Arc::new(Mutex::new((0, 0, 1920, 1080)));
    static ref WORKERW_HANDLE: Arc<Mutex<Option<isize>>> = Arc::new(Mutex::new(None));
    static ref IS_WINDOWS_11_24H2: Arc<Mutex<Option<bool>>> = Arc::new(Mutex::new(None));
}

fn is_windows_11_24h2_or_later() -> bool {
    {
        let cached = IS_WINDOWS_11_24H2.lock().unwrap();
        if let Some(result) = *cached {
            return result;
        }
    }

    let build_number = get_windows_build_number();
    println!("[desktop_injection] Windows build number: {}", build_number);

    let is_24h2 = build_number >= 26100;
    *IS_WINDOWS_11_24H2.lock().unwrap() = Some(is_24h2);

    if is_24h2 {
        println!("[desktop_injection] Detected Windows 11 24H2 or later");
    } else {
        println!("[desktop_injection] Detected older Windows version (pre-24H2)");
    }

    is_24h2
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
                    None => {
                        println!(
                            "[desktop_injection] Warning: RtlGetVersion not found in ntdll.dll"
                        );
                    }
                }
            }
            Err(e) => {
                println!(
                    "[desktop_injection] Warning: Could not load ntdll.dll: {}",
                    e
                );
            }
        }

        0
    }
}

pub fn inject_behind_desktop(
    hwnd: HWND,
    _x: i32,
    _y: i32,
    width: i32, // Correct dimensions provided by caller
    height: i32,
) -> Result<(), String> {
    println!("[desktop_injection] Injecting window into desktop");
    stop_watchdog();
    thread::sleep(Duration::from_millis(300));

    unsafe {
        let progman = FindWindowW(
            PCWSTR(windows::core::w!("Progman").as_ptr()),
            PCWSTR(windows::core::w!("Program Manager").as_ptr()),
        )
        .map_err(|e| format!("FindWindowW failed: {}", e))?;

        println!(
            "[desktop_injection] Using provided dimensions: {}x{}",
            width, height
        );

        *CURRENT_HWND.lock().unwrap() = Some(hwnd.0 as isize);
        *WINDOW_BOUNDS.lock().unwrap() = (0, 0, width, height);

        // CRITICAL FIX: Don't modify extended styles - window already created correctly
        // NO WS_EX_LAYERED - that's incompatible with D3D11 and was removed from wmf_player
        // WS_EX_TRANSPARENT already set during window creation

        // CRITICAL FIX: Don't modify base styles - window already created as WS_CHILD | WS_DISABLED
        // Modifying styles after creation causes DWM confusion

        if is_windows_11_24h2_or_later() {
            println!("[desktop_injection] Using Windows 11 24H2+ injection method");
            inject_windows_11_24h2(hwnd, progman, 0, 0, width, height)?;
        } else {
            println!("[desktop_injection] Using legacy WorkerW injection method");
            inject_legacy_workerw(hwnd, progman, 0, 0, width, height)?;
        }
    }

    start_watchdog();
    Ok(())
}

unsafe fn inject_windows_11_24h2(
    hwnd: HWND,
    progman: HWND,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> Result<(), String> {
    // LIVELY FIX 1: Use correct SendMessage params (0xD, 0x1 instead of 0, 0)
    println!("[desktop_injection] Sending 0x052C to Progman with Lively's params");
    let _ = SendMessageTimeoutW(
        progman,
        0x052C,
        WPARAM(0xD), // Lively uses 0xD
        LPARAM(0x1), // Lively uses 0x1
        SMTO_NORMAL,
        1000,
        None,
    );
    thread::sleep(Duration::from_millis(500));

    println!("[desktop_injection] Finding SHELLDLL_DefView");
    let shell_view = FindWindowExW(
        Some(progman),
        None,
        PCWSTR(windows::core::w!("SHELLDLL_DefView").as_ptr()),
        PCWSTR::null(),
    )
    .map_err(|e| format!("DefView not found: {}", e))?;

    // LIVELY FIX 2: Convert WS_POPUP to WS_CHILD (ONLY - NO WS_DISABLED!)
    // WS_DISABLED causes DWM to crash when interacting with desktop
    println!("[desktop_injection] Converting WS_POPUP to WS_CHILD (no WS_DISABLED)");
    let mut style = GetWindowLongPtrW(hwnd, GWL_STYLE);
    style &= !(WS_POPUP.0 as isize);
    style &= !(WS_DISABLED.0 as isize); // Explicitly remove WS_DISABLED
    style |= WS_CHILD.0 as isize; // Add ONLY WS_CHILD
    SetWindowLongPtrW(hwnd, GWL_STYLE, style);

    // LIVELY FIX 3: Add WS_EX_LAYERED and call SetLayeredWindowAttributes
    // This MUST be done BEFORE SetParent for Windows 11 24H2
    println!("[desktop_injection] Adding WS_EX_LAYERED with alpha 255 (Lively's approach)");
    let mut ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
    ex_style |= WS_EX_LAYERED.0 as isize;
    SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex_style);

    // THIS IS THE CRITICAL CALL - without it, the layered window is broken
    SetLayeredWindowAttributes(hwnd, COLORREF(0), 255, LWA_ALPHA)
        .map_err(|e| format!("SetLayeredWindowAttributes failed: {}", e))?;

    println!("[desktop_injection] Parenting to Progman");
    SetParent(hwnd, Some(progman)).map_err(|e| format!("SetParent failed: {}", e))?;

    // LIVELY FIX 4: Position relative to DefView (not HWND_BOTTOM)
    // Use SWP_NOMOVE | SWP_NOSIZE to set Z-order only, then position separately
    println!("[desktop_injection] Positioning below DefView (Lively's Z-order)");
    let window_flags = SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE;

    SetWindowPos(hwnd, Some(shell_view), 0, 0, 0, 0, window_flags)
        .map_err(|e| format!("SetWindowPos Z-order failed: {}", e))?;

    // Now set actual position and size
    println!(
        "[desktop_injection] Setting position ({}, {}) and size {}x{}",
        x, y, width, height
    );
    SetWindowPos(hwnd, None, x, y, width, height, SWP_NOACTIVATE)
        .map_err(|e| format!("SetWindowPos position failed: {}", e))?;

    // LIVELY FIX 5: Ensure WorkerW stays at bottom
    println!("[desktop_injection] Ensuring WorkerW Z-order");
    ensure_workerw_zorder(progman)?;

    let _ = ShowWindow(hwnd, SW_SHOWNA);

    // CRITICAL: DO NOT call InvalidateRect or refresh desktop on Windows 11 24H2
    // Lively explicitly skips this because it destroys WorkerW
    println!("[desktop_injection] Skipping desktop refresh (would destroy WorkerW on 24H2)");

    println!("[desktop_injection] Windows 11 24H2 injection completed");
    Ok(())
}

// Helper function to ensure WorkerW stays at bottom (Lively's approach)
unsafe fn ensure_workerw_zorder(progman: HWND) -> Result<(), String> {
    // Find WorkerW as child of Progman (24H2 structure)
    let workerw = FindWindowExW(
        Some(progman),
        None,
        PCWSTR(windows::core::w!("WorkerW").as_ptr()),
        PCWSTR::null(),
    );

    if let Ok(workerw) = workerw {
        // Get last child of Progman
        let last_child = get_last_child_window(progman);

        if last_child != workerw.0 as isize {
            println!("[desktop_injection] WorkerW not at bottom, fixing Z-order");
            let window_flags = SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE;
            SetWindowPos(workerw, Some(HWND_BOTTOM), 0, 0, 0, 0, window_flags)
                .map_err(|e| format!("Failed to fix WorkerW Z-order: {}", e))?;
        }
    }

    Ok(())
}

unsafe fn get_last_child_window(parent: HWND) -> isize {
    let mut last_child: isize = 0;

    extern "system" fn enum_child_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        unsafe {
            let last_ptr = lparam.0 as *mut isize;
            *last_ptr = hwnd.0 as isize;
        }
        BOOL(1)
    }

    let _ = EnumChildWindows(
        Some(parent),
        Some(enum_child_proc),
        LPARAM(&mut last_child as *mut _ as isize),
    );

    last_child
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
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> Result<(), String> {
    // Convert WS_POPUP to WS_CHILD (same as Windows 11 24H2 approach)
    // NO WS_DISABLED - it causes input issues
    println!("[desktop_injection] Converting WS_POPUP to WS_CHILD (no WS_DISABLED)");
    let mut style = GetWindowLongPtrW(hwnd, GWL_STYLE);
    style &= !(WS_POPUP.0 as isize);
    style &= !(WS_DISABLED.0 as isize); // Explicitly remove WS_DISABLED
    style |= WS_CHILD.0 as isize;
    SetWindowLongPtrW(hwnd, GWL_STYLE, style);

    SetParent(hwnd, Some(workerw)).map_err(|e| format!("SetParent failed: {}", e))?;

    SetWindowPos(hwnd, Some(HWND_BOTTOM), x, y, width, height, SWP_NOACTIVATE)
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
        let is_24h2 = is_windows_11_24h2_or_later();

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

                    if !is_24h2 {
                        if find_workerw().is_none() {
                            let (x, y, width, height) = *WINDOW_BOUNDS.lock().unwrap();

                            match FindWindowW(
                                PCWSTR(windows::core::w!("Progman").as_ptr()),
                                PCWSTR(windows::core::w!("Program Manager").as_ptr()),
                            ) {
                                Ok(progman) => {
                                    if let Ok(workerw) = spawn_workerw_with_retry(progman) {
                                        *WORKERW_HANDLE.lock().unwrap() = Some(workerw.0 as isize);
                                        let _ =
                                            parent_to_workerw(hwnd, workerw, x, y, width, height);
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

unsafe fn is_window_valid(hwnd: HWND) -> bool {
    use windows::Win32::UI::WindowsAndMessaging::IsWindow;
    IsWindow(Some(hwnd)).as_bool()
}

pub fn stop_watchdog() {
    *WATCHDOG_STOP_FLAG.lock().unwrap() = true;
    *CURRENT_HWND.lock().unwrap() = None;
    *WORKERW_HANDLE.lock().unwrap() = None;
    let mut handle_lock = WATCHDOG_HANDLE.lock().unwrap();
    if let Some(handle) = handle_lock.take() {
        drop(handle_lock);
        thread::sleep(Duration::from_millis(100));
        let _ = handle;
    }
}

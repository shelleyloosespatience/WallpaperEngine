use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use windows::core::{BOOL, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, FindWindowExW, FindWindowW, GetWindowLongPtrW, SendMessageTimeoutW, SetParent,
    SetWindowLongPtrW, SetWindowPos, ShowWindow, GWL_EXSTYLE, GWL_STYLE, HWND_BOTTOM, SMTO_NORMAL,
    SWP_NOACTIVATE, SW_SHOWNA, WS_CHILD, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
    WS_EX_TRANSPARENT, WS_POPUP,
};

lazy_static::lazy_static! {
    static ref WATCHDOG_HANDLE: Arc<Mutex<Option<thread::JoinHandle<()>>>> = Arc::new(Mutex::new(None));
    static ref WATCHDOG_STOP_FLAG: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
    static ref CURRENT_HWND: Arc<Mutex<Option<isize>>> = Arc::new(Mutex::new(None));
    static ref WINDOW_BOUNDS: Arc<Mutex<(i32, i32, i32, i32)>> = Arc::new(Mutex::new((0, 0, 1920, 1080)));
    static ref WORKERW_HANDLE: Arc<Mutex<Option<isize>>> = Arc::new(Mutex::new(None));
    static ref IS_WINDOWS_11_24H2: Arc<Mutex<Option<bool>>> = Arc::new(Mutex::new(None));
}

/// checks if running on Windows 11 24H2 or later (build >= 26100)
fn is_windows_11_24h2_or_later() -> bool {
    // Check cache first
    {
        let cached = IS_WINDOWS_11_24H2.lock().unwrap();
        if let Some(result) = *cached {
            return result;
        }
    }

    // Detect Windows version using build number
    let build_number = get_windows_build_number();
    println!("[desktop_injection] Windows build number: {}", build_number);

    // Windows 11 24H2 = build 26100
    // Windows 11 starts at build 22000
    let is_24h2 = build_number >= 26100;

    // Cache the result
    *IS_WINDOWS_11_24H2.lock().unwrap() = Some(is_24h2);

    if is_24h2 {
        println!("[desktop_injection] Detected Windows 11 24H2 or later");
    } else {
        println!("[desktop_injection] Detected older Windows version (pre-24H2)");
    }

    is_24h2
}

/// Get Windows build number using RtlGetVersion
fn get_windows_build_number() -> u32 {
    unsafe {
        use windows::Win32::System::SystemInformation::OSVERSIONINFOEXW;

        let mut version_info: OSVERSIONINFOEXW = std::mem::zeroed();
        version_info.dwOSVersionInfoSize = std::mem::size_of::<OSVERSIONINFOEXW>() as u32;

        // RtlGetVersion function signature
        type RtlGetVersion = unsafe extern "system" fn(*mut OSVERSIONINFOEXW) -> i32;

        // Load ntdll.dll and get RtlGetVersion
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
                            // Success
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

        // otherwise assume older Windows
        println!("[desktop_injection] Warning: Could not detect build number, assuming pre-24H2");
        0
    }
}

pub fn inject_behind_desktop(
    hwnd: HWND,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> Result<(), String> {
    println!("[desktop_injection] Injecting window into desktop");
    stop_watchdog();
    thread::sleep(Duration::from_millis(300));

    *CURRENT_HWND.lock().unwrap() = Some(hwnd.0 as isize);
    *WINDOW_BOUNDS.lock().unwrap() = (x, y, width, height);

    unsafe {
        // extended styles (common for both methods)
        let mut ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        ex_style |=
            (WS_EX_TRANSPARENT.0 | WS_EX_LAYERED.0 | WS_EX_TOOLWINDOW.0 | WS_EX_NOACTIVATE.0)
                as isize;
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex_style);

        // find Progman
        let progman = FindWindowW(
            PCWSTR(windows::core::w!("Progman").as_ptr()),
            PCWSTR(windows::core::w!("Program Manager").as_ptr()),
        )
        .map_err(|e| format!("FindWindowW failed: {}", e))?;

        // choose injection method based on Windows version
        if is_windows_11_24h2_or_later() {
            println!("[desktop_injection] Using Windows 11 24H2+ injection method");
            inject_windows_11_24h2(hwnd, progman, x, y, width, height)?;
        } else {
            println!("[desktop_injection] Using legacy WorkerW injection method");
            inject_legacy_workerw(hwnd, progman, x, y, width, height)?;
        }
    }

    start_watchdog();
    Ok(())
}

/// windows 11 24H2+ injection method: parent to Progman and use Z-ordering
unsafe fn inject_windows_11_24h2(
    hwnd: HWND,
    progman: HWND,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> Result<(), String> {
    // first we try to double-send 0x052C for 24H2 (CRITICAL - as per user's suggestion)
    println!("[desktop_injection] Sending 0x052C to Progman (first attempt)");
    let _ = SendMessageTimeoutW(
        progman,
        0x052C,
        WPARAM(0),
        LPARAM(0),
        SMTO_NORMAL,
        2000,
        None,
    );
    thread::sleep(Duration::from_millis(1000));

    println!("[desktop_injection] Sending 0x052C to Progman (second attempt)");
    let _ = SendMessageTimeoutW(
        progman,
        0x052C,
        WPARAM(0),
        LPARAM(0),
        SMTO_NORMAL,
        2000,
        None,
    );
    thread::sleep(Duration::from_millis(500));

    // then we try to find SHELLDLL_DefView (icon window)
    println!("[desktop_injection] Finding SHELLDLL_DefView");
    let shell_view = FindWindowExW(
        Some(progman),
        None,
        PCWSTR(windows::core::w!("SHELLDLL_DefView").as_ptr()),
        PCWSTR::null(),
    )
    .map_err(|e| format!("DefView not found: {}", e))?;

    println!(
        "[desktop_injection] Found SHELLDLL_DefView: {:?}",
        shell_view
    );

    // then we try to set as child window
    println!("[desktop_injection] Setting window style to WS_CHILD");
    let mut style = GetWindowLongPtrW(hwnd, GWL_STYLE);
    style &= !(WS_POPUP.0 as isize);
    style |= WS_CHILD.0 as isize;
    SetWindowLongPtrW(hwnd, GWL_STYLE, style);

    // then we try to parent to Progman (NOT WorkerW !)
    println!("[desktop_injection] Parenting to Progman");
    SetParent(hwnd, Some(progman)).map_err(|e| format!("SetParent failed: {}", e))?;

    // then we try to position below shell_view (icons on top, video below)
    println!("[desktop_injection] Positioning below SHELLDLL_DefView");
    SetWindowPos(hwnd, Some(shell_view), x, y, width, height, SWP_NOACTIVATE)
        .map_err(|e| format!("SetWindowPos failed: {}", e))?;

    let _ = ShowWindow(hwnd, SW_SHOWNA);

    println!("[desktop_injection] Windows 11 24H2 injection completed successfully");
    Ok(())
}

/// anddd ourr WorkerW injection method for older Windows versions (win 10 lol not vista)
unsafe fn inject_legacy_workerw(
    hwnd: HWND,
    progman: HWND,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> Result<(), String> {
    // then spawn uh the WorkerW using the normal method
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
    Err("WorkerW spawn failed after all attempts, please open an issue on GitHub, go to settings for the redirect".into())
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
    let mut style = GetWindowLongPtrW(hwnd, GWL_STYLE);
    style &= !(WS_POPUP.0 as isize);
    style |= WS_CHILD.0 as isize;
    SetWindowLongPtrW(hwnd, GWL_STYLE, style);

    let parent_result =
        SetParent(hwnd, Some(workerw)).map_err(|e| format!("SetParent failed: {}", e))?;
    if parent_result.0.is_null() {
        return Err("SetParent returned null".into());
    }

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
        println!("[watchdog] Starting (monitors desktop injection)");

        let mut check_count = 0;
        let is_24h2 = is_windows_11_24h2_or_later();

        loop {
            if *WATCHDOG_STOP_FLAG.lock().unwrap() {
                println!("[watchdog] Stop flag set, exiting");
                break;
            }

            let sleep_duration = if check_count < 12 {
                Duration::from_secs(2)
            } else {
                Duration::from_secs(5)
            };

            thread::sleep(sleep_duration);
            check_count += 1;

            let hwnd_opt = *CURRENT_HWND.lock().unwrap();
            if let Some(handle_ptr) = hwnd_opt {
                unsafe {
                    let hwnd = HWND(handle_ptr as *mut _);

                    if !is_window_valid(hwnd) {
                        println!("[watchdog] Window invalid, stopping");
                        break;
                    }

                    // diff watchdog behavior for diff Windows versions
                    if is_24h2 {
                        // On 24H2, we don't need to check for WorkerW
                        // Just verify the window is still visible
                        // (Z-order maintenance could be added here later on, idh time rn)
                    } else {
                        // older Window check if workerw still exists
                        if find_workerw().is_none() {
                            println!("[watchdog] WorkerW missing — re-injecting");
                            let (x, y, width, height) = *WINDOW_BOUNDS.lock().unwrap();

                            match FindWindowW(
                                PCWSTR(windows::core::w!("Progman").as_ptr()),
                                PCWSTR(windows::core::w!("Program Manager").as_ptr()),
                            ) {
                                Ok(progman) => {
                                    if let Ok(workerw) = spawn_workerw_with_retry(progman) {
                                        *WORKERW_HANDLE.lock().unwrap() = Some(workerw.0 as isize);
                                        match parent_to_workerw(hwnd, workerw, x, y, width, height)
                                        {
                                            Ok(_) => {
                                                println!(
                                                    "[watchdog] Re-injected wallpaper successfully"
                                                )
                                            }
                                            Err(e) => {
                                                println!("[watchdog] Failed to re-inject: {}", e)
                                            }
                                        }
                                    } else {
                                        println!("[watchdog] Failed to spawn WorkerW");
                                    }
                                }
                                Err(e) => {
                                    println!("[watchdog] Progman not found: {}", e);
                                    continue;
                                }
                            }
                        }
                    }
                }
            } else {
                println!("[watchdog] No active window, stopping");
                break;
            }
        }

        println!("[watchdog] Stopped");
    });

    *WATCHDOG_HANDLE.lock().unwrap() = Some(handle);
}

unsafe fn is_window_valid(hwnd: HWND) -> bool {
    use windows::Win32::UI::WindowsAndMessaging::IsWindow;
    IsWindow(Some(hwnd)).as_bool()
}

pub fn stop_watchdog() {
    println!("[watchdog] Stop requested");
    *WATCHDOG_STOP_FLAG.lock().unwrap() = true;
    *CURRENT_HWND.lock().unwrap() = None;
    *WORKERW_HANDLE.lock().unwrap() = None;
    let mut handle_lock = WATCHDOG_HANDLE.lock().unwrap();
    if let Some(handle) = handle_lock.take() {
        drop(handle_lock);
        thread::sleep(Duration::from_millis(100));
        thread::sleep(Duration::from_millis(100));
        let _ = handle;
    }
}

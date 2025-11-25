use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use windows::core::{BOOL, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, FindWindowExW, FindWindowW, GetWindowLongPtrW, SendMessageTimeoutW, SetParent,
    SetWindowLongPtrW, SetWindowPos, ShowWindow, GWL_EXSTYLE, GWL_STYLE, HWND_BOTTOM, SMTO_NORMAL,
    SWP_NOACTIVATE, SW_SHOWNA, WS_CHILD, WS_EX_LAYERED, WS_EX_NOACTIVATE,
    WS_EX_TOOLWINDOW, WS_EX_TRANSPARENT, WS_POPUP,
};
lazy_static::lazy_static! {
    static ref WATCHDOG_HANDLE: Arc<Mutex<Option<thread::JoinHandle<()>>>> = Arc::new(Mutex::new(None));
    static ref WATCHDOG_STOP_FLAG: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
    static ref CURRENT_HWND: Arc<Mutex<Option<isize>>> = Arc::new(Mutex::new(None));
    static ref WINDOW_BOUNDS: Arc<Mutex<(i32, i32, i32, i32)>> = Arc::new(Mutex::new((0, 0, 1920, 1080)));
    static ref WORKERW_HANDLE: Arc<Mutex<Option<isize>>> = Arc::new(Mutex::new(None));
}
pub fn inject_behind_desktop(
    hwnd: HWND,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> Result<(), String> {
    println!("injecting window into desktop");
    // stop existing watchdog
    stop_watchdog();
    thread::sleep(Duration::from_millis(300));

    *CURRENT_HWND.lock().unwrap() = Some(hwnd.0 as isize);
    *WINDOW_BOUNDS.lock().unwrap() = (x, y, width, height);
    unsafe {
        // window styles
        let mut ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        ex_style |=
            (WS_EX_TRANSPARENT.0 | WS_EX_LAYERED.0 | WS_EX_TOOLWINDOW.0 | WS_EX_NOACTIVATE.0)
                as isize;
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex_style);

        let progman = FindWindowW(
            PCWSTR(windows::core::w!("Progman").as_ptr()),
            PCWSTR(windows::core::w!("Program Manager").as_ptr()),
        )
        .map_err(|e| format!("FindWindowW failed: {}", e))?;

        let workerw = spawn_workerw_with_retry(progman)?;

        // save the WorkerW handle
        *WORKERW_HANDLE.lock().unwrap() = Some(workerw.0 as isize);

        parent_to_workerw(hwnd, workerw, x, y, width, height)?;
    }
    start_watchdog();
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
            println!("ok: workerw found on attempt {}", attempt + 1);
            return Ok(workerw);
        }
    }
    Err("workerw spawn failed after 10 attempts".into())
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
    // Switch to child style for reparenting
    let mut style = GetWindowLongPtrW(hwnd, GWL_STYLE);
    style &= !(WS_POPUP.0 as isize);
    style |= WS_CHILD.0 as isize;
    SetWindowLongPtrW(hwnd, GWL_STYLE, style);

    let parent_result =
        SetParent(hwnd, Some(workerw)).map_err(|e| format!("setparent failed: {}", e))?;
    if parent_result.0.is_null() {
        return Err("setparent returned null".into());
    }

    // Position window first WITHOUT showing it
    SetWindowPos(hwnd, Some(HWND_BOTTOM), x, y, width, height, SWP_NOACTIVATE)
        .map_err(|e| format!("setwindowpos failed: {}", e))?;

    // NOW show the window after itss properly positioned
    // This prevents it from flashing in taskbar
    let _ = ShowWindow(hwnd, SW_SHOWNA);

    Ok(())
}
fn start_watchdog() {
    // Set stop flag to false
    *WATCHDOG_STOP_FLAG.lock().unwrap() = false;

    // Clean up any previous watchdog thread
    let mut handle_lock = WATCHDOG_HANDLE.lock().unwrap();
    if let Some(old_handle) = handle_lock.take() {
        drop(handle_lock);
        // Dont wait for old thread, let it die naturally
        let _ = old_handle;
    } else {
        drop(handle_lock);
    }

    let handle = thread::spawn(|| {
        println!("[watchdog] starting (monitors workerw/progman)");

        let mut check_count = 0;

        loop {
            // check stop flag
            if *WATCHDOG_STOP_FLAG.lock().unwrap() {
                println!("[watchdog] stop flag set, exiting");
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
                        println!("[watchdog] window invalid, stopping");
                        break;
                    }

                    if find_workerw().is_none() {
                        println!("[watchdog] workerw missing — re-injecting");
                        let (x, y, width, height) = *WINDOW_BOUNDS.lock().unwrap();

                        match FindWindowW(
                            PCWSTR(windows::core::w!("Progman").as_ptr()),
                            PCWSTR(windows::core::w!("Program Manager").as_ptr()),
                        ) {
                            Ok(progman) => {
                                if let Ok(workerw) = spawn_workerw_with_retry(progman) {
                                    *WORKERW_HANDLE.lock().unwrap() = Some(workerw.0 as isize);
                                    match parent_to_workerw(hwnd, workerw, x, y, width, height) {
                                        Ok(_) => {
                                            println!(
                                                "[watchdog] re-injected wallpaper successfully"
                                            )
                                        }
                                        Err(e) => println!("[watchdog] failed to re-inject: {}", e),
                                    }
                                } else {
                                    println!("[watchdog] failed spawn workerw");
                                }
                            }
                            Err(e) => {
                                println!("[watchdog] progman not found: {}", e);
                                continue;
                            }
                        }
                    }
                }
            } else {
                println!("[watchdog] no active window, stopping");
                break;
            }
        }

        println!("[watchdog] stopped");
    });
    *WATCHDOG_HANDLE.lock().unwrap() = Some(handle);
}

unsafe fn is_window_valid(hwnd: HWND) -> bool {
    use windows::Win32::UI::WindowsAndMessaging::IsWindow;
    IsWindow(Some(hwnd)).as_bool()
}

pub fn stop_watchdog() {
    println!("[watchdog] stop requested");

    // stop flag
    *WATCHDOG_STOP_FLAG.lock().unwrap() = true;

    // Clear Current State->
    *CURRENT_HWND.lock().unwrap() = None;
    *WORKERW_HANDLE.lock().unwrap() = None;

    // Wait for watchdog thread to finish
    let mut handle_lock = WATCHDOG_HANDLE.lock().unwrap();
    if let Some(handle) = handle_lock.take() {
        drop(handle_lock);

        // Give it some time to finish gracefully
        thread::sleep(Duration::from_millis(100));

        // IFit doesn't finish, that's okay, it will check the flag soon
        let _ = handle;
    }
}

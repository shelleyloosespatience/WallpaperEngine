use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use windows::core::{BOOL, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, FindWindowExW, FindWindowW, GetWindowLongPtrW, SendMessageTimeoutW,
    SetParent, SetWindowLongPtrW, SetWindowPos, ShowWindow, GWL_EXSTYLE, GWL_STYLE, HWND_BOTTOM,
    SMTO_NORMAL, SWP_NOACTIVATE, SWP_SHOWWINDOW, SW_SHOWNA, WS_CHILD, WS_EX_LAYERED,
    WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TRANSPARENT, WS_POPUP,
};

#[derive(Clone, Copy, Debug)]
struct SharedHwnd(HWND);
unsafe impl Send for SharedHwnd {}
unsafe impl Sync for SharedHwnd {}

lazy_static::lazy_static! {
    static ref WATCHDOG_RUNNING: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
    static ref CURRENT_HWND: Arc<Mutex<Option<SharedHwnd>>> = Arc::new(Mutex::new(None));
    static ref WINDOW_BOUNDS: Arc<Mutex<(i32, i32, i32, i32)>> = Arc::new(Mutex::new((0, 0, 1920, 1080)));
}

pub fn inject_behind_desktop(hwnd: HWND, x: i32, y: i32, width: i32, height: i32) -> Result<(), String> {
    *CURRENT_HWND.lock().unwrap() = Some(SharedHwnd(hwnd));
    *WINDOW_BOUNDS.lock().unwrap() = (x, y, width, height);

    unsafe {
        // set exstyle: transparent + layered + toolwindow + noactivate so window doesn't take input/alt-tab
        // exstyle-warning: may still interact with some shell behaviors on windows updates :)
        let mut ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        ex_style |= (WS_EX_TRANSPARENT.0 | WS_EX_LAYERED.0 | WS_EX_TOOLWINDOW.0 | WS_EX_NOACTIVATE.0) as isize;
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex_style);

        let progman = FindWindowW(
            PCWSTR(windows::core::w!("Progman").as_ptr()),
            PCWSTR(windows::core::w!("Program Manager").as_ptr()),
        )
        .map_err(|e| format!("FindWindowW failed: {}", e))?;

        let workerw = spawn_workerw_with_retry(progman)?;
        parent_to_workerw(hwnd, workerw, x, y, width, height)?;
    }

    start_watchdog();
    Ok(())
}

unsafe fn spawn_workerw_with_retry(progman: HWND) -> Result<HWND, String> {
    for attempt in 0..10 {
        let _ = SendMessageTimeoutW(progman, 0x052C, WPARAM(0), LPARAM(0), SMTO_NORMAL, 2000, None);
        thread::sleep(Duration::from_millis(200 + (attempt * 100)));

        if let Some(workerw) = find_workerw() {
            println!("ok: workerw found on attempt {}", attempt + 1); // minimal dev log
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

unsafe fn parent_to_workerw(hwnd: HWND, workerw: HWND, x: i32, y: i32, width: i32, height: i32) -> Result<(), String> {
    // switch to child style so reparenting to workerw is valid
    // reparent-critical
    let mut style = GetWindowLongPtrW(hwnd, GWL_STYLE);
    style &= !(WS_POPUP.0 as isize);
    style |= WS_CHILD.0 as isize;
    SetWindowLongPtrW(hwnd, GWL_STYLE, style);

    let parent_result = SetParent(hwnd, Some(workerw))
        .map_err(|e| format!("setparent failed: {}", e))?;
    if parent_result.0.is_null() {
        return Err("setparent returned null".into());
    }

    // ensure bottom z-order: keeps window behind desktop icons and reduces focus stealing
    // z-order
    SetWindowPos(
        hwnd,
        Some(HWND_BOTTOM),
        x,
        y,
        width,
        height,
        SWP_NOACTIVATE | SWP_SHOWWINDOW,
    )
    .map_err(|e| format!("setwindowpos failed: {}", e))?;

    // show no activate: do not steal focus; layered/transparent flags still apply
    // focus-safe
    let _ = ShowWindow(hwnd, SW_SHOWNA);

    Ok(())
}

fn start_watchdog() {
    let mut running = WATCHDOG_RUNNING.lock().unwrap();
    if *running {
        return;
    }
    *running = true;
    drop(running);

    thread::spawn(|| {
        println!("watchdog: starting (monitors workerw/progman)");

        let mut check_count = 0;
        
        loop {
            let sleep_duration = if check_count < 12 {
                Duration::from_secs(2) // Check every 2 seconds for first minute
            } else {
                Duration::from_secs(5) // Then every 5 seconds
            };
            
            thread::sleep(sleep_duration);
            check_count += 1;

            let hwnd_opt = *CURRENT_HWND.lock().unwrap();
            if let Some(handle) = hwnd_opt {
                unsafe {
                    let hwnd = handle.0;

                    if !is_window_valid(hwnd) {
                        println!("watchdog: window invalid, stopping"); // dev log
                        break;
                    }

                    if find_workerw().is_none() {
                        println!("watchdog: workerw missing — re-injecting"); // dev log
                        let (x, y, width, height) = *WINDOW_BOUNDS.lock().unwrap();

                        match FindWindowW(
                            PCWSTR(windows::core::w!("Progman").as_ptr()),
                            PCWSTR(windows::core::w!("Program Manager").as_ptr()),
                        ) {
                            Ok(progman) => {
                                if let Ok(workerw) = spawn_workerw_with_retry(progman) {
                                    match parent_to_workerw(hwnd, workerw, x, y, width, height) {
                                        Ok(_) => println!("watchdog: re-injected wallpaper successfully"), // dev log
                                        Err(e) => println!("watchdog: failed to re-inject: {}", e),
                                    }
                                } else {
                                    println!("watchdog: failed spawn workerw");
                                }
                            }
                            Err(e) => {
                                println!("watchdog: progman not found: {}", e);
                                continue;
                            }
                        }
                    }
                }
            } else {
                println!("watchdog: no active window, stopping");
                break;
            }
        }

        *WATCHDOG_RUNNING.lock().unwrap() = false;
        println!("watchdog: stopped");
    });
}

unsafe fn is_window_valid(hwnd: HWND) -> bool {
    use windows::Win32::UI::WindowsAndMessaging::IsWindow;
    IsWindow(Some(hwnd)).as_bool()
}

pub fn stop_watchdog() {
    println!("watchdog: stop requested");
    *CURRENT_HWND.lock().unwrap() = None;
    *WATCHDOG_RUNNING.lock().unwrap() = false;
}
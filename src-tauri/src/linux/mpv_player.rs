// this is a working mpv implementation
// planned to be added in future versions where we support multiple players and for linux
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use windows::Win32::Foundation::HWND;

pub struct MpvPlayer {
    process: Child,
    hwnd: HWND,
    ipc_pipe_name: String,
}

// safe because we only access from main thread
unsafe impl Send for MpvPlayer {}
unsafe impl Sync for MpvPlayer {}

impl MpvPlayer {
    pub fn new(hwnd: HWND, video_path: &str, width: i32, height: i32) -> Result<Self, String> {
        println!("[mpv_player] Starting MPV process");

        // generate unique pipe name (Windows named pipe format)
        let ipc_pipe_name = format!("\\\\.\\pipe\\mpvsocket_{}", std::process::id());

        // Convert HWND to i64 for MPV's --wid parameter
        let wid = hwnd.0 as i64;

        // bld MPV command like Lively does
        let mut cmd = Command::new("mpv.exe");
        cmd.arg("--volume=0")
            .arg("--loop-file")
            .arg("--keep-open")
            .arg("--no-window-dragging")
            .arg("--cursor-autohide=no")
            .arg("--stop-screensaver=no")
            .arg("--input-default-bindings=no")
            .arg("--no-border")
            .arg("--input-cursor=no")
            .arg("--no-osc")
            .arg("--hwdec=auto-safe")
            .arg(format!("--wid={}", wid)) // CRITICAL: Render into our window!
            .arg(format!("--input-ipc-server={}", ipc_pipe_name))
            .arg(video_path)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Launch mpv.exe
        let process = cmd
            .spawn()
            .map_err(|e| format!("Failed to spawn MPV: {}", e))?;

        println!(
            "[mpv_player] MPV process started with PID: {} rendering to HWND: {}",
            process.id(),
            wid
        );

        Ok(Self {
            process,
            hwnd,
            ipc_pipe_name,
        })
    }

    pub fn send_command(&self, command: &str) -> Result<(), String> {
        use std::fs::OpenOptions;
        use std::io::Write;

        //named pipe and write command
        match OpenOptions::new().write(true).open(&self.ipc_pipe_name) {
            Ok(mut pipe) => {
                pipe.write_all(command.as_bytes())
                    .map_err(|e| format!("Failed to write to pipe: {}", e))?;
                println!("[mpv_player] Sent command: {}", command.trim());
                Ok(())
            }
            Err(e) => {
                // no ready that's OK for first few commands
                println!("[mpv_player] Pipe not ready: {}", e);
                Ok(())
            }
        }
    }

    pub fn play(&self) -> Result<(), String> {
        self.send_command("{\"command\":[\"set_property\",\"pause\",false]}\n")
    }

    pub fn pause(&self) -> Result<(), String> {
        self.send_command("{\"command\":[\"set_property\",\"pause\",true]}\n")
    }

    pub fn stop(&self) -> Result<(), String> {
        self.send_command("{\"command\":[\"quit\"]}\n")
    }

    pub fn shutdown(&mut self) {
        println!("[mpv_player] Shutting down MPV");
        let _ = self.stop();
        std::thread::sleep(std::time::Duration::from_millis(100));
        let _ = self.process.kill();
    }

    pub fn hwnd(&self) -> HWND {
        self.hwnd
    }
}

impl Drop for MpvPlayer {
    fn drop(&mut self) {
        self.shutdown();
    }
}

// Just create the window - MPV will render to it
pub unsafe fn create_mpv_window(width: i32, height: i32) -> Result<HWND, String> {
    use windows::core::w;
    use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
    use windows::Win32::UI::WindowsAndMessaging::*;

    let class_name = w!("MpvPlayerWindow");

    let wc = WNDCLASSW {
        lpfnWndProc: Some(std::mem::transmute(DefWindowProcW as *const ())),
        hInstance: windows::Win32::Foundation::HINSTANCE(std::ptr::null_mut()),
        lpszClassName: windows::core::PCWSTR(class_name.as_ptr()),
        style: CS_HREDRAW | CS_VREDRAW,
        hbrBackground: windows::Win32::Graphics::Gdi::HBRUSH(0 as _),
        ..Default::default()
    };

    let _ = RegisterClassW(&wc);

    use crate::os_version::get_windows_version;
    let win_ver = get_windows_version();

    let ex_style = if win_ver.is_windows_11_24h2_plus() {
        println!("[mpv_player] Using Windows 11 24H2+ window style");
        WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_NOPARENTNOTIFY
    } else {
        println!("[mpv_player] Using Windows 10/11 window style");
        WS_EX_TRANSPARENT | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_NOPARENTNOTIFY
    };

    let hwnd = CreateWindowExW(
        ex_style,
        class_name,
        w!("MPV Video Player"),
        WS_POPUP,
        0,
        0,
        width,
        height,
        None,
        None,
        None,
        None,
    )
    .map_err(|e| format!("CreateWindowExW failed: {}", e))?;

    if win_ver.is_windows_11_24h2_plus() {
        use windows::Win32::Foundation::COLORREF;
        use windows::Win32::UI::WindowsAndMessaging::{SetLayeredWindowAttributes, LWA_ALPHA};
        SetLayeredWindowAttributes(hwnd, COLORREF(0), 255, LWA_ALPHA)
            .map_err(|e| format!("SetLayeredWindowAttributes failed: {}", e))?;
    }

    let _ = SetWindowPos(
        hwnd,
        Some(HWND_BOTTOM),
        0,
        0,
        0,
        0,
        SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE | SWP_HIDEWINDOW,
    );

    Ok(hwnd)
}

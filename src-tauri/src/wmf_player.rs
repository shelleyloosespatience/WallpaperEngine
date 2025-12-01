use windows::core::{implement, w, BSTR, PCWSTR};
use windows::Win32::Foundation::{COLORREF, HINSTANCE, HMODULE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Direct3D::D3D_DRIVER_TYPE_HARDWARE;
use windows::Win32::Graphics::Direct3D11::*;
use windows::Win32::Graphics::Dxgi::Common::*;
use windows::Win32::Graphics::Gdi::{BeginPaint, EndPaint, PAINTSTRUCT};
use windows::Win32::Media::MediaFoundation::*;
use windows::Win32::System::Com::*;
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::os_version::get_windows_version;

type WmfResult<T> = std::result::Result<T, String>;

pub struct WmfPlayer {
    hwnd: HWND,
    media_engine: Option<IMFMediaEngine>,
    _d3d_device: Option<ID3D11Device>,
    _callback: Option<IMFMediaEngineNotify>,
    com_initialized: bool,
    mf_initialized: bool,
}

unsafe impl Send for WmfPlayer {}
unsafe impl Sync for WmfPlayer {}

impl WmfPlayer {
    pub fn new(width: i32, height: i32) -> WmfResult<Self> {
        unsafe {
            println!("[wmf: crit]  Initializing player");

            let hr = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
            let com_initialized = if hr.is_ok() || hr.0 == 0x00000001 {
                true
            } else {
                println!("[wmf] warning:  COM already initialized: {:?}", hr);
                false
            };

            let mf_result = MFStartup(MF_VERSION, MFSTARTUP_FULL);
            let mf_initialized = if mf_result.is_ok() {
                true
            } else {
                println!("[wmf] warning: MF startup failed: {:?}", mf_result);
                false
            };

            let hwnd = create_player_window(width, height)?;
            let d3d_device = create_optimized_d3d_device()?;
            let (media_engine, callback) = create_optimized_media_engine(hwnd, &d3d_device)?;

            println!("[wmf] Optimized player created");

            Ok(Self {
                hwnd,
                media_engine: Some(media_engine),
                _d3d_device: Some(d3d_device),
                _callback: Some(callback),
                com_initialized,
                mf_initialized,
            })
        }
    }
    // happens, windows canonacalizes thier urls or somthh and browsers don't understand that path
    // extremely important step like literally ln73
    pub fn load_video(&self, path: &str) -> WmfResult<()> {
        println!("[wmf] Loading video");

        unsafe {
            let engine = self.media_engine.as_ref().ok_or_else(|| {
                println!("[wmf] error: engine not initialized");
                "engine not initialized".to_string()
            })?;

            let clean_path = path
                .strip_prefix(r"\\?\")
                .unwrap_or(path)
                .replace('\\', "/");

            let file_url = if clean_path.starts_with("file:") {
                clean_path
            } else {
                format!("file:///{}", clean_path)
            };

            println!("[wmf] video url: {}", file_url);

            let wide_url: Vec<u16> = file_url.encode_utf16().chain(Some(0)).collect();
            let bstr = BSTR::from_wide(&wide_url[..wide_url.len() - 1]);

            engine.SetSource(&bstr).map_err(|e| {
                println!("[wmf] error: SetSource failed: {}", e);
                format!("SetSource failed: {}", e)
            })?;

            std::thread::sleep(std::time::Duration::from_millis(150));

            // OPTIMIZATIONS: Uh Added Loop enabled, audio off, low priority
            engine.SetLoop(true).map_err(|e| {
                println!("[wmf] error: SetLoop failed: {}", e);
                format!("SetLoop failed: {}", e)
            })?;

            engine.SetMuted(true).map_err(|e| {
                println!("[wmf] error: SetMuted failed: {}", e);
                format!("SetMuted failed: {}", e)
            })?;
            let _ = engine.SetVolume(0.0);
            println!("[wmf] OK loaded.");
            Ok(())
        }
    }

    pub fn play(&self) -> WmfResult<()> {
        unsafe {
            let engine = self.media_engine.as_ref().ok_or_else(|| {
                println!("[wmf] error: engine not initialized");
                "engine not initialized".to_string()
            })?;
            let _ = engine.SetPlaybackRate(1.0);
            engine.Play().map_err(|e| {
                println!("[wmf] error: Play failed: {}", e);
                format!("Play failed: {}", e)
            })?;

            println!("[wmf] Playback started");
            Ok(())
        }
    }

    pub fn hwnd(&self) -> HWND {
        self.hwnd
    }

    pub fn reload_media_engine(&mut self) -> WmfResult<()> {
        println!("[wmf] Reloading media engine");

        unsafe {
            if let Some(engine) = self.media_engine.take() {
                let _ = engine.Shutdown();
            }
            let d3d_device = self
                ._d3d_device
                .as_ref()
                .ok_or_else(|| "d3d device not initialized".to_string())?;
            let (new_engine, new_callback) = create_optimized_media_engine(self.hwnd, d3d_device)?;
            self.media_engine = Some(new_engine);
            self._callback = Some(new_callback);
            println!("[wmf] Media engine reloaded");
            Ok(())
        }
    }

    pub fn shutdown(&mut self) {
        println!("[wmf] Shutting down player");
        unsafe {
            if let Some(engine) = self.media_engine.take() {
                let _ = engine.Pause();
                let _ = engine.Shutdown();
            }

            if !self.hwnd.0.is_null() {
                let _ = DestroyWindow(self.hwnd);
                self.hwnd = HWND(std::ptr::null_mut());
            }

            self._d3d_device = None;
            self._callback = None;

            if self.mf_initialized {
                let _ = MFShutdown();
                self.mf_initialized = false;
            }

            if self.com_initialized {
                CoUninitialize();
                self.com_initialized = false;
            }
        }
        println!("[wmf] OK kill(0)");
    }
}

impl Drop for WmfPlayer {
    fn drop(&mut self) {
        self.shutdown();
    }
}

unsafe fn create_player_window(width: i32, height: i32) -> WmfResult<HWND> {
    let class_name = w!("WmfPlayerWindow");

    let wc = WNDCLASSW {
        lpfnWndProc: Some(wnd_proc),
        hInstance: HINSTANCE(std::ptr::null_mut()),
        lpszClassName: PCWSTR(class_name.as_ptr()),
        style: CS_HREDRAW | CS_VREDRAW,
        hbrBackground: windows::Win32::Graphics::Gdi::HBRUSH(1 as _),
        ..Default::default()
    };

    let _ = RegisterClassW(&wc);

    let win_ver = get_windows_version();

    // OS-SPECIFIC WINDOW CREATION
    // Windows 11 24H2+: Requires WS_EX_LAYERED for raised desktop with layered ShellView
    // Windows 10/11 Pre-24H2: Use WS_EX_TRANSPARENT to avoid D3D11 swap chain conflicts
    let ex_style = if win_ver.is_windows_11_24h2_plus() {
        println!("[wmf_player] Using Windows 11 24H2+ window style (WS_EX_LAYERED)");
        WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_NOPARENTNOTIFY
    } else {
        println!("[wmf_player] Using Windows 10/11 window style (WS_EX_TRANSPARENT)");
        WS_EX_TRANSPARENT | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_NOPARENTNOTIFY
    };

    let hwnd = CreateWindowExW(
        ex_style,
        class_name,
        w!("WMF Player"),
        WS_POPUP, // Created as POPUP (no WS_DISABLED), converted to CHILD during injection
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

    // Initialize layered window attributes ONLY on Windows 11 24H2+
    // On Windows 10, layered windows with D3D11 swap chains can cause issues
    if win_ver.is_windows_11_24h2_plus() {
        println!("[wmf_player] Initializing layered window attributes");
        SetLayeredWindowAttributes(hwnd, COLORREF(0), 255, LWA_ALPHA)
            .map_err(|e| format!("SetLayeredWindowAttributes failed: {}", e))?;
    }

    // Hide window initially - will be shown after injection
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

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    // Log every message to identify what causes the hang
    if msg != WM_PAINT && msg != WM_ERASEBKGND && msg != 0x0113
    /* WM_TIMER */
    {
        println!("[wmf_wnd] Received message: 0x{:04X}", msg);
    }

    match msg {
        WM_DESTROY => {
            println!("[wmf_wnd] WM_DESTROY");
            PostQuitMessage(0);
            LRESULT(0)
        }
        WM_MOUSEACTIVATE => {
            println!("[wmf_wnd] WM_MOUSEACTIVATE - returning MA_NOACTIVATE");
            LRESULT(MA_NOACTIVATE as isize)
        }
        WM_NCHITTEST => {
            // Return HTNOWHERE immediately without logging (too many messages)
            LRESULT(HTNOWHERE as isize)
        }
        WM_SETCURSOR => LRESULT(1),
        WM_ACTIVATE => {
            println!("[wmf_wnd] WM_ACTIVATE");
            LRESULT(0)
        }
        WM_SETFOCUS => {
            println!("[wmf_wnd] WM_SETFOCUS");
            LRESULT(0)
        }
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let _hdc = BeginPaint(hwnd, &mut ps);
            let _ = EndPaint(hwnd, &ps);
            LRESULT(0)
        }
        WM_ERASEBKGND => LRESULT(1),
        _ => {
            // Log unhandled messages
            if msg >= 0x0200 && msg <= 0x020E {
                println!("[wmf_wnd] Mouse message: 0x{:04X}", msg);
            }
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
    }
}

// OPTIMIZATION: hardware acceleration but with power-saving flags
unsafe fn create_optimized_d3d_device() -> WmfResult<ID3D11Device> {
    let mut device: Option<ID3D11Device> = None;
    let mut context: Option<ID3D11DeviceContext> = None;

    // hardware acceleration with VIDEO_SUPPORT for efficiency (uh ai told me this, idk if it works or not)
    D3D11CreateDevice(
        None,
        D3D_DRIVER_TYPE_HARDWARE,
        HMODULE::default(),
        D3D11_CREATE_DEVICE_VIDEO_SUPPORT | D3D11_CREATE_DEVICE_BGRA_SUPPORT,
        None,
        D3D11_SDK_VERSION,
        Some(&mut device),
        None,
        Some(&mut context),
    )
    .map_err(|e| format!("D3D11CreateDevice failed: {}. Update graphics drivers", e))?;

    device.ok_or_else(|| "D3D device is null".into())
}

unsafe fn create_optimized_media_engine(
    hwnd: HWND,
    device: &ID3D11Device,
) -> WmfResult<(IMFMediaEngine, IMFMediaEngineNotify)> {
    let dxgi_manager = create_dxgi_manager(device)?;
    let callback = MediaEngineNotify::new();
    let callback_interface: IMFMediaEngineNotify = callback.into();
    let attributes = create_optimized_mf_attributes(hwnd, &dxgi_manager, &callback_interface)?;

    let factory: IMFMediaEngineClassFactory =
        CoCreateInstance(&CLSID_MFMediaEngineClassFactory, None, CLSCTX_ALL)
            .map_err(|e| format!("CoCreateInstance failed: {}", e))?;

    // low-latency flag for better performance
    let engine = factory
        .CreateInstance(MF_MEDIA_ENGINE_REAL_TIME_MODE.0 as u32, &attributes)
        .map_err(|e| format!("CreateInstance failed: {}", e))?;

    Ok((engine, callback_interface))
}

unsafe fn create_dxgi_manager(device: &ID3D11Device) -> WmfResult<IMFDXGIDeviceManager> {
    let mut reset_token: u32 = 0;
    let mut manager: Option<IMFDXGIDeviceManager> = None;
    MFCreateDXGIDeviceManager(&mut reset_token, &mut manager)
        .map_err(|e| format!("MFCreateDXGIDeviceManager failed: {}", e))?;

    let manager = manager.ok_or_else(|| "DXGI manager is null".to_string())?;

    manager
        .ResetDevice(device, reset_token)
        .map_err(|e| format!("ResetDevice failed: {}", e))?;

    Ok(manager)
}

unsafe fn create_optimized_mf_attributes(
    hwnd: HWND,
    dxgi_manager: &IMFDXGIDeviceManager,
    callback: &IMFMediaEngineNotify,
) -> WmfResult<IMFAttributes> {
    let mut attributes: Option<IMFAttributes> = None;
    MFCreateAttributes(&mut attributes, 8)
        .map_err(|e| format!("MFCreateAttributes failed: {}", e))?;

    let attributes = attributes.ok_or_else(|| "MFCreateAttributes returned null".to_string())?;

    attributes
        .SetUnknown(&MF_MEDIA_ENGINE_DXGI_MANAGER, dxgi_manager)
        .map_err(|e| format!("SetUnknown DXGI failed: {}", e))?;

    attributes
        .SetUINT64(
            &MF_MEDIA_ENGINE_VIDEO_OUTPUT_FORMAT,
            DXGI_FORMAT_B8G8R8A8_UNORM.0 as u64,
        )
        .map_err(|e| format!("SetUINT64 format failed: {}", e))?;

    attributes
        .SetUINT64(&MF_MEDIA_ENGINE_PLAYBACK_HWND, hwnd.0 as u64)
        .map_err(|e| format!("SetUINT64 hwnd failed: {}", e))?;

    attributes
        .SetUnknown(&MF_MEDIA_ENGINE_CALLBACK, callback)
        .map_err(|e| format!("SetUnknown callback failed: {}", e))?;

    // OPTIMIZATION: Disable DRM/protection for lower overhead
    attributes
        .SetUINT32(&MF_MEDIA_ENGINE_CONTENT_PROTECTION_FLAGS, 0)
        .map_err(|e| format!("SetUINT32 protection failed: {}", e))?;

    // OPTIMIZATION: Background audio category (lowest priority)
    attributes
        .SetUINT32(&MF_MEDIA_ENGINE_AUDIO_CATEGORY, 0)
        .map_err(|e| format!("SetUINT32 audio failed: {}", e))?;

    // OPTIMIZATION: Enable hardware acceleration
    attributes
        .SetUINT32(
            &MF_MEDIA_ENGINE_VIDEO_OUTPUT_FORMAT,
            DXGI_FORMAT_B8G8R8A8_UNORM.0 as u32,
        )
        .map_err(|e| format!("SetUINT32 video format failed: {}", e))?;

    Ok(attributes)
}

#[implement(IMFMediaEngineNotify)]
struct MediaEngineNotify;

impl MediaEngineNotify {
    fn new() -> Self {
        Self
    }
}

impl IMFMediaEngineNotify_Impl for MediaEngineNotify_Impl {
    fn EventNotify(&self, event: u32, _param1: usize, _param2: u32) -> windows::core::Result<()> {
        // ignore event spams
        if event == 7 {
            println!("[wmf] Media error event");
        }
        Ok(())
    }
}

use windows::core::{implement, w, BSTR, PCWSTR};
use windows::Win32::Foundation::{HINSTANCE, HMODULE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Direct3D::D3D_DRIVER_TYPE_HARDWARE;
use windows::Win32::Graphics::Direct3D11::*;
use windows::Win32::Graphics::Dxgi::Common::*;
use windows::Win32::Graphics::Gdi::{BeginPaint, EndPaint, PAINTSTRUCT};
use windows::Win32::Media::MediaFoundation::*;
use windows::Win32::System::Com::*;
use windows::Win32::UI::WindowsAndMessaging::*;
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
            println!("initializing new player c:");

            // COM initialization - be careful with thread state
            let hr = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
            let com_initialized = if hr.is_ok() || hr.0 == 0x00000001 {
                true
            } else {
                println!("[wmf] warning: COM already initialized or failed: {:?}", hr);
                false
            };

            // MF initialization
            let mf_result = MFStartup(MF_VERSION, MFSTARTUP_FULL);
            let mf_initialized = if mf_result.is_ok() {
                true
            } else {
                println!("[wmf] warning: MF startup failed: {:?}", mf_result);
                false
            };

            let hwnd = create_player_window(width, height)?;
            let d3d_device = create_d3d_device()?;
            let (media_engine, callback) = create_media_engine(hwnd, &d3d_device)?;

            println!("[wmf] player created successfully");

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

    pub fn load_video(&self, path: &str) -> WmfResult<()> {
        println!("[wmf] load video");
        println!("[wmf] input path: {}", path);

        unsafe {
            let engine = self.media_engine.as_ref().ok_or_else(|| {
                println!("[wmf] error: engine not initialized");
                "engine not initialized".to_string()
            })?;

            // windows like always is ass, and browsers dont get what windows path makes so we normalization
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

            println!("[wmf] setting video source...");
            engine.SetSource(&bstr).map_err(|e| {
                println!("[wmf] error: setsource failed: {}", e);
                format!("setsource failed: {}", e)
            })?;
            println!("[wmf] source set ok");

            // js some time for source to be processed
            std::thread::sleep(std::time::Duration::from_millis(150));

            println!("[wmf] configuring playback (loop, muted)...");
            engine.SetLoop(true).map_err(|e| {
                println!("[wmf] error: setloop failed: {}", e);
                format!("setloop failed: {}", e)
            })?;
            engine.SetMuted(true).map_err(|e| {
                println!("[wmf] error: setmuted failed: {}", e);
                format!("setmuted failed: {}", e)
            })?;
            println!("[wmf] playback configured");

            println!("[wmf] video loaded");
            Ok(())
        }
    }

    pub fn stop(&self) -> Result<(), String> {
        println!("[wmf] stop video");
        unsafe {
            let engine = self
                .media_engine
                .as_ref()
                .ok_or_else(|| "engine not initialized".to_string())?;

            // first pause
            let _ = engine.Pause();

            // set source to empty to clear the video
            let empty_bstr = BSTR::new();
            let _ = engine.SetSource(&empty_bstr);

            // seek to beginning
            let _ = engine.SetCurrentTime(0.0);
        }
        println!("[wmf] video stopped and cleared");
        Ok(())
    }

    pub fn play(&self) -> WmfResult<()> {

        unsafe {
            let engine = self.media_engine.as_ref().ok_or_else(|| {
                println!("[wmf] error: engine not initialized");
                "engine not initialized".to_string()
            })?;

            println!("[wmf] set playback rate 1.0...");
            let _ = engine.SetPlaybackRate(1.0);

            println!("[wmf] starting playback...");
            engine.Play().map_err(|e| {
                println!("[wmf] error: play failed: {}", e);
                format!("play failed: {}", e)
            })?;
            println!("[wmf] playback started");

            println!("works ok");
            Ok(())
        }
    }

    pub fn hwnd(&self) -> HWND {
        self.hwnd
    }

    pub fn reload_media_engine(&mut self) -> WmfResult<()> {
        println!("[wmf] reloading media engine for video swap");

        unsafe {
            // Shutdown old engine
            if let Some(engine) = self.media_engine.take() {
                let _ = engine.Shutdown();
            }

            // Recreate media engine with same device
            let d3d_device = self
                ._d3d_device
                .as_ref()
                .ok_or_else(|| "d3d device not initialized".to_string())?;

            let (new_engine, new_callback) = create_media_engine(self.hwnd, d3d_device)?;

            self.media_engine = Some(new_engine);
            self._callback = Some(new_callback);

            println!("[wmf] media engine reloaded successfully");
            Ok(())
        }
    }

    pub fn shutdown(&mut self) {
        println!("[wmf] shutting down player resources...");

        unsafe {
            // Shutdown media engine first
            if let Some(engine) = self.media_engine.take() {
                println!("[wmf] shutting down media engine");
                let _ = engine.Pause();
                let _ = engine.Shutdown();
            }

            // Destroy window
            if !self.hwnd.0.is_null() {
                println!("[wmf] destroying window {:?}", self.hwnd);
                let _ = DestroyWindow(self.hwnd);
                self.hwnd = HWND(std::ptr::null_mut());
            }

            // Clear other resources
            self._d3d_device = None;
            self._callback = None;

            // MF shutdown
            if self.mf_initialized {
                let _ = MFShutdown();
                self.mf_initialized = false;
            }

            // COM uninitialize
            if self.com_initialized {
                CoUninitialize();
                self.com_initialized = false;
            }
        }

        println!("[wmf] shutdown complete");
    }
}

impl Drop for WmfPlayer {
    fn drop(&mut self) {
        println!("[wmf] dropping player");
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

    let ex_style = WS_EX_LAYERED
        | WS_EX_TRANSPARENT
        | WS_EX_TOOLWINDOW
        | WS_EX_NOACTIVATE
        | WS_EX_NOPARENTNOTIFY;

    let hwnd = CreateWindowExW(
        ex_style,
        class_name,
        w!("WMF Player"),
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
    .map_err(|e| format!("createwindowexw failed: {}", e))?;

    use windows::Win32::Foundation::COLORREF;
    use windows::Win32::UI::WindowsAndMessaging::{SetLayeredWindowAttributes, LWA_ALPHA};
    let _ = SetLayeredWindowAttributes(hwnd, COLORREF(0), 255, LWA_ALPHA);

    // WE DONT SHOW WINDOW YET - IT WILL BE SHOWN AFTER INJECTION
    // THIS PREVENTS IT FROM APPEARING IN TASKBAR BEFORE BEING INJECTED
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
    match msg {
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        WM_MOUSEACTIVATE => LRESULT(MA_NOACTIVATE as isize),
        WM_NCHITTEST => LRESULT(HTTRANSPARENT as isize),
        WM_SETCURSOR => LRESULT(1),
        WM_ACTIVATE => LRESULT(0),
        WM_SETFOCUS => LRESULT(0),
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let _hdc = BeginPaint(hwnd, &mut ps);
            let _ = EndPaint(hwnd, &ps);
            LRESULT(0)
        }
        WM_ERASEBKGND => LRESULT(1),
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

unsafe fn create_d3d_device() -> WmfResult<ID3D11Device> {
    let mut device: Option<ID3D11Device> = None;
    let mut context: Option<ID3D11DeviceContext> = None;

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
    .map_err(|e| format!("d3d11createdevice failed: {}. update graphics drivers.", e))?;

    device.ok_or_else(|| "d3d device is null".into())
}

unsafe fn create_media_engine(
    hwnd: HWND,
    device: &ID3D11Device,
) -> WmfResult<(IMFMediaEngine, IMFMediaEngineNotify)> {
    let dxgi_manager = create_dxgi_manager(device)?;
    let callback = MediaEngineNotify::new();
    let callback_interface: IMFMediaEngineNotify = callback.into();
    let attributes = create_mf_attributes(hwnd, &dxgi_manager, &callback_interface)?;

    let factory: IMFMediaEngineClassFactory =
        CoCreateInstance(&CLSID_MFMediaEngineClassFactory, None, CLSCTX_ALL)
            .map_err(|e| format!("cocreateinstance failed: {}", e))?;

    let engine = factory
        .CreateInstance(0, &attributes)
        .map_err(|e| format!("createinstance failed: {}", e))?;

    Ok((engine, callback_interface))
}

unsafe fn create_dxgi_manager(device: &ID3D11Device) -> WmfResult<IMFDXGIDeviceManager> {
    let mut reset_token: u32 = 0;
    let mut manager: Option<IMFDXGIDeviceManager> = None;
    MFCreateDXGIDeviceManager(&mut reset_token, &mut manager)
        .map_err(|e| format!("mfcreatedxgidevicemanager failed: {}", e))?;

    let manager = manager.ok_or_else(|| "dxgi manager is null".to_string())?;

    manager
        .ResetDevice(device, reset_token)
        .map_err(|e| format!("resetdevice failed: {}", e))?;

    Ok(manager)
}

unsafe fn create_mf_attributes(
    hwnd: HWND,
    dxgi_manager: &IMFDXGIDeviceManager,
    callback: &IMFMediaEngineNotify,
) -> WmfResult<IMFAttributes> {
    let mut attributes: Option<IMFAttributes> = None;
    MFCreateAttributes(&mut attributes, 5)
        .map_err(|e| format!("mfcreateattributes failed: {}", e))?;

    let attributes = attributes.ok_or_else(|| "mfcreateattributes returned null".to_string())?;

    attributes
        .SetUnknown(&MF_MEDIA_ENGINE_DXGI_MANAGER, dxgi_manager)
        .map_err(|e| format!("setunknown dxgi failed: {}", e))?;

    attributes
        .SetUINT64(
            &MF_MEDIA_ENGINE_VIDEO_OUTPUT_FORMAT,
            DXGI_FORMAT_B8G8R8A8_UNORM.0 as u64,
        )
        .map_err(|e| format!("setuint64 format failed: {}", e))?;

    attributes
        .SetUINT64(&MF_MEDIA_ENGINE_PLAYBACK_HWND, hwnd.0 as u64)
        .map_err(|e| format!("setuint64 hwnd failed: {}", e))?;

    attributes
        .SetUnknown(&MF_MEDIA_ENGINE_CALLBACK, callback)
        .map_err(|e| format!("setunknown callback failed: {}", e))?;

    attributes
        .SetUINT32(&MF_MEDIA_ENGINE_CONTENT_PROTECTION_FLAGS, 0)
        .map_err(|e| format!("setuint32 protection failed: {}", e))?;

    attributes
        .SetUINT32(&MF_MEDIA_ENGINE_AUDIO_CATEGORY, 0)
        .map_err(|e| format!("setuint32 audio category failed: {}", e))?;

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
    fn EventNotify(&self, event: u32, param1: usize, param2: u32) -> windows::core::Result<()> {
        if event == 7 {
            println!("[wmf] error event: code={} detail={}", param1, param2);
        }
        Ok(())
    }
}

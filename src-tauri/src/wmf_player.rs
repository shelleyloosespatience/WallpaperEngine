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
    // hwnd for native window :3
    hwnd: HWND,
    // media engine core
    media_engine: Option<IMFMediaEngine>,
    // d3d device for rendering
    _d3d_device: Option<ID3D11Device>,
    // callback for events
    _callback: Option<IMFMediaEngineNotify>,
}

unsafe impl Send for WmfPlayer {}

impl WmfPlayer {
    pub fn new(width: i32, height: i32) -> WmfResult<Self> {
        unsafe {
            // com init, allow changed mode
            let hr = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
            if hr.is_err() && hr.0 != 0x00000001 {
                return Err(format!("com init failed: {:?}", hr));
            }

            MFStartup(MF_VERSION, MFSTARTUP_FULL)
                .map_err(|e| format!("mf startup failed: {}", e))?;

            let hwnd = create_player_window(width, height)?;
            let d3d_device = create_d3d_device()?;
            let (media_engine, callback) = create_media_engine(hwnd, &d3d_device)?;

            Ok(Self {
                hwnd,
                media_engine: Some(media_engine),
                _d3d_device: Some(d3d_device),
                _callback: Some(callback),
            })
        }
    }

    pub fn load_video(&self, path: &str) -> WmfResult<()> {
        // path normalization, url conversion
        println!("[wmf] load video");
        println!("[wmf] input path: {}", path);
        
        unsafe {
            let engine = self
                .media_engine
                .as_ref()
                .ok_or_else(|| {
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
            let bstr = BSTR::from_wide(&wide_url[..wide_url.len()-1]);
            
            println!("[wmf] setting video source...");
            engine.SetSource(&bstr).map_err(|e| {
                println!("[wmf] error: setsource failed: {}", e);
                format!("setsource failed: {}", e)
            })?;
            println!("[wmf] source set ok");
            
            std::thread::sleep(std::time::Duration::from_millis(100));
            
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
        // pause playback
        println!("[wmf] stop video");
        unsafe {
            let engine = self
                .media_engine
                .as_ref()
                .ok_or_else(|| "engine not initialized".to_string())?;
            
            engine.Pause()
                .map_err(|e| format!("failed to pause: {}", e))?;
        }
        println!("[wmf] video stopped");
        Ok(())
    }

    pub fn play(&self) -> WmfResult<()> {
        // start playback, set window z-order
        println!("[wmf] ========== play video ==========");
        
        unsafe {
            let engine = self
                .media_engine
                .as_ref()
                .ok_or_else(|| {
                    println!("[wmf] error: engine not initialized");
                    "engine not initialized".to_string()
                })?;
            
            // set playback rate to normal
            println!("[wmf] set playback rate 1.0...");
            let _ = engine.SetPlaybackRate(1.0);
            
            println!("[wmf] starting playback...");
            engine.Play().map_err(|e| {
                println!("[wmf] error: play failed: {}", e);
                format!("play failed: {}", e)
            })?;
            println!("[wmf] playback started");
            
            // keep window in background
            println!("[wmf] positioning window to background...");
            use windows::Win32::UI::WindowsAndMessaging::{SetWindowPos, HWND_BOTTOM, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE};
            let _ = SetWindowPos(
                self.hwnd,
                Some(HWND_BOTTOM),
                0,
                0,
                0,
                0,
                SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE,
            );
            println!("[wmf] window positioned to background");
            
            println!("[wmf] ========== video playing ==========");
            Ok(())
        }
    }

    pub fn hwnd(&self) -> HWND {
        self.hwnd
    }
}

impl Drop for WmfPlayer {
    fn drop(&mut self) {
        unsafe {
            // shutdown engine, cleanup
            if let Some(engine) = self.media_engine.take() {
                let _ = engine.Shutdown();
            }
            
            self._d3d_device = None;
            self._callback = None;
            
            if !self.hwnd.0.is_null() {
                let _ = DestroyWindow(self.hwnd);
                self.hwnd = HWND(std::ptr::null_mut());
            }
            
            let _ = MFShutdown();
            CoUninitialize();
        }
    }
}

unsafe fn create_player_window(width: i32, height: i32) -> WmfResult<HWND> {
    // native window, transparent, non-interactive
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

    // ex style for transparency/toolwindow
    let ex_style = WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_NOPARENTNOTIFY;

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

    // set layered attributes for alpha
    use windows::Win32::UI::WindowsAndMessaging::{SetLayeredWindowAttributes, LWA_ALPHA};
    use windows::Win32::Foundation::COLORREF;
    let _ = SetLayeredWindowAttributes(hwnd, COLORREF(0), 255, LWA_ALPHA);
    
    // show without activating
    let _ = ShowWindow(hwnd, SW_SHOWNA);
    
    // keep in background
    use windows::Win32::UI::WindowsAndMessaging::SetWindowPos;
    let _ = SetWindowPos(
        hwnd,
        Some(HWND_BOTTOM),
        0,
        0,
        0,
        0,
        SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW,
    );

    Ok(hwnd)
}

unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    // minimal message handling, no focus, no input
    match msg {
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        WM_MOUSEACTIVATE => LRESULT(MA_NOACTIVATE as isize),
        WM_NCHITTEST => LRESULT(HTTRANSPARENT as isize),
        WM_SETCURSOR => {
            // always transparent cursor
            LRESULT(1)
        },
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
    // d3d11 device for video
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
    // mf media engine setup, dxgi + callback
    let dxgi_manager = create_dxgi_manager(device)?;
    let callback = MediaEngineNotify::new();
    let callback_interface: IMFMediaEngineNotify = callback.into();
    let attributes = create_mf_attributes(hwnd, &dxgi_manager, &callback_interface)?;
    
    let factory: IMFMediaEngineClassFactory = CoCreateInstance(
        &CLSID_MFMediaEngineClassFactory,
        None,
        CLSCTX_ALL,
    )
    .map_err(|e| format!("cocreateinstance failed: {}", e))?;

    let engine = factory
        .CreateInstance(0, &attributes)
        .map_err(|e| format!("createinstance failed: {}", e))?;

    Ok((engine, callback_interface))
}

unsafe fn create_dxgi_manager(device: &ID3D11Device) -> WmfResult<IMFDXGIDeviceManager> {
    // dxgi device manager for mf
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
    // mf attributes for engine
    let mut attributes: Option<IMFAttributes> = None;
    MFCreateAttributes(&mut attributes, 5)
        .map_err(|e| format!("mfcreateattributes failed: {}", e))?;

    let attributes = attributes.ok_or_else(|| "mfcreateattributes returned null".to_string())?;

    attributes
        .SetUnknown(&MF_MEDIA_ENGINE_DXGI_MANAGER, dxgi_manager)
        .map_err(|e| format!("setunknown dxgi failed: {}", e))?;

    attributes
        .SetUINT64(&MF_MEDIA_ENGINE_VIDEO_OUTPUT_FORMAT, DXGI_FORMAT_B8G8R8A8_UNORM.0 as u64)
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

    // low-latency mode for gpu
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
    // event callback, error event = 7
    fn EventNotify(&self, event: u32, param1: usize, param2: u32) -> windows::core::Result<()> {
        if event == 7 {
            println!("[wmf] error event: code={} detail={}", param1, param2);
        }
        Ok(())
    }
}
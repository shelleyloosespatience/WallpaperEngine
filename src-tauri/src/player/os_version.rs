use std::sync::OnceLock;

static OS_VERSION: OnceLock<WindowsVersion> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowsVersion {
    Windows10,
    Windows11Pre24H2,
    Windows1124H2Plus,
    Unknown,
}

impl WindowsVersion {
    #[allow(dead_code)]
    pub fn is_windows_11_24h2_plus(&self) -> bool {
        matches!(self, WindowsVersion::Windows1124H2Plus)
    }

    #[allow(dead_code)]
    pub fn is_windows_11(&self) -> bool {
        matches!(
            self,
            WindowsVersion::Windows11Pre24H2 | WindowsVersion::Windows1124H2Plus
        )
    }


    #[allow(dead_code)]
    pub fn is_windows_10(&self) -> bool {
        matches!(self, WindowsVersion::Windows10)
    }
}

/// the current Windows version (cached after first call)
pub fn get_windows_version() -> WindowsVersion {
    *OS_VERSION.get_or_init(|| detect_windows_version())
}

fn detect_windows_version() -> WindowsVersion {
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
                        let status = rtl_get_version(&mut version_info);

                        if status == 0 {
                            let build = version_info.dwBuildNumber;
                            let major = version_info.dwMajorVersion;

                            println!(
                                "[os_version] Detected Windows {}.{} Build {}",
                                major, version_info.dwMinorVersion, build
                            );

                            return match (major, build) {
                                (10, b) if b >= 26100 => {
                                    println!("[os_version] Windows 11 24H2+ detected");
                                    WindowsVersion::Windows1124H2Plus
                                }
                                (10, b) if b >= 22000 => {
                                    println!("[os_version] Windows 11 (pre-24H2) detected");
                                    WindowsVersion::Windows11Pre24H2
                                }
                                (10, _) => {
                                    println!("[os_version] Windows 10 detected");
                                    WindowsVersion::Windows10
                                }
                                _ => {
                                    println!("[os_version] Unknown Windows version");
                                    WindowsVersion::Unknown
                                }
                            };
                        }
                    }
                    None => {
                        println!("[os_version] Warning: RtlGetVersion not found in ntdll.dll");
                    }
                }
            }
            Err(e) => {
                println!("[os_version] Warning: Could not load ntdll.dll: {}", e);
            }
        }

        WindowsVersion::Unknown
    }
}

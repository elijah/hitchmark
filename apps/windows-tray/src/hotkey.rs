use anyhow::Result;

/// Global Ctrl+Alt+H hotkey registration.
pub struct GlobalHotkey {
    #[cfg(target_os = "windows")]
    id: i32,
}

impl GlobalHotkey {
    pub fn register() -> Result<Self> {
        #[cfg(target_os = "windows")]
        {
            use windows::Win32::UI::WindowsAndMessaging::{
                RegisterHotKey, HWND, MOD_ALT, MOD_CONTROL,
            };

            let id = 1;
            // 'H' virtual-key code.
            let key_h: u32 = b'H' as u32;
            let ok = unsafe { RegisterHotKey(HWND(0), id, MOD_CONTROL | MOD_ALT, key_h) };
            if !ok.as_bool() {
                anyhow::bail!("RegisterHotKey(Ctrl+Alt+H) failed");
            }
            return Ok(Self { id });
        }

        #[cfg(not(target_os = "windows"))]
        {
            Ok(Self {})
        }
    }

    pub fn poll_triggered(&self) -> bool {
        #[cfg(target_os = "windows")]
        {
            use windows::Win32::UI::WindowsAndMessaging::{
                PeekMessageW, HWND, MSG, PM_REMOVE, WM_HOTKEY,
            };

            let mut msg = MSG::default();
            let has_message =
                unsafe { PeekMessageW(&mut msg, HWND(0), WM_HOTKEY, WM_HOTKEY, PM_REMOVE) };
            return has_message.as_bool() && msg.wParam.0 == self.id as usize;
        }

        #[cfg(not(target_os = "windows"))]
        {
            false
        }
    }
}

#[cfg(target_os = "windows")]
impl Drop for GlobalHotkey {
    fn drop(&mut self) {
        use windows::Win32::UI::WindowsAndMessaging::{UnregisterHotKey, HWND};

        let _ = unsafe { UnregisterHotKey(HWND(0), self.id) };
    }
}

use std::ffi::CString;

use windows::{
    core::PCSTR,
    Win32::{
        Foundation::{HWND, POINT},
        UI::WindowsAndMessaging::{
            CreatePopupMenu, GetCursorPos, InsertMenuA, SetForegroundWindow, TrackPopupMenu, HMENU,
            MF_BYPOSITION, MF_DISABLED, MF_SEPARATOR, TPM_BOTTOMALIGN,
        },
    },
};

use super::TrayEvent;

pub struct TrayMenu {
    handle: HMENU,
    tray_window_handle: HWND,
    position: (i32, i32),
}

impl TrayMenu {
    #[must_use]
    #[allow(clippy::new_ret_no_self)]
    /// # Panics
    ///
    /// Will panic if the underlying call to [`CreatePopupMenu()`] fails.
    pub fn new(tray_window_handle: HWND) -> TrayMenuBuilder {
        TrayMenuBuilder {
            handle: unsafe { CreatePopupMenu().unwrap() },
            tray_window_handle,
            position: None,
            item_count: 0,
        }
    }

    pub fn show(&self) {
        unsafe {
            SetForegroundWindow(self.tray_window_handle);
            TrackPopupMenu(
                self.handle,
                TPM_BOTTOMALIGN,
                self.position.0,
                self.position.1,
                0,
                self.tray_window_handle,
                None,
            );
        }
    }
}

pub struct TrayMenuBuilder {
    handle: HMENU,
    tray_window_handle: HWND,
    item_count: u32,
    position: Option<(i32, i32)>,
}

impl TrayMenuBuilder {
    pub fn set_position(&mut self, position: (i32, i32)) -> &mut Self {
        self.position = Some(position);

        self
    }

    /// Adds a button to the tray popup menu.
    ///
    /// # Arguments
    ///
    /// * `label` - The text to displayed on the button item.
    /// * `event` - The event to generate when the button is pressed.
    ///             If `None`, the button will be disabled and greyed out.
    ///
    /// # Panics
    ///
    /// Will panic if `label` is not nul-terminated
    pub fn add_button(&mut self, label: &str, event: Option<TrayEvent>) -> &mut Self {
        self.item_count += 1;

        let mut uflags = MF_BYPOSITION;

        if event.is_none() {
            uflags |= MF_DISABLED;
        }

        let c_label = CString::new(label).expect("invalid C-style string");
        // CStr::from_bytes_with_nul(label.as_bytes())
        //     .unwrap()
        //     .as_ptr()
        //     .cast::<u8>(),

        unsafe {
            InsertMenuA(
                self.handle,
                self.item_count,
                uflags,
                event.map_or(0, |ev| ev as usize),
                PCSTR(c_label.as_ptr().cast::<u8>()),
            )
        };

        self
    }

    pub fn add_separator(&mut self) -> &mut Self {
        self.item_count += 1;

        unsafe {
            InsertMenuA(
                self.handle,
                self.item_count,
                MF_BYPOSITION | MF_SEPARATOR,
                0,
                PCSTR::null(),
            )
        };

        self
    }

    pub fn build(&mut self) -> TrayMenu {
        TrayMenu {
            handle: self.handle,
            tray_window_handle: self.tray_window_handle,

            // Default position to position of mouse cursor
            position: self.position.unwrap_or({
                let mut cursor_position = POINT::default();
                unsafe { GetCursorPos(&mut cursor_position) };
                (cursor_position.x, cursor_position.y)
            }),
        }
    }
}

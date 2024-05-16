#![allow(dead_code)]

use windows::{
    core::{w, HRESULT, PCSTR, PCWSTR},
    Win32::{
        Foundation::{BOOL, HWND, LPARAM, WPARAM},
        System::LibraryLoader::GetModuleHandleA,
        UI::{
            Controls::{
                TaskDialogIndirect, TASKDIALOGCONFIG, TASKDIALOGCONFIG_0, TASKDIALOG_FLAGS,
                TDF_ENABLE_HYPERLINKS, TDN_HYPERLINK_CLICKED, TD_ERROR_ICON, TD_INFORMATION_ICON,
                TD_SHIELD_ICON, TD_WARNING_ICON,
            },
            Shell::ShellExecuteW,
            WindowsAndMessaging::{IDCANCEL, IDOK, MESSAGEBOX_RESULT, SW_SHOWNORMAL},
        },
    },
};

#[derive(Clone, Copy)]
pub enum TaskDialogIcon {
    ErrorIcon,
    WarningIcon,
    InformationIcon,
    ShieldIcon,
    NoIcon,
}

impl TaskDialogIcon {
    #[must_use]
    pub fn to_icon_id(&self) -> PCWSTR {
        match self {
            Self::ErrorIcon => TD_ERROR_ICON,
            Self::WarningIcon => TD_WARNING_ICON,
            Self::InformationIcon => TD_INFORMATION_ICON,
            Self::ShieldIcon => TD_SHIELD_ICON,
            Self::NoIcon => PCWSTR(std::ptr::null()),
        }
    }
}

/// Creates a Windows task dialog modal to display information or
/// prompt user action. The dialog is created on a separate thread
/// so as not to occupy the main thread's message queue.
pub struct TaskDialog {}

impl TaskDialog {
    /// Returns a factory used to construct a new task dialog window.
    #[must_use]
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> TaskDialogBuilder {
        TaskDialogBuilder {
            dialog_title_utf16_nul: [0u16; 1].to_vec(),
            dialog_heading_utf16_nul: [0u16; 1].to_vec(),
            dialog_content_utf16_nul: [0u16; 1].to_vec(),
            dialog_icon: TaskDialogIcon::NoIcon,
            dialog_allow_hyperlinks: false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum TaskDialogAction {
    Ok,
    Cancel,
}

impl TaskDialogAction {
    #[must_use]
    pub fn from_id(id: MESSAGEBOX_RESULT) -> TaskDialogAction {
        match id {
            IDCANCEL => TaskDialogAction::Cancel,
            IDOK => TaskDialogAction::Ok,
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TaskDialogResult {
    verified: bool,
    action: TaskDialogAction,
}

#[derive(Clone)]
pub struct TaskDialogBuilder {
    dialog_title_utf16_nul: Vec<u16>,
    dialog_heading_utf16_nul: Vec<u16>,
    dialog_content_utf16_nul: Vec<u16>,

    dialog_icon: TaskDialogIcon,
    dialog_allow_hyperlinks: bool,
}

impl TaskDialogBuilder {
    pub fn set_title<Param: AsRef<str>>(&mut self, title: Param) -> &mut Self {
        self.dialog_title_utf16_nul = str::encode_utf16(title.as_ref())
            .chain(Some(0))
            .collect::<Vec<u16>>();
        self
    }

    pub fn set_heading<Param: AsRef<str>>(&mut self, heading: Param) -> &mut Self {
        self.dialog_heading_utf16_nul = str::encode_utf16(heading.as_ref())
            .chain(Some(0))
            .collect::<Vec<u16>>();
        self
    }

    pub fn set_content<Param: AsRef<str>>(&mut self, content: Param) -> &mut Self {
        self.dialog_content_utf16_nul = str::encode_utf16(content.as_ref())
            .chain(Some(0))
            .collect::<Vec<u16>>();
        self
    }

    pub fn set_icon(&mut self, icon: TaskDialogIcon) -> &mut Self {
        self.dialog_icon = icon;
        self
    }

    /// Allows for hyperlinks to be used in the title, heading, content or
    /// footer of the dialog modal using a HTML-like syntax. Hyperlinks are
    /// disabled by default and have to be explicitly enabled using this method.
    ///
    /// # Examples
    ///
    /// ``` Rust
    /// let dialog = TaskDialog::new()
    ///     .set_title("Title")
    ///     .set_heading("Heading")
    ///     .set_content("This is a <a href=\"https://xela.me\">link</a>!")
    ///     .set_hyperlinks_enabled(true)
    ///     .build();
    /// ```
    pub fn set_hyperlinks_enabled(&mut self, enabled: bool) -> &mut Self {
        self.dialog_allow_hyperlinks = enabled;
        self
    }

    /// Creates the dialog modal and blocks the calling thread until it is
    /// closed. The dialog itself runs on a separate thread so as to not to
    /// occupy the message queue of the calling thread.
    #[allow(clippy::missing_panics_doc)]
    pub fn display_blocking(&mut self) -> TaskDialogResult {
        let dialog_title = self.dialog_title_utf16_nul.clone();
        let dialog_heading = self.dialog_heading_utf16_nul.clone();
        let dialog_content = self.dialog_content_utf16_nul.clone();
        let icon = self.dialog_icon;
        let allow_hyperlinks = self.dialog_allow_hyperlinks;

        // Create the dialog thread and return the result
        std::thread::spawn(move || {
            let mut dialog_flags = 0;

            if allow_hyperlinks {
                dialog_flags |= TDF_ENABLE_HYPERLINKS.0;
            }

            let config = TASKDIALOGCONFIG {
                cbSize: u32::try_from(std::mem::size_of::<TASKDIALOGCONFIG>()).unwrap(),
                hInstance: unsafe { GetModuleHandleA(PCSTR(std::ptr::null())).unwrap().into() },
                pszWindowTitle: PCWSTR(dialog_title.as_ptr()),
                pszMainInstruction: PCWSTR(dialog_heading.as_ptr()),
                pszContent: PCWSTR(dialog_content.as_ptr()),
                Anonymous1: TASKDIALOGCONFIG_0 {
                    pszMainIcon: icon.to_icon_id(),
                },
                dwFlags: TASKDIALOG_FLAGS(dialog_flags),
                pfCallback: Some(dialog_notification_callback),
                ..TASKDIALOGCONFIG::default()
            };

            let mut button_id = 0;
            let mut radio_id = 0;
            let mut verified = BOOL(0);

            // Is this blocking..?
            crate::logf!("Creating dialog");
            unsafe {
                TaskDialogIndirect(
                    &config,
                    Some(&mut button_id),
                    Some(&mut radio_id),
                    Some(&mut verified),
                )
                .unwrap();
            }

            TaskDialogResult {
                verified: verified.as_bool(),
                action: TaskDialogAction::from_id(MESSAGEBOX_RESULT(button_id)),
            }
        })
        .join()
        .unwrap()
    }
}

/// Handles messages for task dialog modals and, if needed, forwards the
/// result of the dialog to the calling thread.
unsafe extern "system" fn dialog_notification_callback(
    _hwnd: HWND,
    notification: u32,
    _wparam: WPARAM,
    lparam: LPARAM,
    _callback_data: isize,
) -> HRESULT {
    if notification == u32::try_from(TDN_HYPERLINK_CLICKED.0).unwrap() {
        unsafe {
            ShellExecuteW(
                HWND(0),
                w!("open"),
                PCWSTR(lparam.0 as *const u16),
                PCWSTR(std::ptr::null()),
                PCWSTR(std::ptr::null()),
                SW_SHOWNORMAL,
            );
        }
    }

    HRESULT(0)
}

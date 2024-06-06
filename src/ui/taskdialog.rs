#![allow(dead_code)]

use std::{
    sync::{atomic::AtomicU32, Arc, Mutex},
    thread::JoinHandle,
};

use windows::{
    core::{w, HRESULT, PCSTR, PCWSTR},
    Win32::{
        Foundation::{BOOL, HWND, LPARAM, S_FALSE, S_OK, WPARAM},
        System::LibraryLoader::GetModuleHandleA,
        UI::{
            Controls::{
                TaskDialogIndirect, TASKDIALOGCONFIG, TASKDIALOGCONFIG_0,
                TASKDIALOG_COMMON_BUTTON_FLAGS, TASKDIALOG_FLAGS, TDCBF_CANCEL_BUTTON,
                TDCBF_NO_BUTTON, TDCBF_OK_BUTTON, TDCBF_YES_BUTTON, TDF_CALLBACK_TIMER,
                TDF_ENABLE_HYPERLINKS, TDF_SHOW_PROGRESS_BAR, TDF_VERIFICATION_FLAG_CHECKED,
                TDM_SET_PROGRESS_BAR_POS, TDM_SET_PROGRESS_BAR_RANGE, TDN_BUTTON_CLICKED,
                TDN_DIALOG_CONSTRUCTED, TDN_HYPERLINK_CLICKED, TDN_TIMER, TD_ERROR_ICON,
                TD_INFORMATION_ICON, TD_SHIELD_ICON, TD_WARNING_ICON,
            },
            Shell::ShellExecuteW,
            WindowsAndMessaging::{
                EndDialog, PostMessageA, SendMessageA, BN_CLICKED, IDCANCEL, IDNO, IDOK, IDYES,
                MESSAGEBOX_RESULT, SW_SHOWNORMAL, WM_COMMAND,
            },
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

pub struct TaskDialog {
    cb_data: Arc<Mutex<TaskDialogCallbackData>>,
    result: JoinHandle<TaskDialogResult>,
}

impl TaskDialog {
    /// Returns a factory used to construct a new task dialog window.
    #[must_use]
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> TaskDialogBuilder {
        TaskDialogBuilder {
            dialog_title_utf16_nul: [0u16; 1].to_vec(),
            dialog_heading_utf16_nul: [0u16; 1].to_vec(),
            dialog_content_utf16_nul: [0u16; 1].to_vec(),
            dialog_footer_utf16_nul: [0u16; 1].to_vec(),
            dialog_icon: TaskDialogIcon::NoIcon,
            dialog_allow_hyperlinks: false,
            dialog_verification_text_utf16_nul: [0u16; 1].to_vec(),
            initial_verification_value: false,
            dialog_buttons: [].to_vec(),
            dialog_progress_source: None,
            dialog_progress_min: 0,
            dialog_progress_max: 100,
        }
    }

    /// Returns the task dialogs window handle ([HWND]) as [isize].
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn handle(&self) -> isize {
        let lock = self.cb_data.lock().unwrap();
        lock.dialog_handle
    }

    /// Blocks the calling thread until the task dialog is dismissed
    /// and returns its result.
    #[allow(clippy::missing_panics_doc, clippy::must_use_candidate)]
    pub fn result(self) -> TaskDialogResult {
        let rs = self.result;
        rs.join().unwrap()
    }

    /// Sends the specified close action to the dialog and blockingly waits
    /// for the dialog to exit and return its result.
    ///
    /// # Notes
    ///
    /// The resulting action when this is called is always [`TaskDialogAction::Cancel`].
    // Todo: Figure out why PostMessage with WM_COMMAND always results in
    //       IDCANCEL regardless of what command is specified.
    #[allow(clippy::must_use_candidate, clippy::missing_panics_doc)]
    pub fn close(self) -> TaskDialogResult {
        let low_word = u16::try_from(IDOK.0).unwrap();
        let high_word = u16::try_from(BN_CLICKED).unwrap();

        let dword_wparam = (u32::from(low_word)) | u32::from(high_word) << 16;

        unsafe {
            PostMessageA(
                HWND(self.handle()),
                WM_COMMAND,
                WPARAM(dword_wparam as usize),
                LPARAM(0),
            )
            .unwrap();
        }

        self.result()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TaskDialogAction {
    None,
    Ok,
    Yes,
    No,
    Cancel,
}

impl TaskDialogAction {
    #[must_use]
    pub fn from_id(id: MESSAGEBOX_RESULT) -> TaskDialogAction {
        match id {
            IDCANCEL => TaskDialogAction::Cancel,
            IDYES => TaskDialogAction::Yes,
            IDNO => TaskDialogAction::No,
            IDOK => TaskDialogAction::Ok,
            _ => TaskDialogAction::None,
        }
    }

    #[must_use]
    /// # Panics
    ///
    /// Panics if `self` is `TaskDialogAction::None`
    pub fn to_id(&self) -> MESSAGEBOX_RESULT {
        match self {
            TaskDialogAction::Cancel => IDCANCEL,
            TaskDialogAction::Yes => IDYES,
            TaskDialogAction::No => IDNO,
            TaskDialogAction::Ok => IDOK,
            TaskDialogAction::None => {
                panic!("conversion failed because `TaskDialogAction::None` is not a valid MESSAGEBOX_RESULT");
            }
        }
    }

    #[must_use]
    pub fn to_common_button_flag(&self) -> TASKDIALOG_COMMON_BUTTON_FLAGS {
        match self {
            Self::None => TASKDIALOG_COMMON_BUTTON_FLAGS(0),
            Self::Yes => TDCBF_YES_BUTTON,
            Self::No => TDCBF_NO_BUTTON,
            Self::Ok => TDCBF_OK_BUTTON,
            Self::Cancel => TDCBF_CANCEL_BUTTON,
        }
    }
}

pub struct TaskDialogCallbackData {
    dialog_handle: isize,
    dialog_progress_source: Option<Arc<AtomicU32>>,
    dialog_progress_min: u16,
    dialog_progress_max: u16,
}

#[derive(Debug, Clone)]
pub struct TaskDialogResult {
    pub verified: bool,
    pub action: TaskDialogAction,
    pub progress: Option<u32>,
}

#[derive(Clone)]
pub struct TaskDialogBuilder {
    dialog_title_utf16_nul: Vec<u16>,
    dialog_heading_utf16_nul: Vec<u16>,
    dialog_content_utf16_nul: Vec<u16>,
    dialog_footer_utf16_nul: Vec<u16>,

    dialog_icon: TaskDialogIcon,
    dialog_allow_hyperlinks: bool,

    dialog_verification_text_utf16_nul: Vec<u16>,
    initial_verification_value: bool,

    dialog_buttons: Vec<TaskDialogAction>,

    dialog_progress_source: Option<Arc<AtomicU32>>,
    dialog_progress_min: u16,
    dialog_progress_max: u16,
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

    pub fn set_footer<Param: AsRef<str>>(&mut self, footer: Param) -> &mut Self {
        self.dialog_footer_utf16_nul = str::encode_utf16(footer.as_ref())
            .chain(Some(0))
            .collect::<Vec<u16>>();
        self
    }

    pub fn set_icon(&mut self, icon: TaskDialogIcon) -> &mut Self {
        self.dialog_icon = icon;
        self
    }

    pub fn add_button(&mut self, action: TaskDialogAction) -> &mut Self {
        self.dialog_buttons.push(action);
        self
    }

    /// Adds a checkbox to the bottom of the task dialog. If the checkbox is
    /// checked when the task dialog is dismissed, `TaskDialogResult::verified`
    /// will be set to true. If not, it will be false.
    ///
    /// # Arguments
    ///
    /// * `initial_state` - Determines whether the verification checkbox is
    ///                     checked or unchecked by default.
    pub fn set_verification(&mut self, label: &str, initial_state: bool) -> &mut Self {
        self.initial_verification_value = initial_state;
        self.dialog_verification_text_utf16_nul = str::encode_utf16(label)
            .chain(Some(0))
            .collect::<Vec<u16>>();
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

    /// Setting this will enable a progress bar on the dialog whose progress
    /// value will be read from `source`.
    pub fn set_progress(
        &mut self,
        source: Arc<AtomicU32>,
        value_min: u16,
        value_max: u16,
    ) -> &mut Self {
        self.dialog_progress_source = Some(source);
        self.dialog_progress_min = value_min;
        self.dialog_progress_max = value_max;
        self
    }

    /// Builds the dialog, displays it and returns an instance of it.
    #[allow(clippy::missing_panics_doc)]
    pub fn display(&mut self) -> TaskDialog {
        let dialog_title = Box::new(self.dialog_title_utf16_nul.clone());
        let dialog_heading = Box::new(self.dialog_heading_utf16_nul.clone());
        let dialog_content = Box::new(self.dialog_content_utf16_nul.clone());
        let dialog_footer = Box::new(self.dialog_footer_utf16_nul.clone());
        let dialog_verification_text_utf16_nul =
            Box::new(self.dialog_verification_text_utf16_nul.clone());

        let icon = self.dialog_icon;
        let allow_hyperlinks = self.dialog_allow_hyperlinks;

        let initial_verification_value = self.initial_verification_value;

        let dialog_progress_source = self.dialog_progress_source.clone();

        let mut dialog_common_button_flags = TASKDIALOG_COMMON_BUTTON_FLAGS(0);

        for button in &self.dialog_buttons {
            let flag = button.to_common_button_flag();
            dialog_common_button_flags |= flag;
        }

        let callback_data = Arc::new(Mutex::new(TaskDialogCallbackData {
            dialog_handle: 0,
            dialog_progress_source: dialog_progress_source.clone(),
            dialog_progress_min: 0,
            dialog_progress_max: 100,
        }));

        let mut callback_data_pass = callback_data.clone();

        let task_dialog_result = std::thread::spawn(move || {
            let mut dialog_flags = 0;

            if allow_hyperlinks {
                dialog_flags |= TDF_ENABLE_HYPERLINKS.0;
            }

            if initial_verification_value {
                dialog_flags |= TDF_VERIFICATION_FLAG_CHECKED.0;
            }

            if dialog_progress_source.is_some() {
                dialog_flags |= TDF_CALLBACK_TIMER.0;
                dialog_flags |= TDF_SHOW_PROGRESS_BAR.0;
            }

            let mut dialog_result = TaskDialogResult {
                verified: false,
                action: TaskDialogAction::None,
                progress: None,
            };

            let config = TASKDIALOGCONFIG {
                cbSize: u32::try_from(std::mem::size_of::<TASKDIALOGCONFIG>()).unwrap(),
                hInstance: unsafe { GetModuleHandleA(PCSTR(std::ptr::null())).unwrap().into() },
                pszWindowTitle: PCWSTR(dialog_title.as_ptr()),
                pszMainInstruction: PCWSTR(dialog_heading.as_ptr()),
                pszContent: PCWSTR(dialog_content.as_ptr()),
                pszFooter: PCWSTR(dialog_footer.as_ptr()),
                Anonymous1: TASKDIALOGCONFIG_0 {
                    pszMainIcon: icon.to_icon_id(),
                },
                dwFlags: TASKDIALOG_FLAGS(dialog_flags),
                pfCallback: Some(dialog_notification_callback),
                pszVerificationText: PCWSTR(dialog_verification_text_utf16_nul.as_ptr()),
                dwCommonButtons: dialog_common_button_flags,
                lpCallbackData: std::ptr::from_mut::<Arc<Mutex<TaskDialogCallbackData>>>(
                    &mut callback_data_pass,
                ) as isize,
                ..TASKDIALOGCONFIG::default()
            };

            let mut button_id = 0;
            let mut radio_id = 0;
            let mut verified = BOOL(0);

            unsafe {
                TaskDialogIndirect(
                    &config,
                    Some(&mut button_id),
                    Some(&mut radio_id),
                    Some(&mut verified),
                )
                .unwrap();
            }

            dialog_result.verified = verified.as_bool();

            let cb_data_clone = callback_data_pass.clone();
            let callback_data_guard = (*cb_data_clone).lock().unwrap();

            if let Some(progress) = &callback_data_guard.dialog_progress_source {
                let progress = progress.load(std::sync::atomic::Ordering::SeqCst);
                if progress >= callback_data_guard.dialog_progress_max.into() {
                    dialog_result.progress = Some(progress);
                }
            }

            dialog_result.action = TaskDialogAction::from_id(MESSAGEBOX_RESULT(button_id));

            dialog_result
        });

        TaskDialog {
            // Handle has not yet been initialized by the dialog callback
            // at this point in time, so dialog_handle will forever be 0
            // Should I use a condvar on callback_data and block here?
            cb_data: callback_data.clone(),
            result: task_dialog_result,
        }
    }
}

/// Handles messages for task dialog modals and, if needed, forwards the
/// result of the dialog to the calling thread.
unsafe extern "system" fn dialog_notification_callback(
    hwnd: HWND,
    notification: u32,
    _wparam: WPARAM,
    lparam: LPARAM,
    callback_data: isize,
) -> HRESULT {
    let callback_data_ptr = callback_data as *mut Arc<Mutex<TaskDialogCallbackData>>;
    let mut callback_data_guard = (*callback_data_ptr).lock().unwrap();

    if notification == u32::try_from(TDN_DIALOG_CONSTRUCTED.0).unwrap() {
        // Set HWND in TaskDialogResult passed via callback_data
        callback_data_guard.dialog_handle = hwnd.0;

        // Set progress bar range if relevant
        if (callback_data_guard.dialog_progress_source).is_some() {
            let progress_min = callback_data_guard.dialog_progress_min;
            let progress_max = callback_data_guard.dialog_progress_max;

            // Turn two WORDs (u16) into a single DWORD (u32) as required to
            // create the LPARAM for the TDM_SET_PROGRESS_BAR_RANGE message
            let packed = (u32::from(progress_min)) | u32::from(progress_max) << 16;

            assert!(
                SendMessageA(
                    hwnd,
                    TDM_SET_PROGRESS_BAR_RANGE.0.try_into().unwrap(),
                    WPARAM(0),
                    LPARAM(isize::try_from(packed).unwrap()),
                )
                .0 != 0
            );
        }
    } else if notification == u32::try_from(TDN_TIMER.0).unwrap() {
        // We update the progress bar in timed intervals if it is enabled
        if let Some(progress) = &(callback_data_guard.dialog_progress_source) {
            let progress = progress.load(std::sync::atomic::Ordering::SeqCst);

            PostMessageA(
                hwnd,
                TDM_SET_PROGRESS_BAR_POS.0.try_into().unwrap(),
                WPARAM(progress.try_into().unwrap()),
                LPARAM(0),
            )
            .unwrap();

            // S_OK because S_FALSE resets tick count
            return S_OK;
        }
    } else if notification == u32::try_from(TDN_HYPERLINK_CLICKED.0).unwrap() {
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
    } else if notification == u32::try_from(TDN_BUTTON_CLICKED.0).unwrap() {
        if let Some(progress) = &(callback_data_guard.dialog_progress_source) {
            let progress = progress.load(std::sync::atomic::Ordering::SeqCst);

            if progress >= callback_data_guard.dialog_progress_max.into() {
                // Dialog was closed either by user or programmatically
                // after the task was finished as indicated by the progress bar
                // Since the dialog completed its task before the being closed
                // we override IDCANCEL with IDOK instead when closing.
                EndDialog(hwnd, IDOK.0 as isize).unwrap();
                return S_FALSE;
            }
        }

        return S_OK;
    }

    // Return value is ignored for all other messages so it doesn't matter
    S_OK
}

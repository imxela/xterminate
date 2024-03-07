use windows::core::{PCSTR, PCWSTR};
use windows::Win32::Foundation::{GetLastError, HANDLE, HMODULE, POINT};

use windows::Win32::UI::WindowsAndMessaging::{
    CopyImage, GetCursorPos, LoadImageA, SetSystemCursor, SystemParametersInfoA, HCURSOR,
    IDC_APPSTARTING, IDC_ARROW, IDC_CROSS, IDC_HAND, IDC_HELP, IDC_IBEAM, IDC_NO, IDC_SIZEALL,
    IDC_SIZENESW, IDC_SIZENS, IDC_SIZENWSE, IDC_SIZEWE, IDC_UPARROW, IDC_WAIT, IMAGE_CURSOR,
    IMAGE_FLAGS, LR_LOADFROMFILE, LR_SHARED, OCR_APPSTARTING, OCR_CROSS, OCR_HAND, OCR_HELP,
    OCR_IBEAM, OCR_NO, OCR_NORMAL, OCR_SIZEALL, OCR_SIZENESW, OCR_SIZENS, OCR_SIZENWSE, OCR_SIZEWE,
    OCR_UP, OCR_WAIT, SPI_SETCURSORS, SYSTEM_CURSOR_ID, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS,
};

use crate::error::{AppError, AppResult};
use crate::logf;

pub enum CursorType {
    AppStarting,
    Normal,
    Cross,
    Hand,
    Help,
    IBeam,
    No,
    SizeAll,
    SizeNESW,
    SizeNS,
    SizeNWSE,
    SizeWE,
    Up,
    Wait,
}

/// Returns the position of the cursor.
///
/// # Panics
///
/// This function panics if the internal call to `GetCursorPos()` returns `false`.
#[must_use]
pub fn position() -> (i32, i32) {
    let mut pos = POINT::default();
    assert!(
        unsafe { GetCursorPos(&mut pos).as_bool() },
        "failed to retrieve cursor position (system error {})",
        unsafe { GetLastError().0 }
    );

    (pos.x, pos.y)
}

pub struct Cursor {
    handle: isize,
}

impl Cursor {
    /// Returns the currently active system cursor for the specified cursor type
    ///
    /// # Panics
    ///
    /// Panics if [`LoadImageA`] fails.
    #[must_use]
    pub fn current(cursor_type: &CursorType) -> Self {
        let hcursor = unsafe {
            LoadImageA(
                None,
                std::mem::transmute::<PCWSTR, PCSTR>(idc(cursor_type)),
                IMAGE_CURSOR,
                0,
                0,
                LR_SHARED,
            )
        }
        .unwrap_or_else(|_| {
            panic!("failed to load system cursor (system error {})", unsafe {
                GetLastError().0
            })
        });

        Self { handle: hcursor.0 }
    }

    /// Loads a cursor from the specified file. If the file does not exist
    /// or is not a valid cursor file, [`AppError`] is returned.
    ///
    /// # Errors
    ///
    /// This method returns an error if the underlaying call to [`windows::Windows::Win32::UI::WindowsAndMessaging::LoadImageA`] fails.
    ///
    /// # Panics
    ///
    /// Panics if `filename` can not be turned into a valid [`CString`].
    pub fn load_from_file(filename: &str) -> AppResult<Self> {
        logf!("Loading cursor image from file (path: {})", filename);

        let hcursor = unsafe {
            LoadImageA(
                HMODULE(0),
                PCSTR(
                    std::ffi::CString::new(filename)
                        .unwrap()
                        .as_bytes()
                        .as_ptr(),
                ),
                IMAGE_CURSOR,
                0,
                0,
                LR_LOADFROMFILE,
            )
        };

        match hcursor {
            Ok(v) => Ok(Self { handle: v.0 }),
            Err(e) => Err(AppError::new(
                "failed to load cursor from file",
                unsafe { Some(GetLastError().0 as usize) },
                Some(Box::new(e)),
            )),
        }
    }

    /// Creates a copy of the `self` cursor and returns it.
    ///
    /// # Panics
    ///
    /// This method panics if the internal call to [`CopyImage()`] returns a [`HANDLE`] of value `0`.
    #[must_use]
    pub fn copy(&self) -> Self {
        let cpy = unsafe { CopyImage(HANDLE(self.handle), IMAGE_CURSOR, 0, 0, IMAGE_FLAGS(0)) }
            .unwrap_or_else(|_| {
                panic!("failed to copy image cursor (system error {})", unsafe {
                    GetLastError().0
                })
            });

        Self { handle: cpy.0 }
    }
}

/// Sets the system cursor for the specified cursor type
///
/// # Panics
///
/// This function panics if the internal call to [`SetSystemCursor()`] returns `false`.
pub fn set(cursor_type: &CursorType, cursor: &Cursor) {
    logf!("Setting cursor (idc: {})", idc(cursor_type).0 as isize);

    let success = unsafe { SetSystemCursor(HCURSOR(cursor.handle), ocr(cursor_type)).as_bool() };

    assert!(
        success,
        "{}",
        format!(
            "failed to set system cursor: SetSystemCursor returned 0 (system error {:#08x})",
            unsafe { GetLastError().0 }
        )
    );
}

/// Sets all the system cursor types to the specified cursor
pub fn set_all(cursor: &Cursor) {
    logf!("Setting all cursors");

    // This is terrible but it works
    set(&CursorType::AppStarting, &cursor.copy());
    set(&CursorType::Normal, &cursor.copy());
    set(&CursorType::Cross, &cursor.copy());
    set(&CursorType::Hand, &cursor.copy());
    set(&CursorType::Help, &cursor.copy());
    set(&CursorType::IBeam, &cursor.copy());
    set(&CursorType::No, &cursor.copy());
    set(&CursorType::SizeAll, &cursor.copy());
    set(&CursorType::SizeNESW, &cursor.copy());
    set(&CursorType::SizeNS, &cursor.copy());
    set(&CursorType::SizeNWSE, &cursor.copy());
    set(&CursorType::SizeWE, &cursor.copy());
    set(&CursorType::Up, &cursor.copy());
    set(&CursorType::Wait, &cursor.copy());
}

/// Resets system cursors to Windows the user-defined cursors
///
/// # Panics
///
/// This function panics if the internal call to [`SystemParametersInfoA()`] returns `false`.
pub fn reset() {
    logf!("Resetting cursors");

    let success = unsafe {
        SystemParametersInfoA(
            SPI_SETCURSORS,
            0,
            None,
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        )
        .as_bool()
    };

    assert!(
        success,
        "failed to reset system cursor: SystemPerametersInfoA returned 0 (system error {:#08x})",
        unsafe { GetLastError().0 }
    );
}

/// Converts a [`CursorType`] to a Windows `OCR_` value.
fn ocr(cursor_type: &CursorType) -> SYSTEM_CURSOR_ID {
    match cursor_type {
        CursorType::AppStarting => OCR_APPSTARTING,
        CursorType::Normal => OCR_NORMAL,
        CursorType::Cross => OCR_CROSS,
        CursorType::Hand => OCR_HAND,
        CursorType::Help => OCR_HELP,
        CursorType::IBeam => OCR_IBEAM,
        CursorType::No => OCR_NO,
        CursorType::SizeAll => OCR_SIZEALL,
        CursorType::SizeNESW => OCR_SIZENESW,
        CursorType::SizeNS => OCR_SIZENS,
        CursorType::SizeNWSE => OCR_SIZENWSE,
        CursorType::SizeWE => OCR_SIZEWE,
        CursorType::Up => OCR_UP,
        CursorType::Wait => OCR_WAIT,
    }
}

/// Converts a [`CursorType`] to a Windows `IDC_` value.
fn idc(cursor_type: &CursorType) -> PCWSTR {
    match cursor_type {
        CursorType::AppStarting => IDC_APPSTARTING,
        CursorType::Normal => IDC_ARROW,
        CursorType::Cross => IDC_CROSS,
        CursorType::Hand => IDC_HAND,
        CursorType::Help => IDC_HELP,
        CursorType::IBeam => IDC_IBEAM,
        CursorType::No => IDC_NO,
        CursorType::SizeAll => IDC_SIZEALL,
        CursorType::SizeNESW => IDC_SIZENESW,
        CursorType::SizeNS => IDC_SIZENS,
        CursorType::SizeNWSE => IDC_SIZENWSE,
        CursorType::SizeWE => IDC_SIZEWE,
        CursorType::Up => IDC_UPARROW,
        CursorType::Wait => IDC_WAIT,
    }
}

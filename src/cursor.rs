use windows::Win32::Foundation::{ POINT, HINSTANCE, HANDLE, GetLastError };
use windows::core::{ PCWSTR, PCSTR };

use windows::Win32::UI::WindowsAndMessaging::{ 
    GetCursorPos, 
    LoadImageA,
    CopyImage,
    SetSystemCursor,
    SystemParametersInfoA,

    SYSTEM_CURSOR_ID,
    HCURSOR,
    SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS,

    IMAGE_CURSOR,
    LR_LOADFROMFILE,
    LR_SHARED,
    IMAGE_FLAGS,
    SPI_SETCURSORS,

    OCR_APPSTARTING,
    OCR_NORMAL,
    OCR_CROSS,
    OCR_HAND,
    OCR_HELP,
    OCR_IBEAM,
    OCR_NO,
    OCR_SIZEALL,
    OCR_SIZENESW,
    OCR_SIZENS,
    OCR_SIZENWSE,
    OCR_SIZEWE,
    OCR_UP,
    OCR_WAIT,

    IDC_APPSTARTING,
    IDC_ARROW,
    IDC_CROSS,
    IDC_HAND,
    IDC_HELP,
    IDC_IBEAM,
    IDC_NO,
    IDC_SIZEALL,
    IDC_SIZENESW,
    IDC_SIZENS,
    IDC_SIZENWSE,
    IDC_SIZEWE,
    IDC_UPARROW,
    IDC_WAIT,
};

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
    Wait
}

pub fn position() -> (i32, i32) {
    let mut pos = POINT::default();
    if unsafe { !GetCursorPos(&mut pos).as_bool() } {
        return (-1, -1) // Todo: Error handling
    }

    (pos.x, pos.y)
}

pub struct Cursor {
    handle: isize
}

impl Cursor {
    /// Returns the currently active system cursor for the specified cursor type
    pub fn current(cursor_type: CursorType) -> Self {
        let hcursor = unsafe { 
            LoadImageA(
                None,
                std::mem::transmute::<PCWSTR, PCSTR>(get_idc(cursor_type)),
                IMAGE_CURSOR,
                0,
                0,
                LR_SHARED,
            ).expect(format!("handle this error (code {})", GetLastError().0).as_str())
        };

        Self {
            handle: hcursor.0
        }
    }

    pub fn load(filename: &str) -> Self {
        let hcursor = unsafe {
            LoadImageA(
                HINSTANCE { 0: 0 }, 
                filename, 
                IMAGE_CURSOR, 
                0, 
                0, 
                LR_LOADFROMFILE
            ).expect("handle this error")
        };

        Self {
            handle: hcursor.0
        }
    }

    pub fn copy(&self) -> Self {
        let cpy = unsafe {
            CopyImage(HANDLE(self.handle), IMAGE_CURSOR, 0, 0, IMAGE_FLAGS(0))
        }.expect("handle this error");

        Self {
            handle: cpy.0
        }
    }
}

/// Sets the system cursor for the specified cursor type
pub fn set(cursor_type: CursorType, cursor: &Cursor) {
    let success = unsafe {
        SetSystemCursor(HCURSOR(cursor.handle), get_ocr(cursor_type)).as_bool()
    };

    if !success {
        panic!("handle this (code {})", unsafe { GetLastError().0 });
        // Todo: Handle error
    }
}

/// Sets all the system cursor types to the specified cursor
pub fn set_all(cursor: &Cursor) {
    // This is terrible but it works
    set(CursorType::AppStarting, &cursor.copy());
    set(CursorType::Normal, &cursor.copy());
    set(CursorType::Cross, &cursor.copy());
    set(CursorType::Hand, &cursor.copy());
    set(CursorType::Help, &cursor.copy());
    set(CursorType::IBeam, &cursor.copy());
    set(CursorType::No, &cursor.copy());
    set(CursorType::SizeAll, &cursor.copy());
    set(CursorType::SizeNESW, &cursor.copy());
    set(CursorType::SizeNS, &cursor.copy());
    set(CursorType::SizeNWSE, &cursor.copy());
    set(CursorType::SizeWE, &cursor.copy());
    set(CursorType::Up, &cursor.copy());
    set(CursorType::Wait, &cursor.copy());
}

// Resets system cursors to Windows the user-defined cursors
pub fn reset() {
    let success = unsafe {
        SystemParametersInfoA(SPI_SETCURSORS, 0, std::ptr::null_mut(), SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0)).as_bool()
    };

    if !success {
        panic!("handle this error: failed to reset system cursor, code {}", unsafe { GetLastError().0 });
    }
}

fn get_ocr(cursor_type: CursorType) -> SYSTEM_CURSOR_ID {
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
        CursorType::Wait => OCR_WAIT
    }
}

fn get_idc(cursor_type: CursorType) -> PCWSTR {
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
        CursorType::Wait => IDC_WAIT
    }
}
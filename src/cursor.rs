use windows::Win32::{
    Foundation::POINT,
    UI::WindowsAndMessaging::GetCursorPos
};

pub fn position() -> (i32, i32) {
    let mut pos = POINT::default();
    if unsafe { !GetCursorPos(&mut pos).as_bool() } {
        return (-1, -1) // Todo: Error handling using GetLastError()
    }

    (pos.x, pos.y)
}
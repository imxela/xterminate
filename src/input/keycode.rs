use windows::Win32::UI::WindowsAndMessaging::{  
    RI_MOUSE_BUTTON_1_DOWN,
    RI_MOUSE_BUTTON_1_UP
};

use windows::Win32::UI::Input::KeyboardAndMouse::{
    VIRTUAL_KEY,

    // Keyboard
    VK_LCONTROL,
    VK_LMENU,
    VK_END,

    // Mouse
    VK_LBUTTON
};

#[repr(u16)]
#[derive(Clone, Copy)]
pub enum KeyCode {
    // Keyboard
    LeftControl = VK_LCONTROL.0,
    LeftAlt = VK_LMENU.0,
    End = VK_END.0,

    // Mouse
    LeftMouseButton = VK_LBUTTON.0
}

impl KeyCode {
    /// Converts from a Windows virtual key-code (`VK_XXX`) to a [KeyCode].
    /// 
    /// Currently only implements conversions for KeyCodes used by
    /// xterminator (see [KeyCode]). All other `VK_XXX` values return `None`.
    /// 
    /// For more information about virtual key-codes, refer to the [Windows API documentation](https://docs.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes).
    /// 
    /// # Arguments
    /// 
    /// * `vkey` - The Windows virtual key-code (`VK_XXX`) to convert from
    /// 
    /// # Returns
    /// 
    /// The [KeyCode] equivalent to the specified Windows virtual key-code or
    /// `None` if the specified virtual key-code is not implemented by xterminator (see [KeyCode]).
    pub fn from_vkey(vkey: u16) -> Option<KeyCode> {
        match VIRTUAL_KEY(vkey) {
            // Keyboard
            VK_LCONTROL => Some(KeyCode::LeftControl),
            VK_LMENU => Some(KeyCode::LeftAlt),
            VK_END => Some(KeyCode::End),

            // Mouse
            VK_LBUTTON => Some(KeyCode::LeftMouseButton),

            // The rest are ignored as they won't be used

            _ => None
        }
    }

    /// Converts a Windows `RAWMOUSE::DUMMYUNIONNAME.DUMMYSTRUCTNAME.usButtonFlags`
    /// flag to a [KeyCode].
    /// 
    /// Currently only implements conversions for KeyCodes used by
    /// xterminator (see [KeyCode]). All other `RI_XXX` values return `None`.
    /// 
    /// For information about `usButtonFlags`, refer to the [Windows API documentation](https://docs.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-rawmouse).
    /// 
    /// # Arguments
    /// 
    /// * `ri` - The Windows `RAWMOUSE::DUMMYUNIONNAME.DUMMYSTRUCTNAME.usButtonFlags` flag to convert from
    /// 
    /// # Returns
    /// 
    /// The [KeyCode] equivalent to the specified Windows `RAWMOUSE::DUMMYUNIONNAME.DUMMYSTRUCTNAME.usButtonFlags` 
    /// or `None` if the specified flag is not implemented by xterminator (see [KeyCode]).
    pub fn from_ri(ri: u32) -> Option<KeyCode> {
        match ri {
            RI_MOUSE_BUTTON_1_DOWN => Some(KeyCode::LeftMouseButton),
            RI_MOUSE_BUTTON_1_UP => Some(KeyCode::LeftMouseButton),

            // The rest are ignored as they won't be used

            _ => None
        }
    }
}

impl std::fmt::Display for KeyCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match *self {
            KeyCode::LeftControl => "Left Control",
            KeyCode::LeftAlt => "Left Alt",
            KeyCode::End => "End",

            KeyCode::LeftMouseButton => "Left Mouse Button"
        })
    }
}

// Required for the type to be used as a key in a HashMap
impl std::hash::Hash for KeyCode {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (*self as u32).hash(state);
    }
}

// Required for the type to be used as a key in a HashMap
impl PartialEq for KeyCode {
    fn eq(&self, other: &Self) -> bool {
        *self as u32 == *other as u32
    }
}

impl Eq for KeyCode {}
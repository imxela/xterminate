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
    VK_ESCAPE,
    VK_F4,

    // Mouse
    VK_LBUTTON
};

#[repr(u16)]
#[derive(Clone, Copy, Debug)]
pub enum KeyCode {
    // Keyboard
    LeftControl = VK_LCONTROL.0,
    LeftAlt = VK_LMENU.0,
    End = VK_END.0,
    Escape = VK_ESCAPE.0,

    F4 = VK_F4.0,

    // Mouse
    LeftMouseButton = VK_LBUTTON.0
}

impl KeyCode {
    /// Converts a Windows API virtual key-code `VK_XXX` value to the 
    /// equivalent [KeyCode]. Passing unsupported `VK_XXX` values
    /// (i.e. ones that are not used in xterminate) will return `None`.
    pub fn from_vkey(vkey: u16) -> Option<KeyCode> {
        match VIRTUAL_KEY(vkey) {
            // Keyboard
            VK_LCONTROL => Some(KeyCode::LeftControl),
            VK_LMENU => Some(KeyCode::LeftAlt),
            VK_END => Some(KeyCode::End),
            VK_ESCAPE => Some(KeyCode::Escape),

            VK_F4 => Some(KeyCode::F4),

            // Mouse
            VK_LBUTTON => Some(KeyCode::LeftMouseButton),

            // The rest are ignored as they won't be used

            _ => None
        }
    }

    /// Converts a Windows API raw-input message `RI_XXX` value to the 
    /// equivalent [KeyCode]. Passing unsupported `RI_XXX` values
    /// (i.e. ones that are not used in xterminate) will return `None`.
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
            KeyCode::Escape => "Escape",

            KeyCode::F4 => "F4",

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
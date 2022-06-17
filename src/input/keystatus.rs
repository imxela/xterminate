use windows::Win32::UI::WindowsAndMessaging::{  
    WM_KEYDOWN,
    WM_SYSKEYDOWN,
    WM_KEYUP,
    WM_SYSKEYUP,

    RI_MOUSE_BUTTON_1_DOWN,
    RI_MOUSE_BUTTON_2_DOWN,
    RI_MOUSE_BUTTON_3_DOWN,
    RI_MOUSE_BUTTON_1_UP,
    RI_MOUSE_BUTTON_2_UP,
    RI_MOUSE_BUTTON_3_UP,
    // RI_MOUSE_LEFT_BUTTON_DOWN,
    // RI_MOUSE_RIGHT_BUTTON_DOWN,
    // RI_MOUSE_MIDDLE_BUTTON_DOWN,
    // RI_MOUSE_LEFT_BUTTON_UP,
    // RI_MOUSE_RIGHT_BUTTON_UP,
    // RI_MOUSE_MIDDLE_BUTTON_UP,
    RI_MOUSE_BUTTON_4_DOWN,
    RI_MOUSE_BUTTON_5_DOWN,
    RI_MOUSE_BUTTON_4_UP,
    RI_MOUSE_BUTTON_5_UP
};

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum KeyStatus {
    Released,
    Pressed
}

impl KeyStatus {
    /// Converts a Windows API window-message `WM_XXX` value to the 
    /// equivalent  [KeyStatus]. Passing unsupported `WM_XXX` values
    /// (i.e. ones that are not used in xterminate) will return `None`.
    pub fn from_wm(window_message: u32) -> Option<KeyStatus> {
        match window_message {
            WM_KEYDOWN | WM_SYSKEYDOWN => Some(KeyStatus::Pressed),
            WM_KEYUP | WM_SYSKEYUP => Some(KeyStatus::Released),
            _ => None
        }
    }

    /// Converts a Windows API raw-input message `RI_XXX` value to the 
    /// equivalent  [KeyStatus]. Passing unsupported `RI_XXX` values
    /// (i.e. ones that are not used in xterminate) will return `None`.
    pub fn from_ri(ri: u32) -> Option<KeyStatus> {
        match ri {
            RI_MOUSE_BUTTON_1_DOWN => Some(KeyStatus::Pressed),
            RI_MOUSE_BUTTON_2_DOWN => Some(KeyStatus::Pressed),
            RI_MOUSE_BUTTON_3_DOWN => Some(KeyStatus::Pressed),

            RI_MOUSE_BUTTON_1_UP => Some(KeyStatus::Released),
            RI_MOUSE_BUTTON_2_UP => Some(KeyStatus::Released),
            RI_MOUSE_BUTTON_3_UP => Some(KeyStatus::Released),

            // These are are defined with the same value as RI_MOUSE_BUTTON_X_DOWN and are therefore unnecessary
            // RI_MOUSE_LEFT_BUTTON_DOWN   => Some(KeyStatus::Pressed),
            // RI_MOUSE_RIGHT_BUTTON_DOWN  => Some(KeyStatus::Pressed),
            // RI_MOUSE_MIDDLE_BUTTON_DOWN => Some(KeyStatus::Pressed),

            // These are are defined with the same value as RI_MOUSE_BUTTON_X_UP and are therefore unnecessary
            // RI_MOUSE_LEFT_BUTTON_UP     => Some(KeyStatus::Released),
            // RI_MOUSE_RIGHT_BUTTON_UP    => Some(KeyStatus::Released),
            // RI_MOUSE_MIDDLE_BUTTON_UP   => Some(KeyStatus::Released),

            RI_MOUSE_BUTTON_4_DOWN => Some(KeyStatus::Pressed),
            RI_MOUSE_BUTTON_5_DOWN => Some(KeyStatus::Pressed),
            RI_MOUSE_BUTTON_4_UP   => Some(KeyStatus::Released),
            RI_MOUSE_BUTTON_5_UP   => Some(KeyStatus::Released),
            
            // The rest are ignored as they won't be used
            _ => None
        }
    }
}
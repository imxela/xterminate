/// Automatically generates the [KeyCode] enum, populates it with the
/// specified key-codes, and implement conversion functions for them.
/// In order for xterminate to recognize any keyboard or mouse button
/// it must first be defined in a call to this macro.
/// 
/// # Arguments
/// 
/// * `enum` - The name of the KeyCode used for the enum value
/// * `identifier` - The virtual-key code message (`VK_`) corresponding to the KeyCode
/// * `string` - A pretty-print string literal for the KeyCode
/// * `ri_identifier` - An optional list of raw-input messages (`RI_`) corresponding to the KeyCode.
/// 
/// # Notes
/// 
/// The `ri_identifier` parameter must only be specified for virtual-key codes whose
/// constants differ when using raw-input. One such example is the left mouse button
/// which normally generates the `VK_LBUTTON` message but in raw-input mode instead
/// generates `RI_MOUSE_BUTTON_1_DOWN` and `RI_MOUSE_BUTTON_1_UP` messages. By
/// specifying these, the macro implements conversion functions associated with
/// the specified `enum` so the `RI_` messages can be converted to `VK_` where needed.
macro_rules! generate_keycodes {
    ($(($enum:tt, $identifier:ident, $string:literal, $print:literal)),*) => {
        use windows::Win32::UI::Input::KeyboardAndMouse::{
            VIRTUAL_KEY,

            $($identifier,)*
        };

        as_item! {
            #[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
            pub enum KeyCode { 
                $($enum = $identifier.0 as isize,)* 
            }
        }

        impl KeyCode {
            /// Constructs a new KeyCode from a virtual-key code (`VK_`) message.
            /// A list of possible constants this function accepts can be found
            /// [here](https://docs.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes).
            pub fn from_vkey(vkey: u16) -> Option<KeyCode> {
                match VIRTUAL_KEY(vkey) {
                    _ => None
                }
            }

            /// Constructs a new KeyCode from a raw-input (`RI_`) message.
            /// A list of possible constants this function accepts can be found 
            /// [here](https://docs.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-rawmouse)
            /// under the `DUMMYUNIONNAME.DUMMYSTRUCTNAME.usButtonFlags` section.
            pub fn from_ri(ri: u32) -> Option<KeyCode> {
                match ri {
                    _ => None
                }
            }

            /// Construct a KeyCode from a string representation of a virtual key-code message (`VK`).
            /// List of possible virtual-key code (VK_) constants can be found
            /// [here](https://docs.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes).
            pub fn from_string(string: &str) -> Option<KeyCode> {
                match string {
                    $($string => Some(KeyCode::$enum),)*

                    _ => None
                }
            }
        }

        impl std::fmt::Display for KeyCode {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", match *self {
                    $(KeyCode::$enum => $print,)*
                })
            }
        }
    };

    ($(($enum:tt, $identifier:ident, $print:literal $(, $($ri_identifier:ident),*)?)),*) => {
        use windows::Win32::UI::Input::KeyboardAndMouse::{
            VIRTUAL_KEY,

            $($identifier,)*
        };

        use windows::Win32::UI::WindowsAndMessaging::{  
            $($($($ri_identifier,)*)?)*
        };

        as_item! {
            #[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
            pub enum KeyCode { 
                $($enum = $identifier.0 as isize,)* 
            }
        }

        impl KeyCode {
            /// Constructs a new KeyCode from a virtual-key code (`VK_`) message.
            /// A list of possible constants this function accepts can be found
            /// [here](https://docs.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes).
            pub fn from_vkey(vkey: u16) -> Option<KeyCode> {
                match VIRTUAL_KEY(vkey) {
                    $($identifier => Some(KeyCode::$enum),)*

                    _ => None
                }
            }

            /// Constructs a new KeyCode from a raw-input (`RI_`) message.
            /// A list of possible constants this function accepts can be found 
            /// [here](https://docs.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-rawmouse)
            /// under the `DUMMYUNIONNAME.DUMMYSTRUCTNAME.usButtonFlags` section.
            pub fn from_ri(ri: u32) -> Option<KeyCode> {
                match ri {
                   $($($($ri_identifier => Some(KeyCode::$enum),)*)?)*

                    _ => None
                }
            }

            /// Construct a KeyCode from a string representation of a virtual key-code message (`VK`).
            /// List of possible virtual-key code (VK_) constants can be found
            /// [here](https://docs.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes).
            pub fn from_string(string: &str) -> Option<KeyCode> {
                match string {
                    $(stringify!($identifier) => Some(KeyCode::$enum),)*

                    _ => None
                }
            }
        }

        impl std::fmt::Display for KeyCode {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", match *self {
                    $(KeyCode::$enum => $print,)*
                })
            }
        }
    };
}

macro_rules! as_item {
    ($i:item) => { $i };
}

// Implements virtual-key codes listed in the Microsoft virtual-key code documentation:
// https://docs.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes
generate_keycodes!(
    (LeftMouseButton, VK_LBUTTON, "Left Mouse Button", RI_MOUSE_BUTTON_1_DOWN, RI_MOUSE_BUTTON_1_UP),
    (RightMouseButton, VK_RBUTTON, "Right Mouse Button", RI_MOUSE_BUTTON_2_DOWN, RI_MOUSE_BUTTON_2_UP),
    (Cancel, VK_CANCEL, "Cancel"), // Control-break processing?
    (MiddleMouseButton, VK_MBUTTON, "Middle Mouse Button", RI_MOUSE_BUTTON_3_DOWN, RI_MOUSE_BUTTON_3_UP),
    (MouseButton4, VK_XBUTTON1, "Mouse Button 4", RI_MOUSE_BUTTON_4_DOWN, RI_MOUSE_BUTTON_4_UP),
    (MouseButton5, VK_XBUTTON2, "Mouse Button 5", RI_MOUSE_BUTTON_5_DOWN, RI_MOUSE_BUTTON_5_UP),
    (Backspace, VK_BACK, "Backspace"),
    (Tab, VK_TAB, "Tab"),
    (Clear, VK_CLEAR, "Clear"),
    (Return, VK_RETURN, "Enter"),
    (Shift, VK_SHIFT, "Shift"),
    (Control, VK_CONTROL, "Control"),
    (Alt, VK_MENU, "Alt"),
    (Pause, VK_PAUSE, "Pause"),
    (Caps, VK_CAPITAL, "Caps-lock"),
    (IMEKana, VK_KANA, "IME Kana Mode"),
    // (IMEHangul, VK_HANGUL, "IME Hangul Mode"), // Duplicate
    // (IMEHanguel, VK_HANGUEL, "IME Hanguel Mode"), // Duplicate
    (IMEOn, VK_IME_ON, "IME On"),
    (IMEJunja, VK_JUNJA, "IME Junja Mode"),
    (IMEFinal, VK_FINAL, "IME Final Mode"),
    // (IMEHanja, VK_HANJA, "IME Hanja Mode"), // Duplicate
    (IMEKanji, VK_KANJI, "IME Kanji Mode"),
    (IMEOff, VK_IME_OFF, "IME Off"),
    (Escape, VK_ESCAPE, "Escape"),
    (IMEConvert, VK_CONVERT, "IME Convert"),
    (IMENonConvert, VK_NONCONVERT, "IME Non-Convert"),
    (IMEAccept, VK_ACCEPT, "IME Accept"),
    (IMEModeChange, VK_MODECHANGE, "IME Mode Change Request"),
    (Space, VK_SPACE, "Spacebar"),
    (PageUp, VK_PRIOR, "Page-Up"),
    (PageDown, VK_NEXT, "Page-Down"),
    (End, VK_END, "End"),
    (Home, VK_HOME, "Home"),
    (Left, VK_LEFT, "Left Arrow"),
    (Up, VK_UP, "Up Arrow"),
    (Right, VK_RIGHT, "Right Arrow"),
    (Down, VK_DOWN, "Down Arrow"),
    (Select, VK_SELECT, "Select"),
    (Print, VK_PRINT, "Print"),
    (Execute, VK_EXECUTE, "Execute"),
    (PrintScreen, VK_SNAPSHOT, "Print-Screen"),
    (Insert, VK_INSERT, "Insert"),
    (Delete, VK_DELETE, "Delete"),
    (Help, VK_HELP, "Help"),
    (Alpha0, VK_0, "Alpha 0"),
    (Alpha1, VK_1, "Alpha 1"),
    (Alpha2, VK_2, "Alpha 2"),
    (Alpha3, VK_3, "Alpha 3"),
    (Alpha4, VK_4, "Alpha 4"),
    (Alpha5, VK_5, "Alpha 5"),
    (Alpha6, VK_6, "Alpha 6"),
    (Alpha7, VK_7, "Alpha 7"),
    (Alpha8, VK_8, "Alpha 8"),
    (Alpha9, VK_9, "Alpha 9"),
    (A, VK_A, "A"),
    (B, VK_B, "B"),
    (C, VK_C, "C"),
    (D, VK_D, "D"),
    (E, VK_E, "E"),
    (F, VK_F, "F"),
    (G, VK_G, "G"),
    (H, VK_H, "H"),
    (I, VK_I, "I"),
    (J, VK_J, "J"),
    (K, VK_K, "K"),
    (L, VK_L, "L"),
    (M, VK_M, "M"),
    (N, VK_N, "N"),
    (O, VK_O, "O"),
    (P, VK_P, "P"),
    (Q, VK_Q, "Q"),
    (R, VK_R, "R"),
    (S, VK_S, "S"),
    (T, VK_T, "T"),
    (U, VK_U, "U"),
    (V, VK_V, "V"),
    (W, VK_W, "W"),
    (X, VK_X, "X"),
    (Y, VK_Y, "Y"),
    (Z, VK_Z, "Z"),
    (LeftWin, VK_LWIN, "Left Windows"),
    (RightWin, VK_RWIN, "Right Windows"),
    (Apps, VK_APPS, "Applications"),
    (Sleep, VK_SLEEP, "Sleep"),
    (Numpad0, VK_NUMPAD0, "Numpad 0"),
    (Numpad1, VK_NUMPAD1, "Numpad 1"),
    (Numpad2, VK_NUMPAD2, "Numpad 2"),
    (Numpad3, VK_NUMPAD3, "Numpad 3"),
    (Numpad4, VK_NUMPAD4, "Numpad 4"),
    (Numpad5, VK_NUMPAD5, "Numpad 5"),
    (Numpad6, VK_NUMPAD6, "Numpad 6"),
    (Numpad7, VK_NUMPAD7, "Numpad 7"),
    (Numpad8, VK_NUMPAD8, "Numpad 8"),
    (Numpad9, VK_NUMPAD9, "Numpad 9"),
    (Multiply, VK_MULTIPLY, "Multiply"),
    (Add, VK_ADD, "Add"),
    (Separator, VK_SEPARATOR, "Separator"),
    (Subtract, VK_SUBTRACT, "Subtract"),
    (Decimal, VK_DECIMAL, "Decimal"),
    (Divide, VK_DIVIDE, "Divide"),
    (F1, VK_F1, "F1"),
    (F2, VK_F2, "F2"),
    (F3, VK_F3, "F3"),
    (F4, VK_F4, "F4"),
    (F5, VK_F5, "F5"),
    (F6, VK_F6, "F6"),
    (F7, VK_F7, "F7"),
    (F8, VK_F8, "F8"),
    (F9, VK_F9, "F9"),
    (F10, VK_F10, "F10"),
    (F11, VK_F11, "F11"),
    (F12, VK_F12, "F12"),
    (F13, VK_F13, "F13"),
    (F14, VK_F14, "F14"),
    (F15, VK_F15, "F15"),
    (F16, VK_F16, "F16"),
    (F17, VK_F17, "F17"),
    (F18, VK_F18, "F18"),
    (F19, VK_F19, "F19"),
    (F20, VK_F20, "F20"),
    (F21, VK_F21, "F21"),
    (F22, VK_F22, "F22"),
    (F23, VK_F23, "F23"),
    (F24, VK_F24, "F24"),
    (NumLock, VK_NUMLOCK, "Num Lock"),
    (ScrollLock, VK_SCROLL, "Scroll Lock"),
    (LeftShift, VK_LSHIFT, "Left Shift"),
    (RightShift, VK_RSHIFT, "Right Shift"),
    (LeftControl, VK_LCONTROL, "Left Control"),
    (RightControl, VK_RCONTROL, "RightControl"),
    (LeftAlt, VK_LMENU, "Left Alt"),
    (RightAlt, VK_RMENU, "Right Alt"),
    (BrowserBack, VK_BROWSER_BACK, "Browser Back"),
    (BrowserForward, VK_BROWSER_FORWARD, "Browser Forward"),
    (BrowserRefresh, VK_BROWSER_REFRESH, "Browser Refresh"),
    (BrowserStop, VK_BROWSER_STOP, "Browser Stop"),
    (BrowserSearch, VK_BROWSER_SEARCH, "Browser Search"),
    (BrowserFavorites, VK_BROWSER_FAVORITES, "Browser Favorites"),
    (BrowserHome, VK_BROWSER_HOME, "Browser Home"),
    (VolumeMute, VK_VOLUME_MUTE, "Volume Mute"),
    (VolumeDown, VK_VOLUME_DOWN, "Volume Down"),
    (VolumeUp, VK_VOLUME_UP, "Volume Up"),
    (MediaNextTrack, VK_MEDIA_NEXT_TRACK, "Media Next Track"),
    (MediaPreviousTrack, VK_MEDIA_PREV_TRACK, "Media Previous Track"),
    (MediaStop, VK_MEDIA_STOP, "Stop Media"),
    (MediaPlayPause, VK_MEDIA_PLAY_PAUSE, "Play/Pause Media"),
    (Mail, VK_LAUNCH_MAIL, "Mail"),
    (MediaSelect, VK_LAUNCH_MEDIA_SELECT, "Media Select"),
    (Application1, VK_LAUNCH_APP1, "Application 1"),
    (Application2, VK_LAUNCH_APP2, "Application 2"),
    (Colon, VK_OEM_1, "Colon (;:) (US layout)"),
    (Plus, VK_OEM_PLUS, "Plus (+) (US layout)"),
    (Comma, VK_OEM_COMMA, "Comma (,) (US layout)"),
    (Minus, VK_OEM_MINUS, "Minus (-) (US layout)"),
    (Period, VK_OEM_PERIOD, "Period (.) /US layout)"),
    (Forwardslash, VK_OEM_2, "Forwardslash (/?) (US layout)"),
    (Squiggly, VK_OEM_3, "Squiggly (`~) (US layout)"),
    (OpenBracket, VK_OEM_4, "Open Bracket ([{) (US layout)"),
    (Backslash, VK_OEM_5, "Backslash (\\|) (US layout)"),
    (CloseBracket, VK_OEM_6, "Closed Bracket (]}) (US layout)"),
    (Quote, VK_OEM_7, "Quote (\'\") (US layout)"),
    (Exclamation, VK_OEM_8, "Exclamation (§!) (US layout))"),
    (ArrowBracket, VK_OEM_102, "Arrow Brackets (<>) (US layout)"),
    (IMEProcess, VK_PROCESSKEY, "IME Process"),
    (Unicode, VK_PACKET, "Unicode Character"),
    (Attn, VK_ATTN, "Attn"),
    (CrSel, VK_CRSEL, "CrSel"),
    (ExSel, VK_EXSEL, "ExSel"),
    (ErEOF, VK_EREOF, "ErEOF"),
    (Play, VK_PLAY, "Play"),
    (Zoom, VK_ZOOM, "Zoom"),
    (NoName, VK_NONAME, "No Name"),
    (PA1, VK_PA1, "PA1"),
    (OEMClear, VK_OEM_CLEAR, "Clear")
);
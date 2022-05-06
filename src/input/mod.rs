pub mod keycode;
pub mod keystatus;

pub use keycode::KeyCode;
pub use keystatus::KeyStatus;

use windows::Win32::UI::Input::{
    RAWINPUTDEVICE,
    RAWINPUTHEADER,
    RAWINPUT,
    
    RIDEV_INPUTSINK,
    RIDEV_NOLEGACY,
    RID_INPUT,
    RIDEV_REMOVE,
    
    RegisterRawInputDevices,
    GetRawInputData
};

use windows::Win32::UI::WindowsAndMessaging::{
    WNDCLASSA,
    HMENU,
    MSG,
    
    HWND_MESSAGE,
    WM_CREATE,
    WM_INPUT,
    MAPVK_VSC_TO_VK_EX,

    RegisterClassA,
    CreateWindowExA,
    GetMessageA,
    TranslateMessage,
    DispatchMessageA,
    DefWindowProcA
};

use windows::Win32::System::LibraryLoader::GetModuleHandleA;

use windows::core::PCSTR;

use windows::Win32::Foundation::{
    HWND,
    WPARAM,
    LPARAM,
    LRESULT,
    
    GetLastError
};

use windows::Win32::UI::Input::{
    HRAWINPUT,
    RAWKEYBOARD,
    RAWMOUSE,
    RID_DEVICE_INFO_TYPE,
    
    RIM_TYPEKEYBOARD,
    RIM_TYPEMOUSE,
    RIM_TYPEHID
};

use windows::Win32::UI::Input::KeyboardAndMouse::MapVirtualKeyW;

use windows::Win32::Devices::HumanInterfaceDevice::{
    HID_USAGE_PAGE_GENERIC,
    HID_USAGE_GENERIC_KEYBOARD,
    HID_USAGE_GENERIC_MOUSE
};

use std::collections::HashMap;

static mut KEYS: Option<HashMap<KeyCode, KeyStatus>> = None;
static mut INSTANCE: Option<Input> = None;

pub struct KeyState {
}

impl KeyState {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get(&self, keycode: KeyCode) -> KeyStatus {
        get_key_state(keycode)
    }

    pub fn released(&self, keycode: KeyCode) -> bool {
        self.get(keycode) == KeyStatus::Released
    }

    pub fn pressed(&self, keycode: KeyCode) -> bool {
        self.get(keycode) == KeyStatus::Pressed
    }
}

pub struct Input {
    callback: fn(&mut crate::app::App, KeyState, KeyCode, KeyStatus) -> bool
}

impl Input {
    pub fn poll(callback: fn(&mut crate::app::App, KeyState, KeyCode, KeyStatus) -> bool) { unsafe {
        let mut wndclass = WNDCLASSA::default();
        wndclass.hInstance = GetModuleHandleA(PCSTR(std::ptr::null())); // Equivalent to the hInstance parameter passed to WinMain in C/C++
        wndclass.lpszClassName = PCSTR(String::from("xterminatorwcname").as_mut_ptr());
        wndclass.lpfnWndProc = Some(raw_input_callback);
        
        RegisterClassA(&wndclass);

        {
            Input::set_instance(Self {
                callback
            });
        }
                
        let hwnd = CreateWindowExA(
            Default::default(),
            wndclass.lpszClassName,
            PCSTR(std::ptr::null()),
            Default::default(),
            0,
            0,
            0,
            0,
            HWND_MESSAGE,
            HMENU(0),
            wndclass.hInstance,
            std::ptr::null()
        );

        if hwnd.0 == 0 {
            panic!("window creation failed: CreateWindowExA() returned NULL (os error code {})", GetLastError().0);
        }

        KEYS = Some(HashMap::new());

        let mut message = MSG::default();
        while GetMessageA(&mut message, hwnd, 0, 0).as_bool() {
            TranslateMessage(&message);
            DispatchMessageA(&message);
        }
    } }

    fn set_instance(input: Input) {
        unsafe { INSTANCE = Some(input) };
    }

    fn instance() -> &'static mut Option<Input> {
        unsafe { &mut INSTANCE }
    }
}

fn set_key_state(keycode: KeyCode, keystatus: KeyStatus) { unsafe {
    *KEYS.as_mut().unwrap().entry(keycode).or_insert(keystatus) = keystatus;
} }

fn get_key_state(keycode: KeyCode) -> KeyStatus { unsafe {
    *KEYS.as_mut().unwrap().entry(keycode).or_insert(KeyStatus::Released)
} }

fn process_keyboard_input(keyboard: &RAWKEYBOARD) -> Option<(KeyCode, KeyStatus)> {
    // Maps the scancode to a virtual keycode that differentiates between left/right
    // versions of certain keys (such as L/R control, shift, alt, etc). The VKey value in
    // 'RAWINPUT::data::keyboard::VKey' doesn't differentiate them, but by using the
    // 'MapVirtualKeyW' function the scancode can be mapped to a virtual key-code that does.
    let vkey = unsafe { MapVirtualKeyW(keyboard.MakeCode as u32, MAPVK_VSC_TO_VK_EX) };

    let keycode = KeyCode::from_vkey(vkey as u16)?;
    let keystatus = KeyStatus::from_wm(keyboard.Message)?;

    Some((keycode, keystatus))
}

fn process_mouse_input(mouse: &RAWMOUSE) -> Option<(KeyCode, KeyStatus)> {
    let ri = unsafe { mouse.Anonymous.Anonymous.usButtonFlags };

    let keycode = KeyCode::from_ri(ri as u32)?;
    let keystatus = KeyStatus::from_ri(ri as u32)?;

    Some((keycode, keystatus))
}

unsafe extern "system" fn raw_input_callback(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_CREATE => {
            let mut devices = [RAWINPUTDEVICE::default(); 2];

            let keyboard_device = &mut devices[0];
            keyboard_device.dwFlags = RIDEV_NOLEGACY | RIDEV_INPUTSINK;
            keyboard_device.usUsagePage = HID_USAGE_PAGE_GENERIC;
            keyboard_device.usUsage = HID_USAGE_GENERIC_KEYBOARD;
            keyboard_device.hwndTarget = hwnd;
            

            let mouse_device = &mut devices[1];
            mouse_device.dwFlags = RIDEV_NOLEGACY | RIDEV_INPUTSINK;
            mouse_device.usUsagePage = HID_USAGE_PAGE_GENERIC;
            mouse_device.usUsage = HID_USAGE_GENERIC_MOUSE;
            mouse_device.hwndTarget = hwnd;

            RegisterRawInputDevices(&devices, std::mem::size_of::<RAWINPUTDEVICE>() as u32)
                .expect(format!("RegisterRawInputDevices() failed: {}", GetLastError().0).as_str());

            // LRESULT with a value of 0 signals Windows to continue with window creation
            return LRESULT(0);
        },

        WM_INPUT => {
            let mut dwsize = u32::default();

            if GetRawInputData(
                std::mem::transmute::<LPARAM, HRAWINPUT>(lparam),
                RID_INPUT,
                std::ptr::null_mut(),
                &mut dwsize,
                std::mem::size_of::<RAWINPUTHEADER>() as u32
            ) == std::mem::transmute::<i32, u32>(-1) {
                panic!("first call to GetRawInputData() failed: {}", GetLastError().0)
            }

            let mut data = vec![0; dwsize as usize];
            let data = data.as_mut_ptr() as *mut std::ffi::c_void;

            if GetRawInputData(
                std::mem::transmute::<LPARAM, HRAWINPUT>(lparam),
                RID_INPUT,
                data,
                &mut dwsize,
                std::mem::size_of::<RAWINPUTHEADER>() as u32

            ) != dwsize {
                panic!("second call to GetRawInputData() failed: {}", GetLastError().0)
            }

            let rawinput = data as *mut RAWINPUT;
            let keystate: Option<(KeyCode, KeyStatus)>;

            match RID_DEVICE_INFO_TYPE((*rawinput).header.dwType) {
                RIM_TYPEKEYBOARD => {
                    keystate = process_keyboard_input(&(*rawinput).data.keyboard);
                },

                RIM_TYPEMOUSE => {
                    keystate = process_mouse_input(&(*rawinput).data.mouse);
                },

                RIM_TYPEHID => {
                    // HID messages are unused by the application so just ignore and pass them on
                    return DefWindowProcA(hwnd, msg, wparam, lparam);
                },

                _ => {
                    // Should (knock on wood) be impossible since 'dwType' can only
                    // be any of the above three values acording to Windows docs
                    // Todo: Handle this error
                    panic!("unexpected branching: 'RAWINPUT::header::dwType' contains a value that is not 'RIM_TYPEKEYBOARD', 'RIM_TYPEMOUSE' or 'RIM_TYPEHID'");
                }
            }

            if keystate.is_none() {
                // Unsupported KeyStatus or KeyCode
                // the application does not care about.
                // Pass on message and do nothing.
                return DefWindowProcA(hwnd, msg, wparam, lparam);
            }

            let (keycode, keystatus) = keystate.unwrap();
            set_key_state(keycode, keystatus);

            // Callback determines whether the input message
            // should be consumed or not via its return value.
            // If true, return LRESULT of value 0 to indicate so.
            let should_consume = (Input::instance().as_ref().unwrap().callback)(&mut *crate::app::App::instance(), KeyState::new(), keycode, keystatus);
            if should_consume {
                return LRESULT(0);
            }

            return DefWindowProcA(hwnd, msg, wparam, lparam);
        }

        _ => {
            // Any other message kind can be ignored and passed on
            return DefWindowProcA(hwnd, msg, wparam, lparam);
        }
    }
}

/// This should not be called manually as the application
/// unregisters by itself. This is only ever called if the
/// application panics in order to switch to the system-
/// default message handling.
pub fn unregister() {
    let mut devices = [RAWINPUTDEVICE::default(); 2];

    let keyboard_device = &mut devices[0];
    keyboard_device.dwFlags = RIDEV_REMOVE;
    keyboard_device.usUsagePage = HID_USAGE_PAGE_GENERIC;
    keyboard_device.usUsage = HID_USAGE_GENERIC_KEYBOARD;
    keyboard_device.hwndTarget = HWND(0);
    

    let mouse_device = &mut devices[1];
    mouse_device.dwFlags = RIDEV_REMOVE;
    mouse_device.usUsagePage = HID_USAGE_PAGE_GENERIC;
    mouse_device.usUsage = HID_USAGE_GENERIC_MOUSE;
    mouse_device.hwndTarget = HWND(0);

    unsafe {
        RegisterRawInputDevices(&devices, std::mem::size_of::<RAWINPUTDEVICE>() as u32)
        .expect(format!("RegisterRawInputDevices() failed: {}", GetLastError().0).as_str());
    }
}
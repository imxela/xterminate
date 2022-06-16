pub mod keycode;
pub mod keystatus;

pub use keycode::KeyCode;
pub use keystatus::KeyStatus;

use windows::Win32::UI::Input::{
    RAWINPUTDEVICE,
    RAWINPUTHEADER,
    RAWINPUT,
    
    RIDEV_INPUTSINK,
    RID_INPUT,
    RIDEV_REMOVE,
    
    RegisterRawInputDevices,
    GetRawInputData,
};

use windows::Win32::UI::WindowsAndMessaging::{
    WNDCLASSA,
    HMENU,
    MSG,
    
    HWND_MESSAGE,
    WM_CREATE,
    WM_DESTROY,
    WM_INPUT,
    MAPVK_VSC_TO_VK_EX,
    PM_REMOVE,
    GWLP_USERDATA,

    RegisterClassA,
    CreateWindowExA,
    PeekMessageA,
    TranslateMessage,
    DispatchMessageA,
    DefWindowProcA,
    DestroyWindow, 
    SetWindowLongPtrA, 
    GetWindowLongPtrA
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

use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
 
// Todo: Make this not static?
static mut KEYS: Option<HashMap<KeyCode, KeyStatus>> = None;

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

pub trait InputEventHandler {
    fn handle(&mut self, state: KeyState, keycode: KeyCode, keystatus: KeyStatus) -> bool;
}

pub struct Input {
    hwnd: HWND,
    event_handler: Rc<RefCell<dyn InputEventHandler>>
}

impl Input {
    pub fn create(event_handler: Rc<RefCell<dyn InputEventHandler>>) -> Self { unsafe {
        let mut wndclass = WNDCLASSA::default();
        wndclass.hInstance = GetModuleHandleA(PCSTR(std::ptr::null())); // Equivalent to the hInstance parameter passed to WinMain in C/C++
        wndclass.lpszClassName = PCSTR(String::from("xterminatorwcname").as_mut_ptr());
        wndclass.lpfnWndProc = Some(raw_input_callback);
        
        RegisterClassA(&wndclass);
                
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

        let mut instance = Self {
            hwnd,
            event_handler
        };

        SetWindowLongPtrA(hwnd, GWLP_USERDATA, &mut instance as *mut Input as isize);

        KEYS = Some(HashMap::new());

        instance
    }}

    pub fn poll(&self) { unsafe {
        let mut message = MSG::default();
        if PeekMessageA(&mut message, self.hwnd, 0, 0, PM_REMOVE).as_bool()
        {
            TranslateMessage(&message);
            DispatchMessageA(&message);
        }
    }}

    pub fn unregister(&self) { unsafe {
        // DestroyWindow triggers the WM_DESTROY message in the
        // WndProc handler above, which in turn unregiosters the
        // raw input devices.
        DestroyWindow(self.hwnd);
    }}
}

fn set_key_state(keycode: KeyCode, keystatus: KeyStatus) { unsafe {
    *KEYS.as_mut().unwrap().entry(keycode).or_insert(keystatus) = keystatus;
} }

fn get_key_state(keycode: KeyCode) -> KeyStatus { unsafe {
    *KEYS.as_mut().unwrap().entry(keycode).or_insert(KeyStatus::Released)
} }

/// Processes raw mouse input (`RAWKEYBOARD`) into a universal `KeyStatus`.
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

/// Processes raw mouse input (`RAWMOUSE`) into a universal `KeyStatus`.
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
            keyboard_device.dwFlags = RIDEV_INPUTSINK;
            keyboard_device.usUsagePage = HID_USAGE_PAGE_GENERIC;
            keyboard_device.usUsage = HID_USAGE_GENERIC_KEYBOARD;
            keyboard_device.hwndTarget = hwnd;
            

            let mouse_device = &mut devices[1];
            mouse_device.dwFlags = RIDEV_INPUTSINK;
            mouse_device.usUsagePage = HID_USAGE_PAGE_GENERIC;
            mouse_device.usUsage = HID_USAGE_GENERIC_MOUSE;
            mouse_device.hwndTarget = hwnd;
            
            RegisterRawInputDevices(&devices, std::mem::size_of::<RAWINPUTDEVICE>() as u32)
                .expect(format!("RegisterRawInputDevices() failed: {}", GetLastError().0).as_str());

            return LRESULT(0);
        },

        WM_DESTROY => {
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
        
            RegisterRawInputDevices(&devices, std::mem::size_of::<RAWINPUTDEVICE>() as u32)
                .expect(format!("RegisterRawInputDevices() failed: {}", GetLastError().0).as_str());

            return LRESULT(0)
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

            let instance = GetWindowLongPtrA(hwnd, GWLP_USERDATA) as *mut Input;
            let handler = &mut instance.as_mut().unwrap().event_handler;

            // Callback determines whether the input message
            // should be consumed or not via its return value.
            // If true, return LRESULT of value 0 to indicate so.
            let consume = handler.as_ref().borrow_mut().handle(KeyState::new(), keycode, keystatus);
            if consume {
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
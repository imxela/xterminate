pub mod keybind;
pub mod keycode;
pub mod keystatus;

pub use keybind::Keybind;
pub use keycode::KeyCode;
pub use keystatus::KeyStatus;

use crate::logf;

use windows::Win32::UI::Input::{
    GetRawInputData, RegisterRawInputDevices, RAWINPUT, RAWINPUTDEVICE, RAWINPUTHEADER,
    RIDEV_INPUTSINK, RIDEV_REMOVE, RID_INPUT,
};

use windows::Win32::UI::Input::KeyboardAndMouse::MAPVK_VSC_TO_VK_EX;

use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExA, DefWindowProcA, DestroyWindow, DispatchMessageA, GetWindowLongPtrW,
    PeekMessageA, RegisterClassExA, SetWindowLongPtrW, TranslateMessage, GWLP_USERDATA, HMENU,
    HWND_MESSAGE, MSG, PM_REMOVE, WINDOW_EX_STYLE, WINDOW_STYLE, WM_CREATE, WM_DESTROY, WM_INPUT,
    WNDCLASSEXA,
};

use windows::Win32::System::LibraryLoader::GetModuleHandleA;

use windows::core::PCSTR;

use windows::Win32::Foundation::{GetLastError, HWND, LPARAM, LRESULT, WPARAM};

use windows::Win32::UI::Input::{
    HRAWINPUT, RAWKEYBOARD, RAWMOUSE, RID_DEVICE_INFO_TYPE, RIM_TYPEKEYBOARD, RIM_TYPEMOUSE,
};

use windows::Win32::UI::Input::KeyboardAndMouse::MapVirtualKeyW;

use windows::Win32::Devices::HumanInterfaceDevice::{
    HID_USAGE_GENERIC_KEYBOARD, HID_USAGE_GENERIC_MOUSE, HID_USAGE_PAGE_GENERIC,
};

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone)]
pub struct KeyState {
    keys: Rc<RefCell<HashMap<KeyCode, KeyStatus>>>,
}

impl KeyState {
    /// Should not be called anywhere else.
    /// Use [`Self::clone()`] to create a new instance.
    fn new() -> Self {
        Self {
            keys: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    /// Modifies keystate, not meant to be called manually anywhere else except for in [`Input`].
    fn set(&mut self, keycode: KeyCode, keystatus: KeyStatus) {
        *self.keys.borrow_mut().entry(keycode).or_insert(keystatus) = keystatus;
    }

    pub fn get(&mut self, keycode: KeyCode) -> KeyStatus {
        *self
            .keys
            .borrow_mut()
            .entry(keycode)
            .or_insert(KeyStatus::Released)
    }

    pub fn released(&mut self, keycode: KeyCode) -> bool {
        self.get(keycode) == KeyStatus::Released
    }

    pub fn pressed(&mut self, keycode: KeyCode) -> bool {
        self.get(keycode) == KeyStatus::Pressed
    }
}

pub trait EventHandler {
    fn handle(&mut self, state: KeyState, keycode: KeyCode, keystatus: KeyStatus) -> bool;
}

pub struct Input {
    hwnd: HWND,
    keys: KeyState,
    event_handler: Rc<RefCell<dyn EventHandler>>,
}

impl Input {
    /// # Panics
    /// Panics if the Windows raw input devices fail to be created, most likely
    /// due to a bug. This scenario is irrecoverable.
    pub fn create(event_handler: Rc<RefCell<dyn EventHandler>>) -> Rc<RefCell<Self>> {
        unsafe {
            let mut wndclass = WNDCLASSEXA {
                // Unwrap should not fail here
                cbSize: u32::try_from(std::mem::size_of::<WNDCLASSEXA>()).unwrap(),
                hInstance: GetModuleHandleA(PCSTR(std::ptr::null())).unwrap(),
                lpfnWndProc: Some(raw_input_callback),
                ..Default::default()
            };

            // `CString::new()` fails if the passed byte slice contains a null-terminator.
            // Since the value is hard-coded, I know for a fact that it does not contain one.
            let class_name = std::ffi::CString::new("xterminatorwcname".as_bytes()).unwrap();

            wndclass.lpszClassName = PCSTR(class_name.as_ptr().cast::<u8>());

            assert!(
                RegisterClassExA(&wndclass) > 0,
                "input window class registration failed: RegisterClassA() returned NULL (os error code {})", 
                GetLastError().0
            );

            logf!("Creating input processing message-only window");
            let hwnd = CreateWindowExA(
                WINDOW_EX_STYLE::default(),
                wndclass.lpszClassName,
                PCSTR(std::ptr::null()),
                WINDOW_STYLE::default(),
                0,
                0,
                0,
                0,
                HWND_MESSAGE,
                HMENU(0),
                wndclass.hInstance,
                None,
            );

            assert!(
                hwnd.0 > 0,
                "window creation failed: CreateWindowExA() returned NULL (os error code {})",
                GetLastError().0
            );

            let instance = Rc::new(RefCell::new(Self {
                hwnd,
                keys: KeyState::new(),
                event_handler,
            }));

            SetWindowLongPtrW(
                hwnd,
                GWLP_USERDATA,
                std::ptr::addr_of_mut!(*instance.borrow_mut()) as isize,
            );
            instance
        }
    }

    pub fn poll(&self) {
        unsafe {
            let mut message = MSG::default();
            while PeekMessageA(&mut message, self.hwnd, 0, 0, PM_REMOVE).as_bool() {
                TranslateMessage(&message);
                DispatchMessageA(&message);
            }
        }
    }

    /// Returns a shared copy of the application's [`KeyState`].
    #[must_use]
    pub fn keystate(&self) -> KeyState {
        self.keys.clone()
    }

    pub fn unregister(&self) {
        unsafe {
            // DestroyWindow triggers the WM_DESTROY message in the
            // WndProc handler above, which in turn unregiosters the
            // raw input devices.
            DestroyWindow(self.hwnd);
        }
    }
}

/// Processes raw mouse input [`RAWKEYBOARD`] into a universal [`KeyStatus`].
fn process_keyboard_input(keyboard: &RAWKEYBOARD) -> Option<(KeyCode, KeyStatus)> {
    // Maps the scancode to a virtual keycode that differentiates between left/right
    // versions of certain keys (such as L/R control, shift, alt, etc). The VKey value in
    // 'RAWINPUT::data::keyboard::VKey' doesn't differentiate them, but by using the
    // 'MapVirtualKeyW' function the scancode can be mapped to a virtual key-code that does.
    let vkey = unsafe { MapVirtualKeyW(u32::from(keyboard.MakeCode), MAPVK_VSC_TO_VK_EX) };

    let keycode = KeyCode::from_vkey(u16::try_from(vkey).unwrap())?;
    let keystatus = KeyStatus::from_wm(keyboard.Message)?;

    Some((keycode, keystatus))
}

/// Processes raw mouse input [`RAWMOUSE`] into a universal [`KeyStatus`].
fn process_mouse_input(mouse: &RAWMOUSE) -> Option<(KeyCode, KeyStatus)> {
    let ri = unsafe { mouse.Anonymous.Anonymous.usButtonFlags };

    let keycode = KeyCode::from_ri(u32::from(ri))?;
    let keystatus = KeyStatus::from_ri(u32::from(ri))?;

    Some((keycode, keystatus))
}

unsafe extern "system" fn raw_input_callback(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CREATE => {
            logf!("Recieved WM_CREATE message");

            register_raw_input_devices(hwnd);

            LRESULT(0)
        }

        WM_DESTROY => {
            logf!("Recieved WM_DESTROY message");

            unregister_raw_input_devices();

            LRESULT(0)
        }

        WM_INPUT => process_raw_input_event(hwnd, msg, lparam, wparam),

        _ => {
            // Any other message kind can be ignored and passed on
            DefWindowProcA(hwnd, msg, wparam, lparam)
        }
    }
}

fn register_raw_input_devices(hwnd: HWND) {
    unsafe {
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

        logf!("Registering raw input devices");
        RegisterRawInputDevices(
            &devices,
            u32::try_from(std::mem::size_of::<RAWINPUTDEVICE>())
                .expect("size of struct `RAWINPUTDEVICE` is greater than `u32` max size"),
        )
        .expect(format!("RegisterRawInputDevices() failed: {}", GetLastError().0).as_str());
    }
}

fn unregister_raw_input_devices() {
    unsafe {
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

        logf!("Unregistering raw input devices");
        RegisterRawInputDevices(
            &devices,
            u32::try_from(std::mem::size_of::<RAWINPUTDEVICE>())
                .expect("size of struct `RAWINPUTDEVICE` is greater than `u32` max size"),
        )
        .expect(format!("RegisterRawInputDevices() failed: {}", GetLastError().0).as_str());
    }
}

fn process_raw_input_event(hwnd: HWND, msg: u32, lparam: LPARAM, wparam: WPARAM) -> LRESULT {
    unsafe {
        let mut dwsize = u32::default();

        // Todo: Ensure panics are actually logged properly
        if GetRawInputData(
            std::mem::transmute::<LPARAM, HRAWINPUT>(lparam),
            RID_INPUT,
            None,
            &mut dwsize,
            u32::try_from(std::mem::size_of::<RAWINPUTHEADER>())
                .expect("size of RAWINPUTHEADER struct is greater than `u32` max size"),
        ) == std::mem::transmute::<i32, u32>(-1)
        {
            panic!(
                "first call to GetRawInputData() failed: {}",
                GetLastError().0
            )
        }

        let mut data = vec![0; dwsize as usize];
        let data = data.as_mut_ptr().cast::<std::ffi::c_void>();

        if GetRawInputData(
            std::mem::transmute::<LPARAM, HRAWINPUT>(lparam),
            RID_INPUT,
            Some(data),
            &mut dwsize,
            u32::try_from(std::mem::size_of::<RAWINPUTHEADER>())
                .expect("size of RAWINPUTHEADER struct is greater than `u32` max size"),
        ) != dwsize
        {
            panic!(
                "second call to GetRawInputData() failed: {}",
                GetLastError().0
            )
        }

        let rawinput = data.cast::<RAWINPUT>();

        let keystate = match RID_DEVICE_INFO_TYPE((*rawinput).header.dwType) {
            RIM_TYPEKEYBOARD => process_keyboard_input(&(*rawinput).data.keyboard),
            RIM_TYPEMOUSE => process_mouse_input(&(*rawinput).data.mouse),

            // HID messages are unused by the application so just ignore and pass them on
            // Commented out since it is caught by the wildcard below
            // RIM_TYPEHID => None,

            // Should (knock on wood) be impossible since 'dwType' can only
            // be any of the above three values acording to Windows docs
            _ => None,
        };

        if let Some((keycode, keystatus)) = keystate {
            let instance = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut Input;

            instance.as_mut().unwrap().keys.set(keycode, keystatus);

            // Callback determines whether the input message
            // was processed or not, if it was then LRESULT should be 0.
            let handler = &mut instance.as_mut().unwrap().event_handler;
            let processed = handler.as_ref().borrow_mut().handle(
                instance.as_mut().unwrap().keys.clone(),
                keycode,
                keystatus,
            );
            if processed {
                logf!(
                    "Processed and consumed relevant input: ({}, {})",
                    keycode,
                    keystatus
                );
                return LRESULT(0);
            }
        }

        DefWindowProcA(hwnd, msg, wparam, lparam)
    }
}

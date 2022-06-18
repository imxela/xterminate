use windows::core::PCSTR;

use windows::Win32::Foundation::{
    LPARAM,
    WPARAM,
    LRESULT,
    POINT,
    HWND,
    CHAR,
    HINSTANCE,

    GetLastError
};

use windows::Win32::UI::Shell::{
    NOTIFYICONDATAA,

    NIF_ICON,
    NIF_MESSAGE,
    NIF_TIP,
    NIM_ADD,
    NIM_DELETE,

    Shell_NotifyIconA
};

use windows::Win32::UI::WindowsAndMessaging::{
    HICON,
    HMENU,
    WNDCLASSEXA,
    MSG,

    IMAGE_ICON,
    LR_LOADFROMFILE,
    MF_BYPOSITION,
    PM_REMOVE,
    TPM_BOTTOMALIGN,
    WM_COMMAND,
    WM_LBUTTONDOWN,
    WM_RBUTTONDOWN,
    WM_USER,
    GWLP_USERDATA,

    CreateWindowExA,
    RegisterClassExA,
    DefWindowProcA,
    CreatePopupMenu,
    TrackPopupMenu,
    InsertMenuA,
    SetForegroundWindow,
    PeekMessageA,
    TranslateMessage,
    DispatchMessageA,
    GetCursorPos,
    LoadImageA, 
    DestroyWindow,
    SetWindowLongPtrA, 
    GetWindowLongPtrA
};

use windows::Win32::System::LibraryLoader::GetModuleHandleA;

const TRAYICON_ID: u32 = 1;
const WM_USER_TRAYICON: u32 = WM_USER + TRAYICON_ID;

use std::rc::Rc;
use std::cell::RefCell;

#[repr(usize)]
pub enum TrayEvent {
    OnMenuSelectExit = 0,
    OnMenuSelectResetCursor = 1
}

pub trait TrayEventHandler {
    fn handle(&mut self, event: TrayEvent);
}

impl From<u16> for TrayEvent {
    fn from(v: u16) -> Self {
        match v {
            0 => Self::OnMenuSelectExit,
            1 => Self::OnMenuSelectResetCursor,
            _ => panic!("Invalid enum value '{}'", v)
        }
    }
}

pub struct Tray {
    hwnd: HWND,
    nid: NOTIFYICONDATAA,

    event_handler: Rc<RefCell<dyn TrayEventHandler>>
}

impl Tray {
    pub fn create(icon_filename: &str, event_handler: Rc<RefCell<dyn TrayEventHandler>>) -> Self {
        // Create message-only trayicon window

        let hwnd = Self::create_window();
        let nid = Self::create_trayicon(hwnd, icon_filename);

        let mut tray = Self {
            hwnd: hwnd,
            nid: nid,
            event_handler
        };

        // Todo: Move this into create_window()?
        unsafe { SetWindowLongPtrA(hwnd, GWLP_USERDATA, &mut tray as *mut Tray as isize); }

        tray
    }

    pub fn delete(&self) { unsafe {
        if !Shell_NotifyIconA(NIM_DELETE, &self.nid).as_bool() {
            panic!("tray icon could not be deleted");
        }
        
        DestroyWindow(self.hwnd);
    }}

    fn create_window() -> HWND { unsafe {
        let mut wndclass = WNDCLASSEXA::default();
        wndclass.cbSize = std::mem::size_of::<WNDCLASSEXA>() as u32;
        wndclass.hInstance = GetModuleHandleA(PCSTR(std::ptr::null())); // Equivalent to the hInstance parameter passed to WinMain in C/C++
        wndclass.lpfnWndProc = Some(trayicon_input_callback);
        
        let class_name = std::ffi::CString::new("xterminatortrayiconwcname".as_bytes()).unwrap();
        wndclass.lpszClassName = PCSTR(class_name.as_ptr() as *const u8);
        
        if RegisterClassExA(&wndclass) == 0 {
            panic!("tray-icon window class registration failed: RegisterClassA() returned NULL (os error code {})", GetLastError().0);
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
            HWND(0),
            HMENU(0),
            wndclass.hInstance,
            std::ptr::null()
        );

        if hwnd.0 == 0 {
            panic!("trayicon window creation failed: CreateWindowExA() returned NULL (os error code {})", GetLastError().0);
        }

        hwnd
    }}

    fn create_trayicon(hwnd: HWND, icon_filename: &str) -> NOTIFYICONDATAA { unsafe {
        let mut nid = NOTIFYICONDATAA::default();
        nid.cbSize = std::mem::size_of::<NOTIFYICONDATAA>() as u32;

        nid.hWnd = hwnd;
        nid.uID = TRAYICON_ID;
        nid.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;

        // NIF_MESSAGE
        nid.uCallbackMessage = WM_USER_TRAYICON;

        // NIF_ICON
        nid.hIcon = Self::load_icon_from_file(icon_filename);

        // NIF_TIP
        // str to CHAR array conversion
        let tooltip_str = "xterminate says hi! :)";
        let mut tooltip_message: [CHAR; 128] = [CHAR(0u8); 128];
        tooltip_str.bytes().zip(tooltip_message.iter_mut()).for_each(|(b, ptr)| *ptr = CHAR(b));

        nid.szTip = tooltip_message;
    
        Shell_NotifyIconA(NIM_ADD, &nid);

        nid
    }}

    fn show_menu(&mut self) { unsafe {
        let mut cursor_pos = POINT::default();
        GetCursorPos(&mut cursor_pos);

        let menu_handle = CreatePopupMenu().unwrap();
        InsertMenuA(menu_handle, 1, MF_BYPOSITION, TrayEvent::OnMenuSelectResetCursor as usize , "Reset cursor");
        InsertMenuA(menu_handle, 2, MF_BYPOSITION, TrayEvent::OnMenuSelectExit as usize, "Exit");
        
        // Required or the popup menu won't close properly
        SetForegroundWindow(self.hwnd);

        TrackPopupMenu(menu_handle, TPM_BOTTOMALIGN, cursor_pos.x, cursor_pos.y, 0, self.hwnd, std::ptr::null());
    }} 

    pub fn poll(&self) { unsafe {
        let mut message = MSG::default();
        if PeekMessageA(&mut message, self.hwnd, 0, 0, PM_REMOVE).as_bool()
        {
            TranslateMessage(&message);
            DispatchMessageA(&message);
        }
    }}

    pub fn load_icon_from_file(filename: &str) -> HICON {
        let hicon = unsafe {
            LoadImageA(
                HINSTANCE { 0: 0 }, 
                filename, 
                IMAGE_ICON, 
                0, 
                0, 
                LR_LOADFROMFILE
            )
        };

        match hicon {
            Ok(v) => HICON(v.0),
            Err(_) => {
                panic!("Failed to load icon '{}': is the file missing or corrupt? (os error {})", filename, unsafe { GetLastError().0 })
            }
        }
    }
}

unsafe extern "system" fn trayicon_input_callback(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let instance = GetWindowLongPtrA(hwnd, GWLP_USERDATA) as *mut Tray;

    match msg {
        WM_USER_TRAYICON => {
            match lparam.0 as u32 {
                WM_LBUTTONDOWN | WM_RBUTTONDOWN => {
                    instance.as_mut().unwrap().show_menu();
                    LRESULT(0)
                },

                _ => {
                    DefWindowProcA(hwnd, msg, wparam, lparam)
                }
            }
        },

        WM_COMMAND => {
            // Separate the first and last 2 bytes (4 bits) of wparam, equivalent to LOWORD()/HIWORD()
            // The low bytes tell us which of the popup menu's items were clicked ('command'), and
            // correspond to the WMU_XXX events defined at the top of this file.
            
            // let id = (wparam.0 & 0b11110000) as u16;
            let cmd = (wparam.0 & 0b00001111) as u16;

            let handler = &mut instance.as_mut().unwrap().event_handler;
            handler.borrow_mut().handle(TrayEvent::from(cmd));

            LRESULT(0)
        },

        _ => {
            // Any other messages can be ignored as we only care about the trayicon related ones
            DefWindowProcA(hwnd, msg, wparam, lparam)
        }
    }
}
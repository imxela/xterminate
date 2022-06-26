use windows::Win32::System::Threading::{
    OpenProcess,
    TerminateProcess,
    WaitForSingleObject,
    
    PROCESS_TERMINATE,
    PROCESS_SYNCHRONIZE
};

use windows::Win32::Foundation::{
    GetLastError,

    HANDLE,
    WPARAM,
    LPARAM,
    HWND,
    BOOL,

    ERROR_APP_HANG,
    WAIT_TIMEOUT,
    WAIT_FAILED
};

use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows,
    GetWindowThreadProcessId,
    EnumChildWindows,
    SendNotifyMessageA,

    WM_CLOSE
};

pub struct Process {
    id: u32,
    handle: isize
}

impl Process {
    /// Opens the process with the specified PID and returns a [Process].
    /// 
    /// ## Panics
    /// 
    /// This function panics if the internal call to `OpenProcess()` returns a [HANDLE] of value `0`.
    pub fn open(pid : u32) -> Self {
        let handle = unsafe {
            OpenProcess(PROCESS_TERMINATE | PROCESS_SYNCHRONIZE, false, pid)
        }.expect(format!("failed to open target process ({}) (system error {})", pid, unsafe { GetLastError().0 }).as_str());

        Self {
            id: pid,
            handle: handle.0
        }
    }

    /// Attempts to exit the process gracefully by sending a WM_CLOSE
    /// message to all associated windows. Returns true if the process
    /// exits or, false if it fails or if the timeout is exceeded.
    pub fn try_exit(&mut self, timeout_ms: u32) -> bool { unsafe {
        EnumWindows(Some(Self::enumerate_windows_cb), LPARAM(self.id() as isize));

        let result = WaitForSingleObject(HANDLE(self.handle()), timeout_ms);

        if result == WAIT_TIMEOUT.0 {
            println!("Timed out!");
            return false;
        } else if result == WAIT_FAILED.0 {
            panic!("failed to wait for process exit: WaitForSingleObject returned WAIT_FAILED (os error {})", GetLastError().0);
        }

        true
    } }
    
    unsafe extern "system" fn enumerate_windows_cb(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let mut wnd_process_id = 0;

        GetWindowThreadProcessId(hwnd, &mut wnd_process_id);

        if wnd_process_id == lparam.0 as u32 {
            EnumChildWindows(hwnd, Some(Self::enumerate_child_windows_cb), LPARAM(0));

            if !SendNotifyMessageA(hwnd, WM_CLOSE, WPARAM(0), LPARAM(0)).as_bool() {
                panic!("failed to send WM_CLOSE message to window: SendNotifyMessageA() returned false (os error {})", GetLastError().0);
            }
        }

        // No matching process was found
        BOOL(true as i32)
    }

    unsafe extern "system" fn enumerate_child_windows_cb(hwnd: HWND, _lparam: LPARAM) -> BOOL {
        if !SendNotifyMessageA(hwnd, WM_CLOSE, WPARAM(0), LPARAM(0)).as_bool() {
            panic!("failed to send WM_CLOSE message to window: SendNotifyMessageA() returned false (os error {})", GetLastError().0);
        }

        BOOL(true as i32)
    }

    /// Terminates the `self` process.
    /// 
    /// ## Panics
    /// 
    /// This method panics if the internal call to `TerminateProcess()` returns ´false´.
    pub fn terminate(&self) {
        let success = unsafe { 
            TerminateProcess(HANDLE { 0: self.handle }, ERROR_APP_HANG.0).as_bool()
        };

        if !success {
            panic!("{}", format!("failed to terminate target process ({}) (system error {})", self.id(), unsafe { GetLastError().0 }));
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn handle(&self) -> isize {
        self.handle
    }

}
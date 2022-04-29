use windows::Win32::System::Threading::{ OpenProcess, TerminateProcess, PROCESS_TERMINATE };
use windows::Win32::Foundation::{ HANDLE, ERROR_APP_HANG, GetLastError };

pub struct Process {
    id: u32,
    handle: isize
}

impl Process {
    pub fn open(pid : u32) -> Self {
        let handle = unsafe {
            OpenProcess(PROCESS_TERMINATE, false, pid)
        };

        let handle = handle.expect(format!("OpenProcess failed, code: {}", unsafe { GetLastError().0 }).as_str()).0; // Todo: Handle this error properly (map to custom error type?)

        Self {
            id: pid,
            handle
        }
    }

    pub fn terminate(&self) {
        let success = unsafe { 
            TerminateProcess(HANDLE { 0: self.handle }, ERROR_APP_HANG.0).as_bool()
        };

        if !success {
            // Todo: Handle this error
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn handle(&self) -> isize {
        self.handle
    }

}
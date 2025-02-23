use std::fs::OpenOptions;

#[cfg(unix)]
use std::os::unix::io::{AsRawFd, RawFd};
#[cfg(windows)]
use std::os::windows::io::{AsRawHandle, RawHandle};

#[cfg(unix)]
use libc;
#[cfg(windows)]
use winapi::um::handleapi::INVALID_HANDLE_VALUE;
#[cfg(windows)]
use winapi::um::fileapi::CreateFileA;
#[cfg(windows)]
use winapi::um::winbase::{FILE_ATTRIBUTE_NORMAL, FILE_GENERIC_WRITE, OPEN_EXISTING};

pub struct OutputRedirect {
    #[cfg(unix)]
    original_stdout: RawFd,
    #[cfg(unix)]
    original_stderr: RawFd,

    #[cfg(windows)]
    original_stdout: RawHandle,
    #[cfg(windows)]
    original_stderr: RawHandle,
}

impl OutputRedirect {
    pub fn redirect() -> Self {
        #[cfg(unix)]
        {
            let dev_null = OpenOptions::new().write(true).open("/dev/null").unwrap();
            let dev_null_fd = dev_null.as_raw_fd();

            let original_stdout = unsafe { libc::dup(1) };
            let original_stderr = unsafe { libc::dup(2) };

            unsafe {
                libc::dup2(dev_null_fd, 1);
                libc::dup2(dev_null_fd, 2);
            }

            Self { original_stdout, original_stderr }
        }

        #[cfg(windows)]
        {
            use std::ptr;
            use winapi::um::processenv::GetStdHandle;
            use winapi::um::handleapi::CloseHandle;
            use winapi::um::winbase::{STD_OUTPUT_HANDLE, STD_ERROR_HANDLE};

            let original_stdout = unsafe { GetStdHandle(STD_OUTPUT_HANDLE) };
            let original_stderr = unsafe { GetStdHandle(STD_ERROR_HANDLE) };

            let dev_null = unsafe {
                CreateFileA(
                    b"NUL\0".as_ptr() as *const i8,
                    FILE_GENERIC_WRITE,
                    0,
                    ptr::null_mut(),
                    OPEN_EXISTING,
                    FILE_ATTRIBUTE_NORMAL,
                    ptr::null_mut(),
                )
            };

            if dev_null == INVALID_HANDLE_VALUE {
                panic!("Failed to open NUL device");
            }

            unsafe {
                libc::dup2(dev_null as i32, 1);
                libc::dup2(dev_null as i32, 2);
                CloseHandle(dev_null);
            }

            Self { original_stdout, original_stderr }
        }
    }

    pub fn restore(self) {
        #[cfg(unix)]
        {
            unsafe {
                libc::dup2(self.original_stdout, 1);
                libc::dup2(self.original_stderr, 2);
                libc::close(self.original_stdout);
                libc::close(self.original_stderr);
            }
        }

        #[cfg(windows)]
        {
            use winapi::um::processenv::SetStdHandle;
            use winapi::um::winbase::{STD_OUTPUT_HANDLE, STD_ERROR_HANDLE};

            unsafe {
                SetStdHandle(STD_OUTPUT_HANDLE, self.original_stdout);
                SetStdHandle(STD_ERROR_HANDLE, self.original_stderr);
            }
        }
    }
}

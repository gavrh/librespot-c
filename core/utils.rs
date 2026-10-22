#[cfg(unix)]
use std::fs::OpenOptions;

#[cfg(unix)]
use std::os::unix::io::{AsRawFd, RawFd};
#[cfg(windows)]
use std::os::windows::io::RawHandle;

#[cfg(unix)]
use libc;
#[cfg(windows)]
use winapi::um::fileapi::{CreateFileA, OPEN_EXISTING};
#[cfg(windows)]
use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
#[cfg(windows)]
use winapi::um::processenv::{GetStdHandle, SetStdHandle};
#[cfg(windows)]
use winapi::um::winbase::{STD_ERROR_HANDLE, STD_OUTPUT_HANDLE};
#[cfg(windows)]
use winapi::um::winnt::{FILE_ATTRIBUTE_NORMAL, FILE_GENERIC_WRITE};

pub struct OutputRedirect {
    #[cfg(unix)]
    original_stdout: Option<RawFd>,
    #[cfg(unix)]
    original_stderr: Option<RawFd>,

    #[cfg(windows)]
    original_stdout: Option<RawHandle>,
    #[cfg(windows)]
    original_stderr: Option<RawHandle>,
    #[cfg(windows)]
    dev_null: Option<RawHandle>,
}

impl OutputRedirect {
    pub fn redirect() -> Self {
        Self::try_redirect().unwrap_or_else(Self::inactive)
    }

    pub fn try_redirect() -> Option<Self> {
        #[cfg(unix)]
        {
            let dev_null = OpenOptions::new().write(true).open("/dev/null").ok()?;
            let dev_null_fd = dev_null.as_raw_fd();

            let original_stdout = unsafe { libc::dup(1) };
            if original_stdout < 0 {
                return None;
            }

            let original_stderr = unsafe { libc::dup(2) };
            if original_stderr < 0 {
                unsafe {
                    libc::close(original_stdout);
                }
                return None;
            }

            if unsafe { libc::dup2(dev_null_fd, 1) } < 0 {
                unsafe {
                    libc::close(original_stdout);
                    libc::close(original_stderr);
                }
                return None;
            }

            if unsafe { libc::dup2(dev_null_fd, 2) } < 0 {
                unsafe {
                    libc::dup2(original_stdout, 1);
                    libc::close(original_stdout);
                    libc::close(original_stderr);
                }
                return None;
            }

            Some(Self {
                original_stdout: Some(original_stdout),
                original_stderr: Some(original_stderr),
            })
        }

        #[cfg(windows)]
        {
            use std::ptr;

            let original_stdout = unsafe { GetStdHandle(STD_OUTPUT_HANDLE) };
            let original_stderr = unsafe { GetStdHandle(STD_ERROR_HANDLE) };

            if original_stdout.is_null()
                || original_stdout == INVALID_HANDLE_VALUE
                || original_stderr.is_null()
                || original_stderr == INVALID_HANDLE_VALUE
            {
                return None;
            }

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
                return None;
            }

            if unsafe { SetStdHandle(STD_OUTPUT_HANDLE, dev_null) } == 0 {
                unsafe {
                    CloseHandle(dev_null);
                }
                return None;
            }

            if unsafe { SetStdHandle(STD_ERROR_HANDLE, dev_null) } == 0 {
                unsafe {
                    SetStdHandle(STD_OUTPUT_HANDLE, original_stdout);
                    CloseHandle(dev_null);
                }
                return None;
            }

            Some(Self {
                original_stdout: Some(original_stdout),
                original_stderr: Some(original_stderr),
                dev_null: Some(dev_null),
            })
        }

        #[cfg(not(any(unix, windows)))]
        {
            None
        }
    }

    pub fn restore(mut self) {
        self.restore_inner();
    }

    fn restore_inner(&mut self) {
        #[cfg(unix)]
        {
            if let Some(original_stdout) = self.original_stdout.take() {
                unsafe {
                    libc::dup2(original_stdout, 1);
                    libc::close(original_stdout);
                }
            }

            if let Some(original_stderr) = self.original_stderr.take() {
                unsafe {
                    libc::dup2(original_stderr, 2);
                    libc::close(original_stderr);
                }
            }
        }

        #[cfg(windows)]
        {
            if let Some(original_stdout) = self.original_stdout.take() {
                unsafe {
                    SetStdHandle(STD_OUTPUT_HANDLE, original_stdout);
                }
            }

            if let Some(original_stderr) = self.original_stderr.take() {
                unsafe {
                    SetStdHandle(STD_ERROR_HANDLE, original_stderr);
                }
            }

            if let Some(dev_null) = self.dev_null.take() {
                unsafe {
                    CloseHandle(dev_null);
                }
            }
        }
    }

    fn inactive() -> Self {
        #[cfg(unix)]
        {
            Self {
                original_stdout: None,
                original_stderr: None,
            }
        }

        #[cfg(windows)]
        {
            Self {
                original_stdout: None,
                original_stderr: None,
                dev_null: None,
            }
        }

        #[cfg(not(any(unix, windows)))]
        {
            Self {}
        }
    }
}

impl Drop for OutputRedirect {
    fn drop(&mut self) {
        self.restore_inner();
    }
}

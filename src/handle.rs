use crate::common::{Error, Result};
use crate::ffi;
use std::ffi::CStr;

#[derive(Debug)]
pub struct Handle {
    ptr: ffi::tjhandle,
}

impl Handle {
    pub fn new(init: ffi::TJINIT) -> Result<Self> {
        let ptr = unsafe { ffi::tj3Init(init as libc::c_int) };
        let mut this = Self { ptr };
        if this.ptr.is_null() {
            return Err(this.get_error());
        }
        Ok(this)
    }

    pub fn get_error(&mut self) -> Error {
        let msg = unsafe { CStr::from_ptr(ffi::tj3GetErrorStr(self.ptr)) };
        Error::TurboJpegError(msg.to_string_lossy().into_owned())
    }

    pub fn get(&mut self, param: ffi::TJPARAM) -> libc::c_int {
        unsafe { ffi::tj3Get(self.ptr, param as libc::c_int) }
    }

    pub fn set(&mut self, param: ffi::TJPARAM, value: libc::c_int) -> Result<()> {
        let res = unsafe { ffi::tj3Set(self.ptr, param as libc::c_int, value) };
        if res != 0 {
            return Err(self.get_error());
        }
        Ok(())
    }

    pub unsafe fn as_ptr(&mut self) -> ffi::tjhandle {
        self.ptr
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        unsafe {
            ffi::tj3Destroy(self.ptr);
        }
    }
}

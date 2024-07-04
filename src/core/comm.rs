use std::io::{Read, Result, Write};

use libc::c_void;

use crate::syscall;

pub struct FdComm {
    pub fd: i32,
}

impl Write for FdComm {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        return syscall!(write(self.fd, buf.as_ptr() as *const c_void, buf.len())).map(|res| res as usize);
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

impl Read for FdComm {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        return syscall!(read(self.fd, buf.as_mut_ptr() as *mut c_void, buf.len())).map(|res| res as usize);
    }
}

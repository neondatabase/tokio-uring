use crate::fs::{File, OpenOptions};
use crate::io::SharedFd;

use crate::runtime::driver::op::{Completable, CqeResult, Op};
use crate::runtime::CONTEXT;
use std::ffi::CString;
use std::io;
use std::path::Path;

/// Open a file
#[allow(dead_code)]
pub(crate) struct Open {
    pub(crate) path: CString,
}

impl Op<Open> {
    /// Submit a request to open a file.
    pub(crate) fn open(path: &Path, options: &OpenOptions) -> io::Result<Op<Open>> {
        let path = super::util::cstr(path)?;
        // Get a reference to the memory. The string will be held by the
        // operation state and will not be accessed again until the operation
        // completes.
        let sqe = unsafe {
            let p_ref = path.as_c_str().as_ptr();
            crate::fs::OpenOptionsIoUringExt::as_openat_sqe(options, p_ref)?
        };
        CONTEXT.with(|x| {
            x.handle()
                .expect("Not in a runtime context")
                .submit_op(Open { path }, move |_| sqe)
        })
    }
}

impl Completable for Open {
    type Output = io::Result<File>;

    fn complete(self, cqe: CqeResult) -> Self::Output {
        Ok(File::from_shared_fd(SharedFd::new(cqe.result? as _)))
    }
}

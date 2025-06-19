use std::path::Path;
use std::process::ExitStatus;
use std::{fs, io};

pub trait EarlyRet<E>: Sized {
    fn early_ret(self) -> Result<Self, E>;
}

impl EarlyRet<io::Error> for ExitStatus {
    fn early_ret(self) -> io::Result<Self> {
        match self.success() {
            true => Ok(self),
            false => Err(io::Error::other("Command failed")),
        }
    }
}

pub fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

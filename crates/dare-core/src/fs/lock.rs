//! Exclusive file locks via fs4 (`<path>.darelock`).

use std::fs::{File, OpenOptions};

use fs4::{FileExt, TryLockError};

use crate::error::{CoreError, CoreResult};
use crate::path::{ProjectRoot, SafeRelativePath};

pub struct FileLock {
    file: File,
}

impl FileLock {
    /// Creates/opens `<abs>.darelock` and exclusive `try_lock`.
    pub fn try_acquire(root: &ProjectRoot, rel: &SafeRelativePath) -> CoreResult<Self> {
        let abs = root.resolve(rel)?;
        let parent = abs
            .as_path()
            .parent()
            .ok_or_else(|| CoreError::invalid_input("path has no parent"))?;
        std::fs::create_dir_all(parent.as_std_path()).map_err(|e| CoreError::io(e.to_string()))?;

        let lock_path = format!("{}.darelock", abs.as_path());
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .truncate(false)
            .open(&lock_path)
            .map_err(|e| CoreError::io(e.to_string()))?;

        FileExt::try_lock(&file).map_err(|e| match e {
            TryLockError::WouldBlock => CoreError::io("file lock held"),
            TryLockError::Error(err) => CoreError::io(err.to_string()),
        })?;

        Ok(Self { file })
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        let _ = FileExt::unlock(&self.file);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fs::atomic::atomic_write;
    use crate::path::{ProjectRoot, SafeRelativePath};
    use tempfile::tempdir;

    #[test]
    fn file_lock_try_acquire_contention() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let rel = SafeRelativePath::new("locked.txt").unwrap();
        atomic_write(&root, &rel, b"x").unwrap();

        let first = FileLock::try_acquire(&root, &rel).expect("first lock");
        let second = FileLock::try_acquire(&root, &rel);
        assert!(second.is_err(), "second lock should fail");
        drop(first);
        let third = FileLock::try_acquire(&root, &rel);
        assert!(third.is_ok(), "lock after drop should succeed");
    }
}

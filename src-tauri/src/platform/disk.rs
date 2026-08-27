use std::path::Path;

use crate::domain::DiskUsage;

pub fn disk_usage_for(directory: &Path) -> Option<DiskUsage> {
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        use windows::{core::PCWSTR, Win32::Storage::FileSystem::GetDiskFreeSpaceExW};

        let existing_directory = std::iter::successors(Some(directory), |path| path.parent())
            .find(|path| path.exists())?;
        let path: Vec<u16> = existing_directory
            .as_os_str()
            .encode_wide()
            .chain(Some(0))
            .collect();
        let mut total_bytes = 0;
        let mut available_bytes = 0;
        unsafe {
            GetDiskFreeSpaceExW(
                PCWSTR(path.as_ptr()),
                None,
                Some(&mut total_bytes),
                Some(&mut available_bytes),
            )
            .ok()?;
        }
        Some(DiskUsage {
            total_bytes,
            available_bytes,
        })
    }
    #[cfg(not(windows))]
    {
        let _ = directory;
        None
    }
}

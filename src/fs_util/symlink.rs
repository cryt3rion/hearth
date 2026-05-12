use std::path::{Path, PathBuf};

/// Fully resolve symlinks. Returns None for broken symlinks or other I/O errors.
pub fn canonicalize_safe(p: &Path) -> Option<PathBuf> {
    std::fs::canonicalize(p).ok()
}

/// Is this path executable (Unix `x` bits)?
pub fn is_executable(meta: &std::fs::Metadata) -> bool {
    use std::os::unix::fs::PermissionsExt;
    meta.permissions().mode() & 0o111 != 0
}

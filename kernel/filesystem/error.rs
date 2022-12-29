use kernel_api::headers::errno::ErrnoCode;

// FIXME: Some of these should probably wrap or just use the POSIX values.
pub enum FileSystemErr {
    EBadF,
    ENotDir,
    ENOMEM,
    FileNotFound,
    Unknown,
}

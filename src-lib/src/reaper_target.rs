pub enum ReaperTarget {
    /// Rust: macos/aarch64
    MacOsAarch64,
    /// Rust: macos/x86
    MacOsX86,
    /// Rust: macos/x86_64
    MacOsX86_64,
    /// Rust: windows/x86
    WindowsX86,
    /// Rust: windows/x86_64
    WindowsX64,
    /// Rust: linux/aarch64
    LinuxAarch64,
    /// Rust: linux/arm
    LinuxArmv7l,
    /// Rust: linux/x86
    LinuxI686,
    /// Rust: linux/x86_64
    LinuxX86_64,
}

impl ReaperTarget {
    pub const AT_BUILD_TIME: Option<Self> = Self::from_reaboot_build();

    const fn from_reaboot_build() -> Option<Self> {
        let target = if cfg!(target_os = "macos") {
            if cfg!(target_arch = "aarch64") {
                Self::MacOsAarch64
            } else if cfg!(target_arch = "x86") {
                Self::MacOsX86
            } else if cfg!(target_arch = "x86_64") {
                Self::MacOsX86_64
            } else {
                return None;
            }
        } else if cfg!(target_os = "windows") {
            if cfg!(target_arch = "x86") {
                Self::WindowsX86
            } else if cfg!(target_arch = "x86_64") {
                Self::WindowsX64
            } else {
                return None;
            }
        } else if cfg!(target_os = "linux") {
            if cfg!(target_arch = "aarch64") {
                Self::LinuxAarch64
            } else if cfg!(target_arch = "x86") {
                Self::LinuxI686
            } else if cfg!(target_arch = "x86_64") {
                Self::LinuxX86_64
            } else if cfg!(target_arch = "arm") {
                Self::LinuxArmv7l
            } else {
                return None;
            }
        } else {
            return None;
        };
        Some(target)
    }
}

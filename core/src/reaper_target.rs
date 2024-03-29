use reaboot_reapack::model::Platform;

use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use ts_rs::TS;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
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

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ReaperTargetFamily {
    MacOs,
    Windows,
    Linux,
}

impl ReaperTarget {
    pub const BUILD: Option<Self> = Self::from_reaboot_build();

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

    pub const fn family(&self) -> ReaperTargetFamily {
        use ReaperTarget::*;
        use ReaperTargetFamily::*;
        match self {
            MacOsAarch64 | MacOsX86 | MacOsX86_64 => MacOs,
            WindowsX86 | WindowsX64 => Windows,
            LinuxAarch64 | LinuxArmv7l | LinuxI686 | LinuxX86_64 => Linux,
        }
    }

    pub fn is_compatible_with_reapack_platform(&self, platform: Platform) -> bool {
        let Some(reaper_target_or_family) = ReaperTargetOrFamily::from_reapack_platform(platform)
        else {
            // Platform "All"
            return true;
        };
        match reaper_target_or_family {
            ReaperTargetOrFamily::Target(t) => *self == t,
            ReaperTargetOrFamily::Family(f) => self.family() == f,
        }
    }
}

pub enum ReaperTargetOrFamily {
    Target(ReaperTarget),
    Family(ReaperTargetFamily),
}

impl ReaperTargetOrFamily {
    pub fn from_reapack_platform(platform: Platform) -> Option<Self> {
        use Platform::*;
        let value = match platform {
            All => return None,
            Darwin => Self::Family(ReaperTargetFamily::MacOs),
            Darwin32 => Self::Target(ReaperTarget::MacOsX86),
            Darwin64 => Self::Target(ReaperTarget::MacOsX86_64),
            DarwinArm64 => Self::Target(ReaperTarget::MacOsAarch64),
            Linux => Self::Family(ReaperTargetFamily::Linux),
            Linux32 => Self::Target(ReaperTarget::LinuxI686),
            Linux64 => Self::Target(ReaperTarget::LinuxX86_64),
            LinuxArmv7l => Self::Target(ReaperTarget::LinuxArmv7l),
            LinuxAarch64 => Self::Target(ReaperTarget::LinuxAarch64),
            Windows => Self::Family(ReaperTargetFamily::Windows),
            Win32 => Self::Target(ReaperTarget::WindowsX86),
            Win64 => Self::Target(ReaperTarget::WindowsX64),
        };
        Some(value)
    }
}

use reaboot_reapack::model::Platform;

use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use ts_rs::TS;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum ReaperPlatform {
    /// - REAPER about: macOS-arm64
    /// - REAPER installer: _universal.dmg (in DMG readme.txt also called arm64)
    /// - ReaPack lib: -arm64.dylib
    /// - Rust: macos/aarch64
    #[serde(rename = "macos-arm64")]
    MacOsArm64,
    /// - REAPER about: ?
    /// - REAPER installer: _i386.dmg
    /// - ReaPack lib: -i386.dylib
    /// - Rust: macos/x86
    #[serde(rename = "macos-i386")]
    MacOsI386,
    /// - REAPER about: OSX64
    /// - REAPER installer: _universal.dmg (in DMG readme.txt also called x86_64)
    /// - ReaPack lib: -x86_64.dylib
    /// - Rust: macos/x86_64
    #[serde(rename = "macos-x86_64")]
    MacOsX86_64,
    /// - REAPER about: win32
    /// - REAPER installer: -install.exe
    /// - ReaPack lib: -x86.dll
    /// - Rust: windows/x86
    #[serde(rename = "windows-x86")]
    WindowsX86,
    /// - REAPER about: win64
    /// - REAPER installer: _x64-install.exe
    /// - ReaPack lib: -x64.dll
    /// - Rust: windows/x86_64
    #[serde(rename = "windows-x64")]
    WindowsX64,
    /// - REAPER about: ?
    /// - REAPER installer: _linux_aarch64.tar.xz
    /// - ReaPack lib: -aarch64.so
    /// - Rust: linux/aarch64
    #[serde(rename = "linux-aarch64")]
    LinuxAarch64,
    /// - REAPER about: ?
    /// - REAPER installer: _linux_armv7l.tar.xz
    /// - ReaPack lib: -armv7l.so
    /// - Rust: linux/arm
    #[serde(rename = "linux-armv7l")]
    LinuxArmv7l,
    /// - REAPER about: ?
    /// - REAPER installer: _linux_i686.tar.xz
    /// - ReaPack lib: -i686.so
    /// - Rust: linux/x86
    #[serde(rename = "linux-i686")]
    LinuxI686,
    /// - REAPER about: linux-x86_64
    /// - REAPER installer: _linux_x86_64.tar.xz
    /// - ReaPack lib: -x86_64.so
    /// - Rust: linux/x86_64
    #[serde(rename = "linux-x86_64")]
    LinuxX86_64,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ReaperOs {
    MacOs,
    Windows,
    Linux,
}

impl ReaperPlatform {
    pub const fn from_reaboot_build() -> Self {
        if let Some(p) = Self::from_reaboot_build_checked() {
            p
        } else {
            panic!("You are building ReaBoot on an unsupported platform");
        }
    }

    const fn from_reaboot_build_checked() -> Option<Self> {
        let target = if cfg!(target_os = "macos") {
            if cfg!(target_arch = "aarch64") {
                Self::MacOsArm64
            } else if cfg!(target_arch = "x86") {
                Self::MacOsI386
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

    pub const fn family(&self) -> ReaperOs {
        use ReaperOs::*;
        use ReaperPlatform::*;
        match self {
            MacOsArm64 | MacOsI386 | MacOsX86_64 => MacOs,
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
            ReaperTargetOrFamily::Platform(t) => *self == t,
            ReaperTargetOrFamily::Os(f) => self.family() == f,
        }
    }
}

pub enum ReaperTargetOrFamily {
    Platform(ReaperPlatform),
    Os(ReaperOs),
}

impl ReaperTargetOrFamily {
    pub fn from_reapack_platform(platform: Platform) -> Option<Self> {
        use Platform::*;
        let value = match platform {
            All => return None,
            Darwin => Self::Os(ReaperOs::MacOs),
            Darwin32 => Self::Platform(ReaperPlatform::MacOsI386),
            Darwin64 => Self::Platform(ReaperPlatform::MacOsX86_64),
            DarwinArm64 => Self::Platform(ReaperPlatform::MacOsArm64),
            Linux => Self::Os(ReaperOs::Linux),
            Linux32 => Self::Platform(ReaperPlatform::LinuxI686),
            Linux64 => Self::Platform(ReaperPlatform::LinuxX86_64),
            LinuxArmv7l => Self::Platform(ReaperPlatform::LinuxArmv7l),
            LinuxAarch64 => Self::Platform(ReaperPlatform::LinuxAarch64),
            Windows => Self::Os(ReaperOs::Windows),
            Win32 => Self::Platform(ReaperPlatform::WindowsX86),
            Win64 => Self::Platform(ReaperPlatform::WindowsX64),
        };
        Some(value)
    }
}

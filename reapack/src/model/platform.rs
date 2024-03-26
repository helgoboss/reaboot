use serde::Deserialize;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Platform {
    #[default]
    All,
    Darwin,
    Darwin32,
    Darwin64,
    DarwinArm64,
    Linux,
    Linux32,
    Linux64,
    LinuxArmv7l,
    LinuxAarch64,
    Windows,
    Win32,
    Win64,
}

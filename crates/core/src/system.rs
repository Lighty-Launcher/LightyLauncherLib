//! Compile-time OS / architecture detection and per-vendor name mapping.

use crate::errors::{SystemError, SystemResult};
use serde::Deserialize;
use std::fmt::Display;

/// Current operating system, resolved at compile time.
pub const OS: OperatingSystem = if cfg!(target_os = "windows") {
    OperatingSystem::WINDOWS
} else if cfg!(target_os = "macos") {
    OperatingSystem::OSX
} else if cfg!(target_os = "linux") {
    OperatingSystem::LINUX
} else {
    OperatingSystem::UNKNOWN
};

/// Current CPU architecture, resolved at compile time.
pub const ARCHITECTURE: Architecture = if cfg!(target_arch = "x86") {
    Architecture::X86
} else if cfg!(target_arch = "x86_64") {
    Architecture::X64
} else if cfg!(target_arch = "arm") {
    Architecture::ARM
} else if cfg!(target_arch = "aarch64") {
    Architecture::AARCH64
} else {
    Architecture::UNKNOWN
};

/// Supported operating systems.
#[derive(Deserialize, PartialEq, Eq, Hash, Debug)]
pub enum OperatingSystem {
    #[serde(rename = "windows")]
    WINDOWS,
    #[serde(rename = "linux")]
    LINUX,
    #[serde(rename = "osx")]
    OSX,
    #[serde(rename = "unknown")]
    UNKNOWN,
}

/// Supported CPU architectures.
#[derive(Deserialize, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Architecture {
    #[serde(rename = "x86")]
    X86,
    #[serde(rename = "x64")]
    X64,
    #[serde(rename = "arm")]
    ARM,
    #[serde(rename = "aarch64")]
    AARCH64,
    #[serde(rename = "unknown")]
    UNKNOWN,
}

impl OperatingSystem {
    /// Returns the OS name as used in Mojang's version manifest
    /// (`"windows"`, `"linux"`, `"osx"`).
    pub fn get_vanilla_os(&self) -> SystemResult<&'static str> {
        match self {
            OperatingSystem::WINDOWS => Ok("windows"),
            OperatingSystem::LINUX => Ok("linux"),
            OperatingSystem::OSX => Ok("osx"),
            OperatingSystem::UNKNOWN => Err(SystemError::UnsupportedOS),
        }
    }

    /// Returns the OS name as used by the Adoptium (Temurin) API
    /// (`"windows"`, `"linux"`, `"mac"`).
    pub fn get_adoptium_name(&self) -> SystemResult<&'static str> {
        match self {
            OperatingSystem::WINDOWS => Ok("windows"),
            OperatingSystem::LINUX => Ok("linux"),
            OperatingSystem::OSX => Ok("mac"),
            OperatingSystem::UNKNOWN => Err(SystemError::UnsupportedOS),
        }
    }

    /// Returns the OS name as used by the Oracle GraalVM download URLs
    /// (`"windows"`, `"linux"`, `"macos"`).
    pub fn get_graal_name(&self) -> SystemResult<&'static str> {
        match self {
            OperatingSystem::WINDOWS => Ok("windows"),
            OperatingSystem::LINUX => Ok("linux"),
            OperatingSystem::OSX => Ok("macos"),
            OperatingSystem::UNKNOWN => Err(SystemError::UnsupportedOS),
        }
    }

    /// Returns the OS name as used by the Azul / Foojay APIs (Zulu, Liberica)
    /// (`"windows"`, `"linux"`, `"macos"`).
    pub fn get_zulu_name(&self) -> SystemResult<&'static str> {
        match self {
            OperatingSystem::WINDOWS => Ok("windows"),
            OperatingSystem::LINUX => Ok("linux"),
            OperatingSystem::OSX => Ok("macos"),
            OperatingSystem::UNKNOWN => Err(SystemError::UnsupportedOS),
        }
    }

    /// Returns the archive extension Zulu publishes for this OS
    /// (`"zip"` on Windows, `"tar.gz"` on Linux/macOS).
    pub fn get_zulu_ext(&self) -> SystemResult<&'static str> {
        match self {
            OperatingSystem::WINDOWS => Ok("zip"),
            OperatingSystem::LINUX | OperatingSystem::OSX => Ok("tar.gz"),
            OperatingSystem::UNKNOWN => Err(SystemError::UnsupportedOS),
        }
    }

    /// Returns the archive extension used by every supported JRE distribution
    /// (`"zip"` on Windows, `"tar.gz"` on Linux/macOS).
    pub fn get_archive_type(&self) -> SystemResult<&'static str> {
        match self {
            OperatingSystem::WINDOWS => Ok("zip"),
            OperatingSystem::LINUX | OperatingSystem::OSX => Ok("tar.gz"),
            OperatingSystem::UNKNOWN => Err(SystemError::UnsupportedOS),
        }
    }
}

impl Display for OperatingSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OperatingSystem::WINDOWS => f.write_str("windows"),
            OperatingSystem::LINUX => f.write_str("linux"),
            OperatingSystem::OSX => f.write_str("osx"),
            OperatingSystem::UNKNOWN => f.write_str("unknown"),
        }
    }
}

impl Architecture {
    /// Returns the canonical architecture name (`"x86"`, `"x64"`, `"arm"`, `"aarch64"`).
    pub fn get_simple_name(&self) -> SystemResult<&'static str> {
        match self {
            Architecture::X86 => Ok("x86"),
            Architecture::X64 => Ok("x64"),
            Architecture::ARM => Ok("arm"),
            Architecture::AARCH64 => Ok("aarch64"),
            Architecture::UNKNOWN => Err(SystemError::UnsupportedArchitecture),
        }
    }

    /// Returns the architecture suffix Mojang appends to native classifier
    /// names (`""` for x64, `"-x86"`, `"-arm"`, `"-arm64"`).
    pub fn get_vanilla_arch(&self) -> SystemResult<&'static str> {
        match self {
            Architecture::X86 => Ok("-x86"),
            Architecture::X64 => Ok(""),
            Architecture::ARM => Ok("-arm"),
            Architecture::AARCH64 => Ok("-arm64"),
            Architecture::UNKNOWN => Err(SystemError::UnsupportedArchitecture),
        }
    }

    /// Returns `"32"` or `"64"` — used to resolve the `${arch}` placeholder
    /// in Mojang's library native classifier names.
    pub fn get_arch_bits(&self) -> SystemResult<&'static str> {
        match self {
            Architecture::X86 => Ok("32"),
            Architecture::X64 => Ok("64"),
            Architecture::ARM => Ok("32"),
            Architecture::AARCH64 => Ok("64"),
            Architecture::UNKNOWN => Err(SystemError::UnsupportedArchitecture),
        }
    }

    /// Returns the architecture name as used by the Azul Zulu API
    /// (`"i686"`, `"x64"`, `"arm"`, `"aarch64"`).
    pub fn get_zulu_arch(&self) -> SystemResult<&'static str> {
        match self {
            //TODO: rework this part for java 8 for macos
            Architecture::X86 => Ok("i686"),
            Architecture::X64 => Ok("x64"),
            Architecture::ARM => Ok("arm"),
            Architecture::AARCH64 => Ok("aarch64"),
            Architecture::UNKNOWN => Err(SystemError::UnsupportedArchitecture),
        }
    }
}

impl Display for Architecture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Architecture::X86 => f.write_str("x86"),
            Architecture::X64 => f.write_str("x64"),
            Architecture::ARM => f.write_str("arm"),
            Architecture::AARCH64 => f.write_str("aarch64"),
            Architecture::UNKNOWN => f.write_str("unknown"),
        }
    }
}

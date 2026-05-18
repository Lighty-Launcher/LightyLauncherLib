# system — OS + architecture detection

Compile-time platform probe used by every downloader / extractor /
launcher path resolver in the workspace.

## Constants

```rust,no_run
use lighty_core::system::{OS, ARCHITECTURE, OperatingSystem, Architecture};

fn main() {
    println!("os: {OS}");
    println!("arch: {ARCHITECTURE}");

    if matches!(OS, OperatingSystem::WINDOWS) {
        println!("running on Windows");
    }
}
```

Both `OS` and `ARCHITECTURE` are `const`: resolved at build time via
`cfg!(target_os = …)` / `cfg!(target_arch = …)` — no runtime cost.

## Enums

```rust,ignore
pub enum OperatingSystem { WINDOWS, LINUX, OSX, UNKNOWN }
pub enum Architecture    { X86, X64, ARM, AARCH64, UNKNOWN }
```

`Display` is implemented for both (lowercase canonical name).

## Per-vendor name maps

Different download providers spell platforms differently. Each
fallible accessor returns `SystemResult<&'static str>` and surfaces
`SystemError::UnsupportedOS` / `UnsupportedArchitecture` on `UNKNOWN`.

### `OperatingSystem`

| Method | Windows | Linux | macOS | Used for |
|---|---|---|---|---|
| `get_vanilla_os` | `"windows"` | `"linux"` | `"osx"` | Mojang version manifest |
| `get_adoptium_name` | `"windows"` | `"linux"` | `"mac"` | Adoptium / Temurin API |
| `get_graal_name` | `"windows"` | `"linux"` | `"macos"` | Oracle GraalVM URLs |
| `get_zulu_name` | `"windows"` | `"linux"` | `"macos"` | Azul Zulu / Foojay APIs |
| `get_zulu_ext` | `"zip"` | `"tar.gz"` | `"tar.gz"` | Azul archive extension |
| `get_archive_type` | `"zip"` | `"tar.gz"` | `"tar.gz"` | Generic JRE archive extension |

### `Architecture`

| Method | x86 | x64 | arm | aarch64 | Used for |
|---|---|---|---|---|---|
| `get_simple_name` | `"x86"` | `"x64"` | `"arm"` | `"aarch64"` | Generic display |
| `get_vanilla_arch` | `"-x86"` | `""` | `"-arm"` | `"-arm64"` | Mojang native classifier suffix |
| `get_arch_bits` | `"32"` | `"64"` | `"32"` | `"64"` | `${arch}` placeholder in native classifier names |
| `get_zulu_arch` | `"i686"` | `"x64"` | `"arm"` | `"aarch64"` | Azul Zulu API |

## Errors

```rust,ignore
pub enum SystemError {
    UnsupportedOS,
    UnsupportedArchitecture,
    Io(std::io::Error),
}
```

Most callers convert into the broader `QueryError` / `DownloadError`
chains used by the launch pipeline.

## See also

- [`how-to-use.md`](./how-to-use.md) — usage section *Detect OS / architecture*
- [`../../java/docs/distributions.md`](../../java/docs/distributions.md)
  — how each Java distribution consumes these names

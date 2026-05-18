# Distributions

Four JRE providers, identical surface (`JavaDistribution`). Pick one
per `(version, OS, arch)` — the downloader handles fallback when the
combo isn't published.

## Provider matrix

| Provider | Vendor | Java versions | Best for |
|---|---|---|---|
| `Temurin` | Eclipse Adoptium | 8, 11, 17, 21+ | General default — widest version coverage |
| `GraalVM` | Oracle | 17, 21+ | Performance-sensitive runs (Graal JIT) |
| `Zulu` | Azul Systems | 8, 11, 17, 21+ | TCK-certified, excellent ARM coverage |
| `Liberica` | BellSoft | 8, 11, 17, 21+ | Smallest download size, embedded use |

Known platform gaps:

- **Temurin Java 8 on macOS aarch64** — not published. Apple Silicon
  came after Java 8 EOL'd.
- **GraalVM ≤ 11** — Graal CE only ships Java 17+.
- **Windows aarch64 GraalVM** — currently unsupported.

`JavaDistribution::supports_version(version)` returns whether the
current platform has a build, and `get_fallback(version)` returns the
replacement when it doesn't (chain: Zulu → Liberica → Temurin, in
decreasing platform coverage).

## API endpoints

### Temurin (Adoptium)

```
GET https://api.adoptium.net/v3/assets/
        feature_releases/{version}/ga
        ?image_type=jre&os={os}&architecture={arch}&vendor=eclipse
```

OS values from `OperatingSystem::get_adoptium_name`
(`"windows"`/`"linux"`/`"mac"`).

### GraalVM (Oracle / GitHub Releases)

Direct download URL built from the release pattern:
```
https://download.oracle.com/graalvm/{version}/latest/graalvm-jdk-{version}_{os}-{arch}_bin.{ext}
```

OS from `OperatingSystem::get_graal_name`
(`"windows"`/`"linux"`/`"macos"`); ext from
`OperatingSystem::get_archive_type`.

### Zulu (Azul / Foojay)

```
GET https://api.azul.com/metadata/v1/zulu/packages/
        ?java_version={version}&os={os}&arch={arch}
        &bundle_type=jre&javafx=false&release_status=ga
```

OS from `OperatingSystem::get_zulu_name`; arch from
`Architecture::get_zulu_arch` (`"i686"`/`"x64"`/`"arm"`/`"aarch64"`).

### Liberica (BellSoft / Foojay Disco)

```
GET https://api.foojay.io/disco/v3.0/packages
        ?version={version}&distribution=liberica
        &operating_system={os}&architecture={arch}
        &archive_type={ext}&package_type=jre
```

## Platform support

### Windows

| Distribution | x64 | aarch64 |
|---|---|---|
| Temurin | OK | OK |
| GraalVM | OK | — |
| Zulu | OK | OK |
| Liberica | OK | OK |

### Linux

| Distribution | x64 | aarch64 |
|---|---|---|
| Temurin | OK | OK |
| GraalVM | OK | OK |
| Zulu | OK | OK |
| Liberica | OK | OK |

### macOS

| Distribution | x64 (Intel) | aarch64 (Apple Silicon) |
|---|---|---|
| Temurin | OK | OK (Java 11+) |
| GraalVM | OK (Java 17+) | OK (Java 17+) |
| Zulu | OK | OK |
| Liberica | OK | OK |

## Pick a Java version for Minecraft

The launcher reads the target Java version from the Mojang version
manifest (`javaVersion.majorVersion`); typical mapping:

| Minecraft | Required Java |
|---|---|
| < 1.12 | 8 |
| 1.12 – 1.16.5 | 8 |
| 1.17 – 1.17.1 | 16 |
| 1.18 – 1.20.x | 17 |
| 1.21+ | 21 |

For most installs `Temurin` is the safe choice; switch to `GraalVM` on
modern (1.17+) versions if you want the JIT improvements.

## Picking from the API

```rust
use lighty_java::JavaDistribution;

let dist = JavaDistribution::Temurin;
let url = dist.get_download_url(&21).await?;
println!("install URL: {}", url);
# Ok::<_, lighty_java::DistributionError>(())
```

## Errors

```rust
pub enum DistributionError {
    UnsupportedVersion { version: u8, distribution: &'static str },
    ApiError          { distribution: &'static str, error: String },
    JsonParseError    { distribution: &'static str, error: String },
    NoPackagesFound   { distribution: &'static str },
    System(lighty_core::SystemError),
}
```

`UnsupportedVersion` should not normally reach calling code — the
`get_fallback` helper short-circuits before the URL fetch.

## See also

- [`overview.md`](./overview.md) — what the crate is for
- [`installation.md`](./installation.md) — download walkthrough
- [`../../core/docs/system.md`](../../core/docs/system.md) —
  `OperatingSystem` / `Architecture` name maps

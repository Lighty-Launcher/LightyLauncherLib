# Arguments

The argument system builds the final argv (`[JVM args] + [main class]
+ [game args] + [raw args]`) from a resolved `Version` plus per-launch
overrides. This page is the canonical reference for placeholders, JVM
options and game options — other launch docs cross-ref here.

## The `Arguments` trait

```rust,ignore
pub trait Arguments {
    fn build_arguments(
        &self,
        builder: &Version,
        profile: Option<&UserProfile>,
        arg_overrides:  &HashMap<String, String>,
        arg_removals:   &HashSet<String>,
        jvm_overrides:  &HashMap<String, String>,
        jvm_removals:   &HashSet<String>,
        raw_args:       &[String],
    ) -> Vec<String>;
}
```

Blanket-implemented for every `VersionInfo` — you almost never call
it directly; `LaunchBuilder::run` does. `profile = None` keeps the
hardcoded defaults (`access_token = "0"`, `user_type = "legacy"`),
useful for dry-run inspection.

## Standard placeholders

The loader metadata uses `${name}` placeholders that the builder
substitutes from a variable map. The full set:

### Authentication

| Placeholder | Description | Example |
|---|---|---|
| `${auth_player_name}` | Player username | `"Player123"` |
| `${auth_uuid}` | Player UUID | `"550e8400-e29b-41d4-a716-446655440000"` |
| `${auth_access_token}` | Access token | `"eyJhbGc…"` or `"0"` offline |
| `${auth_xuid}` | Xbox User ID | `"2535405290…"` or `"0"` |
| `${clientid}` | Client ID | `"{client-id}"` |
| `${user_type}` | User type | `"msa"`, `"mojang"` or `"legacy"` |
| `${user_properties}` | User properties JSON | `"{}"` |

`${user_type}` is derived from `profile.provider`:
`Microsoft → "msa"`, `Azuriom → "mojang"`,
`Offline / Custom → "legacy"`.

### Directories

| Placeholder | Description |
|---|---|
| `${game_directory}` | Per-instance runtime dir (honours `KEY_GAME_DIRECTORY` override) |
| `${assets_root}` | Shared assets dir |
| `${natives_directory}` | Extracted natives, temp dir per launch |
| `${library_directory}` | Shared libraries dir |
| `${classpath}` | Java classpath |
| `${classpath_separator}` | `:` (Linux / macOS) or `;` (Windows) |

### Version / launcher

| Placeholder | Description |
|---|---|
| `${version_name}` | Minecraft version |
| `${version_type}` | `"release"`, `"snapshot"`, … |
| `${assets_index_name}` | Asset index ID |
| `${launcher_name}` | Launcher name (defaults to `AppState::name()`) |
| `${launcher_version}` | Launcher version |

## Placeholder key constants

Every placeholder has a typed constant in `lighty_launch::arguments`
to avoid stringly-typed code:

```rust,no_run
use lighty_launch::arguments::{
    KEY_AUTH_PLAYER_NAME, KEY_AUTH_UUID, KEY_AUTH_ACCESS_TOKEN, KEY_AUTH_XUID,
    KEY_CLIENT_ID, KEY_USER_TYPE, KEY_USER_PROPERTIES,
    KEY_VERSION_NAME, KEY_VERSION_TYPE, KEY_ASSETS_INDEX_NAME,
    KEY_GAME_DIRECTORY, KEY_ASSETS_ROOT, KEY_NATIVES_DIRECTORY, KEY_LIBRARY_DIRECTORY,
    KEY_LAUNCHER_NAME, KEY_LAUNCHER_VERSION,
    KEY_CLASSPATH, KEY_CLASSPATH_SEPARATOR,
};
```

`.set(KEY_LAUNCHER_NAME, "MyApp")` overrides the placeholder value.
Anything else is appended as a `--key value` (or `--flag` for empty
value) game argument.

## JVM options

`with_jvm_options()` returns a builder where `.set(key, value)` writes
to a `HashMap<String, String>`. The `-` prefix is added automatically
based on the key shape:

| Input | Becomes |
|---|---|
| `set("Xmx", "4G")` | `-Xmx4G` |
| `set("XX:+UseG1GC", "")` | `-XX:+UseG1GC` |
| `set("Djava.library.path", "/p")` | `-Djava.library.path=/p` |

The defaults (used when the loader metadata supplies no JVM section):

```text
-Djava.library.path=${natives_directory}
-Dminecraft.launcher.brand=${launcher_name}
-Dminecraft.launcher.version=${launcher_version}
-Xmx2G
-XX:+UnlockExperimentalVMOptions
-XX:+UseG1GC
-XX:G1NewSizePercent=20
-XX:G1ReservePercent=20
-XX:MaxGCPauseMillis=50
-XX:G1HeapRegionSize=32M
-cp ${classpath}
```

A handful of JVM args are **always present** even if the loader
metadata omits them — the builder injects them at the top of the
list:

- `-Djava.library.path=${natives_directory}` (LWJGL natives)
- `-Dminecraft.launcher.brand=${launcher_name}`
- `-Dminecraft.launcher.version=${launcher_version}`
- `-XstartOnFirstThread` on macOS (LWJGL / GLFW requirement)
- `-cp ${classpath}` — always the last JVM arg before the main class

Common knobs:

| Option | Purpose | Example |
|---|---|---|
| `-Xmx` | Max heap | `"4G"`, `"8G"` |
| `-Xms` | Initial heap | `"2G"` |
| `-XX:+UseG1GC` / `-XX:+UseZGC` | GC choice | `""` |
| `-XX:MaxGCPauseMillis` | GC pause target (ms) | `"50"` |
| `-XX:G1HeapRegionSize` | G1 region size | `"32M"` |
| `-Dfile.encoding` | File encoding | `"UTF-8"` |
| `-Djava.net.preferIPv4Stack` | Prefer IPv4 | `"true"` |

## Game options

`with_arguments()` exposes the same `.set` / `.remove` shape but for
game args. Two cases:

1. **Known launch placeholder constant** (`KEY_LAUNCHER_NAME`, etc.)
   — substitutes the variable in the existing argv.
2. **Anything else** — appended as a fresh argument (`--key value`,
   or `--key` for empty value).

Common knobs:

| Option | Purpose |
|---|---|
| `--width`, `--height` | Window size |
| `--fullscreen` | Fullscreen mode |
| `--quickPlayPath` | Quick play server file |
| `--quickPlaySingleplayer` | Quick play world |
| `--quickPlayMultiplayer` | Quick play server |
| `--quickPlayRealms` | Quick play realm |
| `--demo` | Demo mode |
| `--server` / `--port` | Auto-connect server / port |

## Access-token routing

The `--accessToken` value is the most sensitive piece of data the
launch pipeline touches, so the resolution is narrow. At
`crates/launch/src/arguments/arguments.rs` the variable-map builder
does:

1. If `profile.token_handle` is `Some` **and** the `keyring` feature
   is active on `lighty-launch`, read the token from the OS keychain
   via `TokenHandle::read()` — the token never sat in `UserProfile`
   heap memory between authentication and launch.
2. Otherwise, clone the in-memory `SecretString` stored in
   `profile.access_token`.
3. `ExposeSecret::expose_secret(&secret)` is called **exactly once**,
   at the moment the value is inserted into the placeholder map. No
   copy of the plaintext is kept past that call.
4. When `profile = None` (offline / dry-run callers), the hardcoded
   default `"0"` is used.

```toml
[dependencies]
lighty-launch = { version = "...", features = ["events", "keyring"] }
```

See `AUTH_SECRETS.md` at the workspace root for the full threat model.

## Final assembly example

```text
java
  -Djava.library.path=/tmp/natives-xxxxx
  -Dminecraft.launcher.brand=MyLauncher
  -Dminecraft.launcher.version=26.5.6
  -Xmx4G -Xms2G -XX:+UseG1GC
  -cp /path/lib1.jar:/path/lib2.jar:...
  net.minecraft.client.main.Main
  --username Player123
  --version 1.21.1
  --gameDir   /home/user/.local/share/MyLauncher/instance
  --assetsDir /home/user/.local/share/MyLauncher/assets
  --assetIndex 16
  --uuid 550e8400-…
  --accessToken 0
  --width 1920 --height 1080
```

## Putting it together

```rust,no_run
# use lighty_auth::UserProfile;
# use lighty_core::AppState;
# use lighty_java::JavaDistribution;
# use lighty_launch::errors::InstallerResult;
# use lighty_launch::launch::Launch;
# use lighty_loaders::types::Loader;
# use lighty_version::VersionBuilder;
use lighty_launch::arguments::KEY_LAUNCHER_NAME;
# async fn run() -> InstallerResult<()> {
# AppState::init("MyLauncher").ok();
# let profile = UserProfile::offline("Player", "");
# let mut instance = VersionBuilder::new("inst", Loader::Fabric, "0.16.9", "1.21.1");

instance.launch(&profile, JavaDistribution::Temurin)
    .with_jvm_options()
        .set("Xmx", "6G")
        .set("XX:+UseG1GC", "")
        .set("XX:+AlwaysPreTouch", "")
        .set("Dfile.encoding", "UTF-8")
        .done()
    .with_arguments()
        .set(KEY_LAUNCHER_NAME, "MyCustomLauncher")
        .set("width",  "1920")
        .set("height", "1080")
        .done()
    .run()
    .await?;
# Ok(()) }
```

## Related

- [How to use](./how-to-use.md) — short usage snippets
- [Launch](./launch.md) — pipeline detail, builder API
- [Installation](./installation.md) — what gets built into the classpath
- [Exports](./exports.md)

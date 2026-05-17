// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

use lighty_auth::{AuthProvider, UserProfile};
use lighty_loaders::types::version_metadata::Version;
use lighty_loaders::types::VersionInfo;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

// Public placeholder keys matching `${...}` tokens in Mojang's version manifest.
// Pass them to `ArgumentsBuilder::set(key, value)` to override substitution.

/// Player username (`${auth_player_name}`).
pub const KEY_AUTH_PLAYER_NAME: &str = "auth_player_name";
/// Player UUID (`${auth_uuid}`).
pub const KEY_AUTH_UUID: &str = "auth_uuid";
/// Mojang/Microsoft access token (`${auth_access_token}`).
pub const KEY_AUTH_ACCESS_TOKEN: &str = "auth_access_token";
/// Xbox Live user ID (`${auth_xuid}`).
pub const KEY_AUTH_XUID: &str = "auth_xuid";
/// OAuth client ID (`${clientid}`).
pub const KEY_CLIENT_ID: &str = "clientid";
/// User type (`${user_type}`).
pub const KEY_USER_TYPE: &str = "user_type";
/// User properties JSON (`${user_properties}`).
pub const KEY_USER_PROPERTIES: &str = "user_properties";
/// Version name (`${version_name}`).
pub const KEY_VERSION_NAME: &str = "version_name";
/// Version type — `"release"`, `"snapshot"`, ... (`${version_type}`).
pub const KEY_VERSION_TYPE: &str = "version_type";
/// Game run directory (`${game_directory}`).
pub const KEY_GAME_DIRECTORY: &str = "game_directory";
/// Assets root directory (`${assets_root}`).
pub const KEY_ASSETS_ROOT: &str = "assets_root";
/// Native libraries directory (`${natives_directory}`).
pub const KEY_NATIVES_DIRECTORY: &str = "natives_directory";
/// Maven libraries directory (`${library_directory}`).
pub const KEY_LIBRARY_DIRECTORY: &str = "library_directory";
/// Assets index id (`${assets_index_name}`).
pub const KEY_ASSETS_INDEX_NAME: &str = "assets_index_name";
/// Launcher brand name (`${launcher_name}`).
pub const KEY_LAUNCHER_NAME: &str = "launcher_name";
/// Launcher version (`${launcher_version}`).
pub const KEY_LAUNCHER_VERSION: &str = "launcher_version";
/// Final classpath value (`${classpath}`).
pub const KEY_CLASSPATH: &str = "classpath";
/// OS-specific classpath separator (`${classpath_separator}`).
pub const KEY_CLASSPATH_SEPARATOR: &str = "classpath_separator";

// Default values used when no real session data is available
const DEFAULT_ACCESS_TOKEN: &str = "0";
const DEFAULT_XUID: &str = "0";
const DEFAULT_USER_TYPE: &str = "legacy";
const DEFAULT_USER_PROPERTIES: &str = "{}";
const DEFAULT_VERSION_TYPE: &str = "release";
const CP_FLAG: &str = "-cp";

/// Maps an `AuthProvider` to the `${user_type}` string the game expects.
///
/// - `Microsoft` → `"msa"` (Microsoft Account, modern online auth)
/// - `Azuriom` → `"mojang"` (legacy Mojang-style sessions, what Azuriom mimics)
/// - `Offline` → `"legacy"` (matches the historical default for unauth'd play)
/// - `Custom` → `"legacy"` (callers can override via `set(KEY_USER_TYPE, ...)`
///   on the `ArgumentsBuilder` when they need something specific)
fn user_type_for(provider: &AuthProvider) -> &'static str {
    match provider {
        AuthProvider::Microsoft { .. } => "msa",
        AuthProvider::Azuriom { .. } => "mojang",
        AuthProvider::Offline | AuthProvider::Custom { .. } => "legacy",
    }
}

/// Builds the final argv (JVM args + main class + game args + raw args)
/// from a resolved [`Version`] plus runtime overrides and removals.
///
/// Implemented blanket-style for every [`VersionInfo`]; user code rarely
/// invokes [`Self::build_arguments`] directly — `LaunchBuilder::run`
/// does it internally.
pub trait Arguments {
    /// Constructs the launch argv for `builder` using the supplied
    /// overrides and removals.
    ///
    /// Pass the authenticated `profile` so auth-derived placeholders
    /// (`${auth_player_name}`, `${auth_uuid}`, `${auth_access_token}`,
    /// `${auth_xuid}`, `${user_type}`) are populated from real session
    /// data. `None` keeps the legacy hardcoded defaults (`"0"`,
    /// `"legacy"`, …) — useful for dry-run argv inspection or tests.
    fn build_arguments(
        &self,
        builder: &Version,
        profile: Option<&UserProfile>,
        arg_overrides: &HashMap<String, String>,
        arg_removals: &HashSet<String>,
        jvm_overrides: &HashMap<String, String>,
        jvm_removals: &HashSet<String>,
        raw_args: &[String],
    ) -> Vec<String>;
}

impl<T: VersionInfo> Arguments for T {
    fn build_arguments(
        &self,
        builder: &Version,
        profile: Option<&UserProfile>,
        arg_overrides: &HashMap<String, String>,
        arg_removals: &HashSet<String>,
        jvm_overrides: &HashMap<String, String>,
        jvm_removals: &HashSet<String>,
        raw_args: &[String],
    ) -> Vec<String> {
        let mut variables = create_variable_map(self, builder, profile);

        for (key, value) in arg_overrides {
            variables.insert(key.clone(), value.clone());
        }

        // KEY_GAME_DIRECTORY is special: the runner already resolved it against
        // version.game_dirs() and wrote the absolute path via set_runtime_dir.
        // Force the resolved value so the JVM gets the absolute path.
        variables.insert(
            KEY_GAME_DIRECTORY.into(),
            self.runtime_dir().display().to_string(),
        );

        let game_args = replace_variables_in_vec(&builder.arguments.game, &variables);

        let mut jvm_args = builder.arguments.jvm
            .as_ref()
            .map(|jvm| replace_variables_in_vec(jvm, &variables))
            .unwrap_or_else(|| build_default_jvm_args(&variables));

        // macOS: -XstartOnFirstThread is MANDATORY for LWJGL/OpenGL
        #[cfg(target_os = "macos")]
        if !jvm_args.iter().any(|arg| arg == "-XstartOnFirstThread") {
            jvm_args.insert(0, "-XstartOnFirstThread".to_string());
        }

        // LWJGL needs java.library.path to find natives.
        if !jvm_args.iter().any(|arg| arg.starts_with("-Djava.library.path=")) {
            let natives_dir = variables.get(KEY_NATIVES_DIRECTORY).cloned().unwrap_or_default();
            jvm_args.insert(0, format!("-Djava.library.path={}", natives_dir));
        }

        if !jvm_args.iter().any(|arg| arg.starts_with("-Dminecraft.launcher.brand=")) {
            let launcher_name = variables.get(KEY_LAUNCHER_NAME).cloned().unwrap_or_default();
            jvm_args.insert(0, format!("-Dminecraft.launcher.brand={}", launcher_name));
        }

        if !jvm_args.iter().any(|arg| arg.starts_with("-Dminecraft.launcher.version=")) {
            let launcher_version = variables.get(KEY_LAUNCHER_VERSION).cloned().unwrap_or_default();
            jvm_args.insert(0, format!("-Dminecraft.launcher.version={}", launcher_version));
        }

        // Classpath must be the last JVM arg before the main class.
        let module_path_opt = jvm_args
            .iter()
            .position(|arg| arg == "-p")
            .and_then(|p_idx| jvm_args.get(p_idx + 1).cloned());

        if let Some(cp_idx) = jvm_args.iter().position(|arg| arg == CP_FLAG) {
            if let Some(ref module_path) = module_path_opt {
                lighty_core::trace_debug!("Module-path detected: {}", module_path);
                if let Some(existing_cp) = jvm_args.get(cp_idx + 1) {
                    let filtered_classpath =
                        filter_classpath_from_modulepath(existing_cp, module_path);
                    lighty_core::trace_debug!("Classpath filtered for module-path");

                    jvm_args[cp_idx + 1] = filtered_classpath;
                } else {
                    lighty_core::trace_warn!("-cp found but no value after it");
                }
            } else {
                lighty_core::trace_debug!("No module-path, keeping existing classpath unchanged");
            }
        } else {
            let classpath = variables.get(KEY_CLASSPATH).cloned().unwrap_or_default();

            if let Some(ref module_path) = module_path_opt {
                lighty_core::trace_debug!("Module-path detected: {}", module_path);
                let filtered_classpath = filter_classpath_from_modulepath(&classpath, module_path);
                lighty_core::trace_debug!("Classpath filtered for module-path");

                jvm_args.push(CP_FLAG.into());
                jvm_args.push(filtered_classpath);
            } else {
                jvm_args.push(CP_FLAG.into());
                jvm_args.push(classpath);
            }
        }

        apply_jvm_overrides(&mut jvm_args, jvm_overrides);
        apply_jvm_removals(&mut jvm_args, jvm_removals);
        let game_args = apply_arg_removals(game_args, arg_removals);

        let mut full_args = jvm_args;
        full_args.push(builder.main_class.main_class.clone());
        full_args.extend(game_args);

        full_args.extend_from_slice(raw_args);

        lighty_core::trace_debug!(args = ?full_args, "Launch arguments built");

        full_args
    }
}

/// Builds the launch-argument placeholder map.
///
/// `profile` drives every auth-related placeholder. When `None`, we keep
/// the legacy hardcoded defaults (this is what the previous offline-only
/// code path effectively produced, and what callers without an auth flow
/// — tests, dry-run tools — get for free).
fn create_variable_map<T: VersionInfo>(
    version: &T,
    builder: &Version,
    profile: Option<&UserProfile>,
) -> HashMap<String, String> {
        let mut map = HashMap::new();

        #[cfg(target_os = "windows")]
        let classpath_separator = ";";
        #[cfg(not(target_os = "windows"))]
        let classpath_separator = ":";

        // Auth placeholders fall back to legacy defaults when profile is None
        // so offline/no-auth callers see the same argv as before.
        let username = profile.map(|p| p.username.as_str()).unwrap_or("");
        let uuid = profile.map(|p| p.uuid.as_str()).unwrap_or("");

        // Token resolution: prefer the OS-keychain handle (read on demand)
        // when present; fall back to the in-memory `SecretString`. The
        // secret is exposed for the strict scope of the argv insertion.
        let token_secret: Option<lighty_auth::SecretString> = profile.and_then(|p| {
            #[cfg(feature = "keyring")]
            if let Some(h) = &p.token_handle {
                return h.read().ok();
            }
            p.access_token.clone()
        });
        let access_token = token_secret
            .as_ref()
            .map(|s| lighty_auth::ExposeSecret::expose_secret(s))
            .unwrap_or(DEFAULT_ACCESS_TOKEN);
        let xuid = profile
            .and_then(|p| p.xuid.as_deref())
            .unwrap_or(DEFAULT_XUID);
        let user_type = profile
            .map(|p| user_type_for(&p.provider))
            .unwrap_or(DEFAULT_USER_TYPE);

        map.insert(KEY_AUTH_PLAYER_NAME.into(), username.into());
        map.insert(KEY_AUTH_UUID.into(), uuid.into());
        map.insert(KEY_AUTH_ACCESS_TOKEN.into(), access_token.into());
        map.insert(KEY_AUTH_XUID.into(), xuid.into());
        map.insert(KEY_CLIENT_ID.into(), lighty_core::AppState::client_id().to_string());
        map.insert(KEY_USER_TYPE.into(), user_type.into());
        map.insert(KEY_USER_PROPERTIES.into(), DEFAULT_USER_PROPERTIES.into());

        map.insert(KEY_VERSION_NAME.into(), version.name().into());
        map.insert(KEY_VERSION_TYPE.into(), DEFAULT_VERSION_TYPE.into());

        // runtime_dir() is the single source of truth shared with the install
        // pipeline so mods land where the game actually scans for them.
        map.insert(KEY_GAME_DIRECTORY.into(), version.runtime_dir().display().to_string());
        map.insert(KEY_ASSETS_ROOT.into(), version.game_dirs().join("assets").display().to_string());
        map.insert(KEY_NATIVES_DIRECTORY.into(), version.game_dirs().join("natives").display().to_string());
        map.insert(KEY_LIBRARY_DIRECTORY.into(), version.game_dirs().join("libraries").display().to_string());

        let assets_index_name = builder.assets_index
            .as_ref()
            .map(|idx| idx.id.clone())
            .unwrap_or_else(|| version.minecraft_version().into());
        map.insert(KEY_ASSETS_INDEX_NAME.into(), assets_index_name);

        map.insert(KEY_LAUNCHER_NAME.into(), lighty_core::AppState::name().to_string());
        map.insert(KEY_LAUNCHER_VERSION.into(), lighty_core::AppState::app_version().to_string());

        let classpath = build_classpath(version, &builder.libraries);
        map.insert(KEY_CLASSPATH.into(), classpath);
        map.insert(KEY_CLASSPATH_SEPARATOR.into(), classpath_separator.to_string());

        map
}

/// Builds the runtime classpath from the resolved library list.
fn build_classpath<T: VersionInfo>(version: &T, libraries: &[lighty_loaders::types::version_metadata::Library]) -> String {
        #[cfg(target_os = "windows")]
        let separator = ";";
        #[cfg(not(target_os = "windows"))]
        let separator = ":";

        let lib_dir = version.game_dirs().join("libraries");

        let mut classpath_entries: Vec<String> = libraries
            .iter()
            .filter_map(|lib| {
                lib.path.as_ref().map(|path| {
                    lib_dir.join(path).display().to_string()
                })
            })
            .collect();

        classpath_entries.push(
            version.game_dirs().join(format!("{}.jar", version.name())).display().to_string()
        );

        classpath_entries.join(separator)
}

/// Default JVM arguments used for legacy versions that don't ship one.
fn build_default_jvm_args(variables: &HashMap<String, String>) -> Vec<String> {
        let natives_dir = variables.get(KEY_NATIVES_DIRECTORY).cloned().unwrap_or_default();
        let launcher_name = variables.get(KEY_LAUNCHER_NAME).cloned().unwrap_or_default();
        let launcher_version = variables.get(KEY_LAUNCHER_VERSION).cloned().unwrap_or_default();
        let classpath = variables.get(KEY_CLASSPATH).cloned().unwrap_or_default();

        vec![
            "-Xms1024M".into(),
            "-Xmx2048M".into(),
            format!("-Djava.library.path={}", natives_dir),
            format!("-Dminecraft.launcher.brand={}", launcher_name),
            format!("-Dminecraft.launcher.version={}", launcher_version),
            CP_FLAG.into(),
            classpath,
        ]
}


/// Replaces variables in a vector of arguments efficiently
fn replace_variables_in_vec(args: &[String], variables: &HashMap<String, String>) -> Vec<String> {
    args.iter()
        .map(|arg| replace_variables_cow(arg, variables).into_owned())
        .collect()
}

/// Cow-based variable replacement. Only allocates when replacements are needed.
fn replace_variables_cow<'a>(
    input: &'a str,
    variables: &HashMap<String, String>
) -> Cow<'a, str> {
    if !input.contains("${") {
        return Cow::Borrowed(input);
    }

    let mut result = String::with_capacity(input.len() + 128);
    let mut last_end = 0;

    for (start, _) in input.match_indices("${") {
        if let Some(end_offset) = input[start..].find('}') {
            let end = start + end_offset;
            let key = &input[start + 2..end];

            result.push_str(&input[last_end..start]);

            if let Some(value) = variables.get(key) {
                result.push_str(value);
            } else {
                result.push_str(&input[start..=end]);
            }

            last_end = end + 1;
        }
    }

    result.push_str(&input[last_end..]);
    Cow::Owned(result)
}

/// Applies JVM overrides, prepending the `-` prefix automatically.
///
/// Format examples:
/// - `Xmx` → `-Xmx`
/// - `XX:+UseG1GC` → `-XX:+UseG1GC`
/// - `Djava.library.path` → `-Djava.library.path`
fn apply_jvm_overrides(jvm_args: &mut Vec<String>, jvm_overrides: &HashMap<String, String>) {
    for (key, value) in jvm_overrides {
        let formatted_option = format_jvm_option(key, value);

        let key_prefix = format!("-{}", key.split('=').next().unwrap_or(key));
        if let Some(pos) = jvm_args.iter().position(|arg| arg.starts_with(&key_prefix)) {
            jvm_args[pos] = formatted_option;
        } else {
            // Insert before -cp so the classpath stays last.
            if let Some(cp_pos) = jvm_args.iter().position(|arg| arg == CP_FLAG) {
                jvm_args.insert(cp_pos, formatted_option);
            } else {
                jvm_args.push(formatted_option);
            }
        }
    }
}

/// Formats a JVM option using the appropriate separator for its kind.
///
/// # Examples
/// - `("Xmx", "4G")` → `-Xmx4G`
/// - `("Xms", "2G")` → `-Xms2G`
/// - `("XX:+UseG1GC", "")` → `-XX:+UseG1GC`
/// - `("Djava.library.path", "/path")` → `-Djava.library.path=/path`
fn format_jvm_option(key: &str, value: &str) -> String {
    if value.is_empty() {
        format!("-{}", key)
    } else if key.starts_with('X') && !key.contains(':') && !key.contains('=') {
        // -Xmx, -Xms, etc. — no separator between key and value.
        format!("-{}{}", key, value)
    } else {
        // -D, -XX:, etc. — `=` separator.
        format!("-{}={}", key, value)
    }
}

/// Removes JVM options whose key appears in `jvm_removals`.
fn apply_jvm_removals(jvm_args: &mut Vec<String>, jvm_removals: &HashSet<String>) {
    jvm_args.retain(|arg| {
        let arg_key = if let Some(stripped) = arg.strip_prefix('-') {
            stripped.split('=').next().unwrap_or(stripped)
                .split(|c: char| c.is_numeric()).next().unwrap_or(stripped)
        } else {
            return true;
        };

        !jvm_removals.contains(arg_key)
    });
}

/// Filters out game arguments whose key matches an entry in `arg_removals`.
fn apply_arg_removals(game_args: Vec<String>, arg_removals: &HashSet<String>) -> Vec<String> {
    game_args.into_iter()
        .filter(|arg| {
            !arg_removals.iter().any(|removal| {
                arg == removal || arg.starts_with(&format!("--{}", removal))
            })
        })
        .collect()
}

/// Filters the classpath, excluding JARs whose artifact already appears
/// on the module-path (matched by base name, version-agnostic).
fn filter_classpath_from_modulepath(classpath: &str, module_path: &str) -> String {
    let separator = get_path_separator();

    // Extract base artifact name from a jar filename: "asm-analysis-9.5.jar" -> "asm-analysis"
    let module_artifacts: std::collections::HashSet<String> = module_path
        .split(separator)
        .filter_map(|path| {
            std::path::Path::new(path)
                .file_name()
                .and_then(|f| f.to_str())
                .and_then(|filename| {
                    if let Some(stem) = filename.strip_suffix(".jar") {
                        let mut base_name = stem;
                        if let Some(pos) = stem.rfind(|c: char| c.is_ascii_digit()) {
                            if let Some(dash_pos) = stem[..pos].rfind('-') {
                                base_name = &stem[..dash_pos];
                            }
                        }
                        Some(base_name.to_string())
                    } else {
                        None
                    }
                })
        })
        .collect();

    lighty_core::trace_debug!("Module artifacts to exclude: {:?}", module_artifacts);

    classpath
        .split(separator)
        .filter(|path| {
            if let Some(filename) = std::path::Path::new(path)
                .file_name()
                .and_then(|f| f.to_str())
            {
                if let Some(stem) = filename.strip_suffix(".jar") {
                    let base_name = if let Some(pos) = stem.rfind(|c: char| c.is_ascii_digit()) {
                        if let Some(dash_pos) = stem[..pos].rfind('-') {
                            &stem[..dash_pos]
                        } else {
                            stem
                        }
                    } else {
                        stem
                    };
                    !module_artifacts.contains(base_name)
                } else {
                    true
                }
            } else {
                true
            }
        })
        .collect::<Vec<&str>>()
        .join(separator)
}

fn get_path_separator() -> &'static str {
    #[cfg(target_os = "windows")] {
        ";"
    }
    #[cfg(not(target_os = "windows"))] {
        ":"
    }
}

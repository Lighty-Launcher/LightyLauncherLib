# Sequence Diagrams

## Complete Launch Sequence

```mermaid
sequenceDiagram
    participant User
    participant AppState
    participant VersionBuilder
    participant Auth
    participant LaunchBuilder
    participant Installer
    participant JavaManager
    participant Process

    User->>AppState: init(name)
    AppState-->>User: paths stored globally

    User->>VersionBuilder: new(name, loader, loader_version, mc_version)
    VersionBuilder-->>User: instance

    User->>Auth: OfflineAuth::new(username)
    User->>Auth: authenticate()
    Auth-->>User: UserProfile

    User->>LaunchBuilder: instance.launch(profile, java_distribution)

    LaunchBuilder->>LaunchBuilder: Prepare Metadata
    LaunchBuilder->>JavaManager: Ensure Java Installed

    alt Java Not Found
        JavaManager->>JavaManager: Download Java Runtime
        JavaManager->>JavaManager: Extract Archive
        JavaManager-->>LaunchBuilder: java_path
    else Java Found
        JavaManager-->>LaunchBuilder: java_path
    end

    LaunchBuilder->>Installer: install(version_data)

    par Parallel Installation (8 buckets via tokio::try_join!)
        Installer->>Installer: Download Libraries
        Installer->>Installer: Download Natives
        Installer->>Installer: Download Client JAR
        Installer->>Installer: Download Assets
        Installer->>Installer: Download Mods (subdir: mods/)
        Installer->>Installer: Download ResourcePacks (subdir: resourcepacks/)
        Installer->>Installer: Download ShaderPacks (subdir: shaderpacks/)
        Installer->>Installer: Download Datapacks (subdir: datapacks/)
    end

    Installer->>Installer: Extract Natives
    Installer-->>LaunchBuilder: Installation Complete

    LaunchBuilder->>LaunchBuilder: Build Arguments
    LaunchBuilder->>Process: Spawn Java Process
    Process-->>LaunchBuilder: PID

    LaunchBuilder->>Process: Register Instance
    LaunchBuilder->>Process: Stream Console (stdout/stderr)

    Process-->>User: Instance Launched

    Note over Process: Game Running...

    Process->>Process: Emit ConsoleOutput Events

    alt Normal Exit
        Process->>Process: Game Exits
        Process->>Process: Emit InstanceExited Event
        Process->>Process: Unregister Instance
    else Manual Close
        User->>Process: close_instance(pid)
        Process->>Process: Kill Signal (SIGTERM/TASKKILL)
        Process->>Process: Emit InstanceExited Event
        Process->>Process: Unregister Instance
    end
```

## Authentication Sequence

### Offline Authentication

```mermaid
sequenceDiagram
    participant User
    participant OfflineAuth
    participant UUID

    User->>OfflineAuth: new(username)
    User->>OfflineAuth: authenticate()
    OfflineAuth->>UUID: generate_offline_uuid(username)
    UUID-->>OfflineAuth: deterministic UUID (v5)
    OfflineAuth-->>User: UserProfile::offline(username, uuid)
    Note over OfflineAuth: provider = AuthProvider::Offline<br/>access_token = None<br/>all other optional fields default to None / false
```

### Microsoft Authentication

```mermaid
sequenceDiagram
    participant User
    participant MicrosoftAuth
    participant DeviceFlow
    participant Xbox
    participant Minecraft

    User->>MicrosoftAuth: new(client_id)
    User->>MicrosoftAuth: authenticate(event_bus)

    MicrosoftAuth->>DeviceFlow: Request Device Code
    DeviceFlow-->>MicrosoftAuth: device_code, user_code, verification_url
    MicrosoftAuth->>User: Display: "Visit {url}, Enter {code}"

    loop Poll for completion
        MicrosoftAuth->>DeviceFlow: Poll for token
        alt User Authorized
            DeviceFlow-->>MicrosoftAuth: access_token + refresh_token
        else Still Pending
            DeviceFlow-->>MicrosoftAuth: authorization_pending
        end
    end

    MicrosoftAuth->>Xbox: Authenticate with Xbox Live
    Xbox-->>MicrosoftAuth: xbox_token, user_hash (UHS)

    MicrosoftAuth->>Xbox: Get XSTS Token
    Xbox-->>MicrosoftAuth: xsts_token

    MicrosoftAuth->>Minecraft: Authenticate with Minecraft (XBL3.0 x=UHS;xsts_token)
    Minecraft-->>MicrosoftAuth: minecraft_access_token

    Note over MicrosoftAuth: Decode xuid from MC token JWT payload
    Note over MicrosoftAuth: (authlib expects it to match --xuid)

    MicrosoftAuth->>Minecraft: Get Profile (Bearer mc_token)
    Minecraft-->>MicrosoftAuth: username, uuid

    MicrosoftAuth-->>User: UserProfile {
        username, uuid,
        access_token: Some(SecretString::from(mc_token)),
        token_handle: None (or Some(handle) if with_keyring),
        xuid: Some(xuid),
        provider: AuthProvider::Microsoft {
            client_id,
            refresh_token: Some(SecretString::from(rt))
        }
    }
    Note over MicrosoftAuth: access_token + refresh_token wrapped in SecretString.<br/>If with_keyring(service) was set, both are written to the OS keychain<br/>and access_token is None — only a TokenHandle is returned.
```

### Silent Re-authentication

After the first device-code flow, subsequent launches can skip user
interaction entirely by reusing the persisted refresh token:

```mermaid
sequenceDiagram
    participant User
    participant Storage as Keyring / Disk
    participant MicrosoftAuth
    participant MS as Microsoft OAuth
    participant Xbox
    participant Minecraft

    User->>Storage: load_profile()
    Storage-->>User: UserProfile (provider has refresh_token)

    User->>MicrosoftAuth: authenticate_with_refresh_token(rt)
    MicrosoftAuth->>MS: POST /token (grant_type=refresh_token)
    alt Refresh token still valid
        MS-->>MicrosoftAuth: new access_token + rotated refresh_token
        MicrosoftAuth->>Xbox: Authenticate (silent)
        Xbox-->>MicrosoftAuth: xbox_token, UHS
        MicrosoftAuth->>Xbox: XSTS
        Xbox-->>MicrosoftAuth: xsts_token
        MicrosoftAuth->>Minecraft: login_with_xbox
        Minecraft-->>MicrosoftAuth: new mc_token
        MicrosoftAuth->>Minecraft: get profile
        Minecraft-->>MicrosoftAuth: username, uuid
        MicrosoftAuth-->>User: UserProfile (refresh_token rotated)
        User->>Storage: save_profile() (persist the new rt)
    else Refresh token expired (~90d) or revoked
        MS-->>MicrosoftAuth: 4xx
        MicrosoftAuth-->>User: AuthError::InvalidToken
        Note over User: Fall back to authenticate() (device-code)
    end
```

## Installation Sequence

```mermaid
sequenceDiagram
    participant Installer
    participant Libraries
    participant Natives
    participant Client
    participant Assets
    participant Mods
    participant EventBus

    Installer->>Installer: Phase 1: Verification (SHA1 Check)

    par Collect Tasks (tokio::join! — 8 buckets)
        Installer->>Libraries: Check libraries
        Libraries-->>Installer: library_tasks[]

        Installer->>Natives: Check natives
        Natives-->>Installer: (native_download_tasks[], native_extract_paths[])

        Installer->>Client: Check client JAR
        Client-->>Installer: client_task?

        Installer->>Assets: Check assets
        Assets-->>Installer: asset_tasks[]

        Installer->>Mods: Check mods (subdir: mods/)
        Mods-->>Installer: (mod_tasks[], mod_bytes)

        Installer->>Mods: Check resourcepacks (subdir: resourcepacks/)
        Mods-->>Installer: (resourcepack_tasks[], resourcepack_bytes)

        Installer->>Mods: Check shaderpacks (subdir: shaderpacks/)
        Mods-->>Installer: (shaderpack_tasks[], shaderpack_bytes)

        Installer->>Mods: Check datapacks (subdir: datapacks/)
        Mods-->>Installer: (datapack_tasks[], datapack_bytes)
    end

    Note over Installer: mod_like_bytes = mod_bytes + resourcepack_bytes + shaderpack_bytes + datapack_bytes<br/>passed to calculate_download_size()

    alt All Files Valid (total_downloads == 0)
        Installer->>EventBus: Emit LaunchEvent::IsInstalled
        Installer->>Natives: Extract natives only
        Natives-->>Installer: Done
    else Files Need Download
        Installer->>EventBus: Emit LaunchEvent::InstallStarted { total_bytes }

        par Phase 2: Parallel Download (8 buckets)
            Installer->>Libraries: Download library_tasks
            Libraries->>EventBus: Emit LaunchEvent::InstallProgress { bytes } (per chunk)

            Installer->>Natives: Download & extract native_tasks
            Natives->>EventBus: Emit LaunchEvent::InstallProgress { bytes }

            Installer->>Client: Download client_task
            Client->>EventBus: Emit LaunchEvent::InstallProgress { bytes }

            Installer->>Assets: Download asset_tasks
            Assets->>EventBus: Emit LaunchEvent::InstallProgress { bytes }

            Installer->>Mods: Download mod_tasks (subdir: mods/)
            Mods->>EventBus: Emit LaunchEvent::InstallProgress { bytes }

            Installer->>Mods: Download resourcepack_tasks (subdir: resourcepacks/)
            Mods->>EventBus: Emit ModloaderEvent::ResourcePacksInstalled { count, bytes }

            Installer->>Mods: Download shaderpack_tasks (subdir: shaderpacks/)
            Mods->>EventBus: Emit ModloaderEvent::ShaderPacksInstalled { count, bytes }

            Installer->>Mods: Download datapack_tasks (subdir: datapacks/)
            Mods->>EventBus: Emit ModloaderEvent::DatapacksInstalled { count, bytes }
        end

        Installer->>EventBus: Emit LaunchEvent::InstallCompleted
    end
```

## Java Management Sequence

```mermaid
sequenceDiagram
    participant LaunchBuilder
    participant JavaManager
    participant Downloader
    participant Extractor
    participant EventBus

    LaunchBuilder->>JavaManager: ensure_java_installed(java_dirs, distribution, version)

    JavaManager->>JavaManager: find_java_binary(java_dirs, distribution, version)

    alt Java Found
        JavaManager->>EventBus: Emit JavaAlreadyInstalled
        JavaManager-->>LaunchBuilder: java_path
    else Java Not Found
        JavaManager->>EventBus: Emit JavaNotFound

        JavaManager->>Downloader: Download JRE archive
        Downloader->>EventBus: Emit JavaDownloadStarted

        loop Download Progress
            Downloader->>EventBus: Emit JavaDownloadProgress
        end

        Downloader->>EventBus: Emit JavaDownloadCompleted
        Downloader-->>JavaManager: archive_path

        JavaManager->>Extractor: Extract archive
        Extractor->>EventBus: Emit JavaExtractionStarted

        loop Extraction Progress
            Extractor->>EventBus: Emit ExtractionProgress
        end

        Extractor->>EventBus: Emit JavaExtractionCompleted
        Extractor-->>JavaManager: extracted_path

        JavaManager->>JavaManager: find_java_binary(extracted_path)
        JavaManager-->>LaunchBuilder: java_path
    end
```

## Instance Control Sequence

```mermaid
sequenceDiagram
    participant User
    participant InstanceControl
    participant InstanceManager
    participant Process
    participant EventBus

    Note over User,EventBus: Get Running Instance

    User->>InstanceControl: get_pid()
    InstanceControl->>InstanceManager: get_pid(instance_name)
    InstanceManager-->>InstanceControl: pid?
    InstanceControl-->>User: Option<u32>

    Note over User,EventBus: Close Instance

    User->>InstanceControl: close_instance(pid)
    InstanceControl->>InstanceManager: close_instance(pid)

    alt Windows
        InstanceManager->>Process: taskkill /PID {pid} /F
    else Unix (Linux/macOS)
        InstanceManager->>Process: kill -SIGTERM {pid}
    end

    Process->>Process: Terminate
    Process->>EventBus: Emit InstanceExited
    Process->>InstanceManager: Unregister instance
    InstanceManager-->>User: Result<()>

    Note over User,EventBus: Delete Instance

    User->>InstanceControl: delete_instance()
    InstanceControl->>InstanceManager: has_running_instances()

    alt Instance Running
        InstanceManager-->>User: Error: InstanceRunning
    else Not Running
        InstanceControl->>InstanceControl: Delete game directory
        InstanceControl->>EventBus: Emit InstanceDeleted
        InstanceControl-->>User: Result<()>
    end
```

## Console Streaming Sequence

```mermaid
sequenceDiagram
    participant Process
    participant StdoutHandler
    participant StderrHandler
    participant EventBus
    participant User

    Process->>Process: Spawn Java Process
    Process->>StdoutHandler: Spawn stdout task
    Process->>StderrHandler: Spawn stderr task

    par Console Streaming
        loop Read stdout
            StdoutHandler->>StdoutHandler: Read line
            StdoutHandler->>EventBus: Emit ConsoleOutput(Stdout, line)
            EventBus->>User: Console line
        end

        loop Read stderr
            StderrHandler->>StderrHandler: Read line
            StderrHandler->>EventBus: Emit ConsoleOutput(Stderr, line)
            EventBus->>User: Console line
        end
    end

    Process->>Process: Wait for exit
    Process->>EventBus: Emit InstanceExited(exit_code)
    Process->>Process: Unregister instance
```

## Event Flow Diagram

```mermaid
flowchart TB
    Start([User Initiates Launch]) --> InitState[Initialize AppState]
    InitState --> CreateVersion[Create VersionBuilder]
    CreateVersion --> Auth[Authenticate User]

    Auth --> |OfflineAuth| OfflineEvent[Emit AuthenticationStarted/Success]
    Auth --> |MicrosoftAuth| MSEvents[Emit Device Code Events]

    OfflineEvent --> StartLaunch[Call launch()]
    MSEvents --> StartLaunch

    StartLaunch --> FetchMeta[Fetch Loader Metadata]
    FetchMeta --> |Emit LoaderEvent| CheckJava[Check Java Installation]

    CheckJava --> |Not Found| DownloadJava[Download Java]
    CheckJava --> |Found| InstallDeps[Install Dependencies]

    DownloadJava --> |Emit JavaEvents| InstallDeps

    InstallDeps --> Verify{All Files Valid?}

    Verify --> |Yes| EmitInstalled[Emit IsInstalled]
    Verify --> |No| EmitStart[Emit InstallStarted]

    EmitInstalled --> ExtractNatives[Extract Natives Only]
    EmitStart --> ParallelDownload[Parallel Download]

    ParallelDownload --> |Libraries| LibEvents[Emit InstallProgress per chunk]
    ParallelDownload --> |Natives| NatEvents[Emit InstallProgress per chunk]
    ParallelDownload --> |Client| ClientEvents[Emit InstallProgress per chunk]
    ParallelDownload --> |Assets| AssetEvents[Emit InstallProgress per chunk]
    ParallelDownload --> |Mods| ModEvents[Emit InstallProgress per chunk]
    ParallelDownload --> |ResourcePacks| RPEvents[Emit ModloaderEvent::ResourcePacksInstalled]
    ParallelDownload --> |ShaderPacks| SPEvents[Emit ModloaderEvent::ShaderPacksInstalled]
    ParallelDownload --> |Datapacks| DPEvents[Emit ModloaderEvent::DatapacksInstalled]

    LibEvents --> EmitComplete[Emit InstallCompleted]
    NatEvents --> EmitComplete
    ClientEvents --> EmitComplete
    AssetEvents --> EmitComplete
    ModEvents --> EmitComplete
    RPEvents --> EmitComplete
    SPEvents --> EmitComplete
    DPEvents --> EmitComplete
    ExtractNatives --> EmitComplete

    EmitComplete --> BuildArgs[Build Arguments]
    BuildArgs --> SpawnProcess[Spawn Java Process]
    SpawnProcess --> Register[Register Instance]
    Register --> EmitLaunched[Emit InstanceLaunched]

    EmitLaunched --> StreamConsole[Stream Console Output]
    StreamConsole --> |Each Line| EmitConsole[Emit ConsoleOutput]

    EmitConsole --> WaitExit{Process Running?}
    WaitExit --> |Yes| StreamConsole
    WaitExit --> |No| EmitExited[Emit InstanceExited]

    EmitExited --> Cleanup[Unregister Instance]
    Cleanup --> End([Launch Complete])
```

## Loader-Specific Sequences

### Fabric Loader

```mermaid
sequenceDiagram
    participant User
    participant FabricLoader
    participant VanillaAPI
    participant FabricAPI
    participant Merger

    User->>FabricLoader: get_metadata()

    FabricLoader->>VanillaAPI: Fetch Vanilla manifest
    VanillaAPI-->>FabricLoader: vanilla_metadata

    FabricLoader->>FabricAPI: Fetch Fabric loader data
    FabricAPI-->>FabricLoader: fabric_loader_data

    FabricLoader->>Merger: Merge vanilla + fabric
    Merger->>Merger: Add Fabric libraries
    Merger->>Merger: Update main class
    Merger->>Merger: Merge arguments
    Merger-->>FabricLoader: merged_metadata

    FabricLoader-->>User: VersionMetaData
```

### LightyUpdater

```mermaid
sequenceDiagram
    participant User
    participant LightyBuilder
    participant ServerAPI
    participant Vanilla

    User->>LightyBuilder: new(name, server_url)
    User->>LightyBuilder: get_metadata()

    LightyBuilder->>ServerAPI: GET {server_url}/version
    ServerAPI-->>LightyBuilder: {
        minecraft_version,
        loader,             // "vanilla" | "fabric" | "quilt" | "neoforge" | "forge"
        loader_version,
        mods: [...]
    }

    LightyBuilder->>Vanilla: Fetch base-loader metadata
    Note over LightyBuilder,Vanilla: loader = "forge" now supported in<br/>merge_metadata.rs (mapped to Loader::Forge).<br/>The lighty_updater feature already activates the<br/>forge feature at the workspace level.
    Vanilla-->>LightyBuilder: base_metadata

    LightyBuilder->>LightyBuilder: Add server mods to metadata
    LightyBuilder-->>User: VersionMetaData with custom mods
```

## Related Documentation

- [Launch Process](../crates/launch/docs/launch.md) - Detailed launch flow
- [Installation](../crates/launch/docs/installation.md) - Installation details
- [Instance Control](../crates/launch/docs/instance-control.md) - Process management
- [Events](../crates/launch/docs/events.md) - Event types reference

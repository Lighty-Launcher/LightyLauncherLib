# Contributing

Thanks for your interest! Here's the fast path from a fresh clone to a
merged PR.

## Reporting issues

Open one [here](https://github.com/Lighty-Launcher/LightyLauncherLib/issues)
and tell us:

- The command you ran (`cargo run --example ... --features ...`)
- The full error / panic backtrace
- Your OS + `rustc --version`
- What you expected to happen

For security findings, open a private security advisory instead of a
public issue.

## How the repo is laid out

```
.
├── src/lib.rs               # Root crate (re-exports + feature gates)
├── crates/
│   ├── core/                # AppState, HTTP, hashing, extract
│   ├── auth/                # Offline / Microsoft / Azuriom
│   ├── event/               # Broadcast bus + typed events
│   ├── java/                # JRE auto-download
│   ├── launch/              # Install + game lifecycle
│   ├── loaders/             # Vanilla / Fabric / Quilt / Forge / NeoForge / OptiFine / LightyUpdater
│   ├── modsloader/          # Modrinth + CurseForge + modpack pipelines
│   └── version/             # VersionBuilder
├── examples/                # Runnable samples
├── docs/                    # Cross-crate docs + media assets
├── Cargo.toml               # Workspace + root-crate manifest
└── README.md
```

Versions are declared once in the root `Cargo.toml` and inherited by
all crates — **don't bump individual crates**, always the workspace.

## Branches

- **`development`** — where PRs land.
- **`production`** — release branch, only updated at release time.

Branch off `development`, never push directly to `production`.

## Local setup

```bash
rustup toolchain install stable      # MSRV is 1.95, edition 2024
git clone https://github.com/Lighty-Launcher/LightyLauncherLib
cd LightyLauncherLib
git checkout development
cargo check --workspace --all-features
```

## Heads-up: `.gitignore` is whitelist-style

The repo ignores everything by default and only allows specific paths:

```
/*

!src/
!.github/**
!crates/
!examples/
!docs/

!LICENCE
!README.md
!Cargo.toml
!Cargo.lock
!.gitignore
```

If you add a new top-level file or folder, **whitelist it in
`.gitignore`** or git will silently skip it.

## Sending a PR

1. Branch off `development`:
   ```bash
   git checkout development && git pull
   git checkout -b feat/<short-name>     # or fix/, refactor/, docs/
   ```
2. Implement, then run the local checks listed above.
3. Push and open a PR targeting `development`.
4. Wait for review — you may be asked to rebase before merge.

Keep PRs focused. Multi-purpose work is OK if the commit message makes
the scope obvious.

## Code style

- **No fully-qualified paths inline** — add a `use` at the top of the file.
- **Comments**: above the code, short, explain *why* not *what*. Inline
  trailing notes are reserved for trivial one-liners

## Adding a dependency

New crate in `Cargo.toml`? **Justify it** in the commit message (or PR
description): what it brings, why an existing dep or a few lines of
hand-written code wouldn't do. Lighter is always preferred — every
extra dep is build time, supply-chain surface, and one more thing to
keep up to date.

## Versioning (CalVer)

Versions look like `YY.MM.PATCH` (e.g. `26.5.12` = May 2026, patch 11):

| Part | Bump when |
|---|---|
| `YY` | New calendar year |
| `MM` | New monthly release (may include breaking changes) |
| `PATCH` | Bug fix / non-breaking addition inside the same month |

API stability is **per-month**, not per-major-number. Consumers pin to
a `YY.MM` and migrate on their own pace.

## Commits

**Commit messages must be clear.** Six months from now, the title
alone should tell you what changed and why.

## Licence

MIT, declared in `Cargo.toml`. By contributing, you agree your work is
published under the same licence.

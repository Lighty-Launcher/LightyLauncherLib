use crate::errors::{ExtractError, ExtractResult};
use async_compression::tokio::bufread::GzipDecoder;
use async_zip::tokio::read::seek::ZipFileReader;
use futures_util::io::{self, BufReader as FuturesBufReader};
use futures_util::StreamExt;
use std::path::{Path, PathBuf};
use tokio::fs::{create_dir_all, OpenOptions};
use tokio::io::AsyncBufRead;
use tokio::io::{AsyncRead, AsyncSeek, BufReader};
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

#[cfg(feature = "events")]
use lighty_event::{EventBus, Event, CoreEvent};

// 2GB cap protects against zip bombs.
const MAX_FILE_SIZE: u64 = 2 * 1024 * 1024 * 1024;
const BUFFER_SIZE: usize = 256 * 1024;

/// Extracts a ZIP archive to `out_dir` with path-traversal, symlink and
/// size-bomb protection.
pub async fn zip_extract<R>(
    archive: R,
    out_dir: &Path,
    #[cfg(feature = "events")] event_bus: Option<&EventBus>,
) -> ExtractResult<()>
where
    R: AsyncRead + AsyncSeek + Unpin + AsyncBufRead,
{
    let out_dir = out_dir.canonicalize()?;
    let mut reader = ZipFileReader::new(archive.compat()).await?;

    let entries_count = reader.file().entries().len();

    #[cfg(feature = "events")]
    if let Some(bus) = event_bus {
        bus.emit(Event::Core(CoreEvent::ExtractionStarted {
            archive_type: "ZIP".to_string(),
            file_count: entries_count,
            destination: out_dir.to_string_lossy().to_string(),
        }));
    }

    for index in 0..entries_count {
        // Collect entry metadata before mutably borrowing reader.
        let (_file_name, path, is_dir, uncompressed_size) = {
            let entry = reader.file().entries().get(index)
                .ok_or_else(|| ExtractError::ZipEntryNotFound { index })?;

            let file_name = entry.filename().as_str()?;
            let is_dir = entry.dir()?;
            let uncompressed_size = entry.uncompressed_size();

            let sanitized = sanitize_file_path(file_name);

            if sanitized.is_absolute() {
                return Err(ExtractError::AbsolutePath {
                    path: file_name.to_string()
                });
            }

            let path = out_dir.join(&sanitized);

            if !is_path_within_base(&path, &out_dir)? {
                return Err(ExtractError::PathTraversal {
                    path: file_name.to_string()
                });
            }

            (file_name.to_string(), path, is_dir, uncompressed_size)
        };

        if is_dir {
            create_dir_all(&path).await?;
        } else {
            if uncompressed_size > MAX_FILE_SIZE {
                return Err(ExtractError::FileTooLarge {
                    size: uncompressed_size,
                    max: MAX_FILE_SIZE,
                });
            }

            if let Some(parent) = path.parent() {
                create_dir_all(parent).await?;
            }

            let entry_reader = reader.reader_with_entry(index).await?;
            let buffered_reader = FuturesBufReader::with_capacity(BUFFER_SIZE, entry_reader);

            let file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&path)
                .await?;

            io::copy(buffered_reader, &mut file.compat_write()).await?;
        }

        #[cfg(feature = "events")]
        if let Some(bus) = event_bus {
            if (index + 1) % 10 == 0 || (index + 1) == entries_count {
                bus.emit(Event::Core(CoreEvent::ExtractionProgress {
                    files_extracted: index + 1,
                    total_files: entries_count,
                }));
            }
        }
    }

    #[cfg(feature = "events")]
    if let Some(bus) = event_bus {
        bus.emit(Event::Core(CoreEvent::ExtractionCompleted {
            archive_type: "ZIP".to_string(),
            files_extracted: entries_count,
        }));
    }

    Ok(())
}

/// Extracts a tar.gz archive to `out_dir` with path-traversal, symlink and
/// size-bomb protection.
pub async fn tar_gz_extract<R>(
    archive: R,
    out_dir: &Path,
    #[cfg(feature = "events")] event_bus: Option<&EventBus>,
) -> ExtractResult<()>
where
    R: AsyncRead + Unpin,
{
    let out_dir = out_dir.canonicalize()?;
    let decoder = GzipDecoder::new(BufReader::new(archive));
    let mut ar = tokio_tar::Archive::new(decoder);

    #[cfg(feature = "events")]
    if let Some(bus) = event_bus {
        bus.emit(Event::Core(CoreEvent::ExtractionStarted {
            archive_type: "TAR.GZ".to_string(),
            file_count: 0,
            destination: out_dir.to_string_lossy().to_string(),
        }));
    }

    let mut entries = ar.entries()?;
    #[cfg(feature = "events")]
    let mut files_extracted = 0usize;

    while let Some(entry) = entries.next().await {
        let mut entry = entry?;
        let path = entry.path()?.to_path_buf();

        if path.is_absolute() {
            continue;
        }

        let dest = out_dir.join(&path);

        if !is_path_within_base(&dest, &out_dir)? {
            continue;
        }

        let entry_type = entry.header().entry_type();
        if entry_type.is_symlink() || entry_type.is_hard_link() {
            continue;
        }

        let size = entry.header().size()?;
        if size > MAX_FILE_SIZE {
            return Err(ExtractError::FileTooLarge {
                size,
                max: MAX_FILE_SIZE,
            });
        }

        entry.unpack(&dest).await?;

        #[cfg(feature = "events")]
        {
            files_extracted += 1;
        }

        #[cfg(feature = "events")]
        if let Some(bus) = event_bus {
            if files_extracted % 10 == 0 {
                bus.emit(Event::Core(CoreEvent::ExtractionProgress {
                    files_extracted,
                    total_files: 0,
                }));
            }
        }
    }

    #[cfg(feature = "events")]
    if let Some(bus) = event_bus {
        bus.emit(Event::Core(CoreEvent::ExtractionCompleted {
            archive_type: "TAR.GZ".to_string(),
            files_extracted,
        }));
    }

    Ok(())
}

/// Returns a relative path without reserved names, redundant separators, ".", or "..".
fn sanitize_file_path(path: &str) -> PathBuf {
    path.replace('\\', "/")
        .split('/')
        .map(sanitize_filename::sanitize)
        .collect()
}

/// Validates that `path` resolves inside `base` using component folding.
///
/// Used during extraction (before the file exists on disk), so we cannot
/// rely on [`std::fs::canonicalize`].
fn is_path_within_base(path: &Path, base: &Path) -> ExtractResult<bool> {
    let normalized_path: PathBuf = path.components()
        .fold(PathBuf::new(), |mut acc, component| {
            match component {
                std::path::Component::Normal(c) => acc.push(c),
                std::path::Component::ParentDir => { acc.pop(); },
                std::path::Component::CurDir => {},
                _ => acc.push(component),
            }
            acc
        });

    let normalized_base: PathBuf = base.components()
        .fold(PathBuf::new(), |mut acc, component| {
            match component {
                std::path::Component::Normal(c) => acc.push(c),
                std::path::Component::ParentDir => { acc.pop(); },
                std::path::Component::CurDir => {},
                _ => acc.push(component),
            }
            acc
        });

    Ok(normalized_path.starts_with(&normalized_base))
}
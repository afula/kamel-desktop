use anyhow::Context;
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;
use uuid::Uuid;
pub async fn write_file_async(
    path: impl AsRef<std::path::Path>,
    content: &[u8],
) -> Result<(), anyhow::Error> {
    use tokio::io::AsyncWriteExt;

    log::trace!("Writing file (async) {}", path.as_ref().display());
    let mut file = tokio::fs::File::create(&path).await?;
    file.write_all(content).await?;
    file.flush().await?;

    Ok(())
}

pub async fn read_file_async(path: impl AsRef<std::path::Path>) -> Result<Vec<u8>, anyhow::Error> {
    use tokio::io::AsyncReadExt;

    log::trace!("Opening file (async) {}", path.as_ref().display());
    let mut file = tokio::fs::File::open(&path).await?;
    let mut content = vec![];
    file.read_to_end(&mut content).await?;
    log::trace!(
        "Read file {} with {} bytes",
        path.as_ref().display(),
        content.len()
    );

    Ok(content)
}

/// Saves a given attachment into a random-generated path. Returns the path.
pub async fn save_attachment(
    dest: &Path,
    ext: &str,
    attachment: &[u8],
) -> Result<PathBuf, anyhow::Error> {
    let fname = Uuid::new_v4().to_simple();
    let fname_formatted = format!("{}", fname);
    let fname_path = Path::new(&fname_formatted);

    let mut path = dest.join(fname_path);
    path.set_extension(ext);

    write_file_async(&path, attachment).await.with_context(|| {
        format!(
            "Could not create and write to attachment file: {}",
            path.display()
        )
    })?;

    Ok(path)
}

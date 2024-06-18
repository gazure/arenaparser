use std::path::PathBuf;
use tracing::info;
use crate::replay::MatchReplay;

pub trait ArenaMatchStorageBackend {

    /// # Errors
    ///
    /// Will return an error if the match replay cannot be written to the storage backend
    fn write(&mut self, match_replay: &MatchReplay) -> anyhow::Result<()>;
}

pub struct DirectoryStorageBackend{
    path: PathBuf,
}

impl DirectoryStorageBackend {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl ArenaMatchStorageBackend for DirectoryStorageBackend {
    fn write(&mut self, match_replay: &MatchReplay) -> anyhow::Result<()> {
        let path = self.path.join(format!("{}.json", match_replay.match_id));
        info!("Writing match replay to file: {}", path.clone().to_str().unwrap_or("Path not found"));
        match_replay.write(path)?;
        info!("Match replay written to file");
        Ok(())
    }
}

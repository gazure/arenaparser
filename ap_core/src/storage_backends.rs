use std::path::PathBuf;
use tracing::info;
use crate::replay::MatchReplay;

pub trait ArenaMatchStorageBackend {
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
        info!("Writing match replay to file: {}", path.clone().to_str().unwrap());
        match_replay.write(path)?;
        info!("Match replay written to file");
        Ok(())
    }
}

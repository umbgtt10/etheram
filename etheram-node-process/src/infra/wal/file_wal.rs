// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_node::implementations::ibft::consensus_wal::ConsensusWal;
use etheram_node::implementations::ibft::wal_writer::WalWriter;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

pub struct FileWal {
    path: PathBuf,
}

impl FileWal {
    pub fn new(db_path: &str) -> Result<Self, String> {
        fs::create_dir_all(Path::new(db_path))
            .map_err(|error| format!("failed to create wal directory {}: {error}", db_path))?;
        Ok(Self {
            path: Path::new(db_path).join("consensus.wal"),
        })
    }

    pub fn load(&self) -> Result<Option<ConsensusWal>, String> {
        if !self.path.exists() {
            return Ok(None);
        }
        let bytes = fs::read(&self.path)
            .map_err(|error| format!("failed to read wal {}: {error}", self.path.display()))?;
        if bytes.is_empty() {
            return Ok(None);
        }
        ConsensusWal::from_bytes(&bytes).map(Some).ok_or_else(|| {
            format!(
                "failed to decode wal {}: invalid bytes",
                self.path.display()
            )
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn store(&self, wal: &ConsensusWal) -> Result<(), String> {
        let temp_path = self.path.with_extension("wal.tmp");
        let mut temp_file = File::create(&temp_path).map_err(|error| {
            format!(
                "failed to create wal temp file {}: {error}",
                temp_path.display()
            )
        })?;
        temp_file.write_all(&wal.to_bytes()).map_err(|error| {
            format!(
                "failed to write wal temp file {}: {error}",
                temp_path.display()
            )
        })?;
        temp_file.sync_all().map_err(|error| {
            format!(
                "failed to sync wal temp file {}: {error}",
                temp_path.display()
            )
        })?;
        if self.path.exists() {
            fs::remove_file(&self.path).map_err(|error| {
                format!(
                    "failed to replace wal file {}: {error}",
                    self.path.display()
                )
            })?;
        }
        fs::rename(&temp_path, &self.path).map_err(|error| {
            format!(
                "failed to publish wal file {}: {error}",
                self.path.display()
            )
        })
    }
}

impl WalWriter for FileWal {
    fn write(&mut self, wal: &ConsensusWal) {
        self.store(wal).unwrap_or_else(|error| {
            panic!("failed to persist wal {}: {}", self.path.display(), error)
        });
    }
}

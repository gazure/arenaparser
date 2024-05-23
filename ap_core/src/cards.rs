use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt::Display;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use tracing::error;

#[derive(Debug)]
pub struct CardsDatabase {
    pub db: BTreeMap<String, CardDbEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CardDbEntry {
    pub id: String,
    pub name: String,
    pub pretty_name: String,
    pub color_identity: Vec<String>,
}

impl CardsDatabase {
    pub fn new(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let cards_db_file = File::open(path)?;
        let cards_db_reader = BufReader::new(cards_db_file);
        let cards_db: BTreeMap<String, CardDbEntry> = serde_json::from_reader(cards_db_reader)?;

        Ok(Self { db: cards_db })
    }

    pub fn get_pretty_name<T>(&self, grp_id: &T) -> anyhow::Result<String>
    where
        T: Display + ?Sized,
    {
        let grp_id = grp_id.to_string();
        let card = self
            .db
            .get(&grp_id)
            .ok_or_else(|| anyhow::anyhow!("Card not found in database"))?;
        Ok(card.pretty_name.clone())
    }

    pub fn get_pretty_name_defaulted<T>(&self, grp_id: &T) -> String
    where
        T: Display + ?Sized,
    {
        self.get_pretty_name(grp_id)
            .unwrap_or_else(|_| grp_id.to_string())
    }

    pub fn get<T>(&self, grp_id: &T) -> Option<&CardDbEntry>
    where
        T: Display + ?Sized,
    {
        let grp_id = grp_id.to_string();
        self.db.get(&grp_id)
    }
}

impl Default for CardsDatabase {
    fn default() -> Self {
        let default_path = Path::new("data/cards.json");
        Self::new(default_path).unwrap_or_else(|e| {
            error!("Error loading default cards database: {:?}", e);
            Self {
                db: BTreeMap::new(),
            }
        })
    }
}

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
pub struct CardFace {
    pub name: String,
    pub type_line: String,
    pub mana_cost: Option<String>,
    pub image_uri: Option<String>,
    pub colors: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CardDbEntry {
    pub id: i32,
    pub set: String,
    pub name: String,
    pub lang: String,
    pub image_uri: Option<String>,
    pub mana_cost: Option<String>,
    pub cmc: f32,
    pub type_line: String,
    pub layout: String,
    pub colors: Option<Vec<String>>,
    pub color_identity: Vec<String>,
    pub card_faces: Option<Vec<CardFace>>,
}

impl CardsDatabase {
    /// # Errors
    ///
    /// Will return an error if the database file cannot be opened or if the database file is not valid JSON
    pub fn new(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let cards_db_file = File::open(path)?;
        let cards_db_reader = BufReader::new(cards_db_file);
        let cards_db: BTreeMap<String, CardDbEntry> = serde_json::from_reader(cards_db_reader)?;

        Ok(Self { db: cards_db })
    }

    /// # Errors
    ///
    /// Will return an error if the card cannot be found in the database
    pub fn get_pretty_name<T>(&self, grp_id: &T) -> anyhow::Result<String>
    where
        T: Display + ?Sized,
    {
        let grp_id = grp_id.to_string();
        let card = self
            .db
            .get(&grp_id)
            .ok_or_else(|| anyhow::anyhow!("Card not found in database"))?;
        Ok(card.name.clone())
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

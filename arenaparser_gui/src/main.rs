use anyhow::Result;
use ap_core::cards::CardsDatabase;
use ap_core::deck::Deck;
use ap_core::mtga_events::gre::DeckMessage;
use rusqlite::Connection;
use serde_json;

fn main() -> Result<()> {
    let cards_path = "data/cards-full.json";
    let cards_db = CardsDatabase::new(cards_path)?;
    let db_path = "data/matches.db";
    let conn = Connection::open(db_path)?;
    let mut statement =
        conn.prepare("SELECT match_id, game_number, deck_cards, sideboard_cards FROM decks")?;
    let decks = statement.query_map([], |row| {
        let match_id: String = row.get(0)?;
        let game_number: i32 = row.get(1)?;
        let name = format!("{}: Game {}", match_id, game_number);
        let mainboard_str: String = row.get(2)?;
        let mainboard: Vec<i32> = serde_json::from_str(&mainboard_str).unwrap_or_default();
        let sideboard_str: String = row.get(3)?;
        let sideboard: Vec<i32> = serde_json::from_str(&sideboard_str).unwrap_or_default();
        let deck_message = DeckMessage {
            deck_cards: mainboard,
            sideboard_cards: sideboard,
        };
        Ok(Deck::new(name, &deck_message, &cards_db))
    })?;
    for deck in decks {
        println!("{}\n", deck?);
    }
    Ok(())
}

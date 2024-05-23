use crate::cards::CardsDatabase;
use crate::mtga_events::gre::DeckMessage;
use std::collections::BTreeMap;
use std::fmt::Display;

fn format_list_of_cards(cards: &[String]) -> String {
    let mut card_quantities: BTreeMap<String, i32> = BTreeMap::new();
    cards.iter().for_each(|card| {
        *card_quantities.entry(card.clone()).or_insert(0) += 1;
    });
    let mut cards = card_quantities
        .iter()
        .map(|(card, quantity)| format!("{} x{}", card, quantity))
        .collect::<Vec<String>>();
    cards.sort();
    cards.join("\n")
}

#[derive(Debug)]
pub struct Deck {
    name: String,
    mainboard: Vec<String>,
    sideboard: Vec<String>,
}

impl From<DeckMessage> for Deck {
    fn from(deck_message: DeckMessage) -> Self {
        let cards_db = CardsDatabase::default();
        Self::new("Found Deck".to_string(), &deck_message, &cards_db)
    }
}

impl From<&DeckMessage> for Deck {
    fn from(deck_message: &DeckMessage) -> Self {
        deck_message.clone().into()
    }
}

impl Display for Deck {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mainboard_str = format_list_of_cards(&self.mainboard);
        let sideboard_str = format_list_of_cards(&self.sideboard);
        write!(
            f,
            "Deck: {}\n\n{}\n\nSideboard:\n{}",
            &self.name, mainboard_str, sideboard_str
        )
    }
}

impl Deck {
    pub fn new(name: String, deck_message: &DeckMessage, cards_db: &CardsDatabase) -> Self {
        let mainboard = deck_message
            .deck_cards
            .iter()
            .map(|card_id| cards_db.get_pretty_name_defaulted(card_id))
            .collect();
        let sideboard = deck_message
            .sideboard_cards
            .iter()
            .map(|card_id| cards_db.get_pretty_name_defaulted(card_id))
            .collect();
        Self {
            name,
            mainboard,
            sideboard,
        }
    }
}

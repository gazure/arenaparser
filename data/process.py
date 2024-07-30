import json
import pprint


def main():
    with open('cards-full.json') as fh:
        cards_full = json.load(fh)
    cards = {}
    for card_id, card in cards_full['cards'].items():
        cards[card_id] = {
            'name': card.get('name'),
            'pretty_name': card.get('pretty_name'),
            'id': card_id
        }

    print(json.dumps(cards))


if __name__ == "__main__":
    main()

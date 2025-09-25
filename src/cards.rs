use rand::rng;
use rand::seq::SliceRandom;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Suit {
    Hearts,
    Spades,
    Clubs,
    Diamonds,
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub enum Rank {
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
    Seven = 7,
    Eight = 8,
    Nine = 9,
    Ten = 10,
    Jack = 11,
    Queen = 12,
    King = 13,
    Ace = 14,
}

#[derive(Debug, Copy, Clone)]
pub struct Card {
    pub suit: Suit,
    pub rank: Rank,
}

impl Card {
    pub fn value(&self) -> u8 {
        self.rank as u8
    }

    pub fn suit_symbol(&self) -> &'static str {
        match self.suit {
            Suit::Hearts => "♥",
            Suit::Spades => "♠",
            Suit::Clubs => "♣",
            Suit::Diamonds => "♦",
        }
    }
}

#[derive(Debug)]
pub struct Deck {
    cards: [Card; 52],
}

impl Deck {
    pub fn new() -> Self {
        let suits = [Suit::Hearts, Suit::Spades, Suit::Clubs, Suit::Diamonds];
        let ranks = [
            Rank::Two,
            Rank::Three,
            Rank::Four,
            Rank::Five,
            Rank::Six,
            Rank::Seven,
            Rank::Eight,
            Rank::Nine,
            Rank::Ten,
            Rank::Jack,
            Rank::Queen,
            Rank::King,
            Rank::Ace,
        ];

        let mut cards = [Card {
            suit: Suit::Hearts,
            rank: Rank::Two,
        }; 52];
        let mut index = 0;

        for &suit in &suits {
            for &rank in &ranks {
                cards[index] = Card { suit, rank };
                index += 1;
            }
        }

        Deck { cards }
    }

    pub fn shuffle(&mut self) {
        let mut rng = rng();
        self.cards.shuffle(&mut rng);
    }

    pub fn split(self) -> (Vec<Card>, Vec<Card>) {
        let mut player1 = Vec::new();
        let mut player2 = Vec::new();

        for (i, card) in self.cards.iter().enumerate() {
            if i % 2 == 0 {
                player1.push(*card);
            } else {
                player2.push(*card);
            }
        }

        (player1, player2)
    }
}

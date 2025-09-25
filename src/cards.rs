use crate::ring_buffer::RingBuffer;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{SeedableRng, rng};

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum Suit {
    Hearts = 0,
    Spades = 1,
    Clubs = 2,
    Diamonds = 3,
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord)]
#[repr(u8)]
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

/// Ultra-compact card representation: 1 byte total
/// Bits 0-1: Suit (4 suits = 2 bits)
/// Bits 2-7: Rank (13 ranks, values 2-14 = 6 bits)
#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(transparent)]
pub struct Card(u8);

impl Card {
    /// Create a new card from suit and rank
    pub fn new(suit: Suit, rank: Rank) -> Self {
        let suit_bits = (suit as u8) & 0b11; // 2 bits for suit
        let rank_bits = (rank as u8) & 0b111111; // 6 bits for rank
        Card((rank_bits << 2) | suit_bits)
    }

    /// Extract the suit from the packed representation
    pub fn suit(&self) -> Suit {
        match self.0 & 0b11 {
            0 => Suit::Hearts,
            1 => Suit::Spades,
            2 => Suit::Clubs,
            3 => Suit::Diamonds,
            _ => unreachable!(), // Only 2 bits, can't exceed 3
        }
    }

    /// Extract the rank from the packed representation
    pub fn rank(&self) -> Rank {
        let rank_value = (self.0 >> 2) & 0b111111;
        match rank_value {
            2 => Rank::Two,
            3 => Rank::Three,
            4 => Rank::Four,
            5 => Rank::Five,
            6 => Rank::Six,
            7 => Rank::Seven,
            8 => Rank::Eight,
            9 => Rank::Nine,
            10 => Rank::Ten,
            11 => Rank::Jack,
            12 => Rank::Queen,
            13 => Rank::King,
            14 => Rank::Ace,
            _ => unreachable!(), // Only valid rank values
        }
    }

    /// Get the numeric value of the card for comparison
    pub fn value(&self) -> u8 {
        (self.0 >> 2) & 0b111111
    }

    /// Get the suit symbol for display
    pub fn suit_symbol(&self) -> &'static str {
        match self.suit() {
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

        let mut cards = [Card::new(Suit::Hearts, Rank::Two); 52];
        let mut index = 0;

        for &suit in &suits {
            for &rank in &ranks {
                cards[index] = Card::new(suit, rank);
                index += 1;
            }
        }

        Deck { cards }
    }

    pub fn shuffle(&mut self) {
        let mut rng = rng();
        self.cards.shuffle(&mut rng);
    }

    pub fn shuffle_with_seed(&mut self, seed: u64) {
        let mut rng = StdRng::seed_from_u64(seed);
        self.cards.shuffle(&mut rng);
    }

    pub fn split(self) -> (PlayerHand, PlayerHand) {
        let mut player1 = PlayerHand::new();
        let mut player2 = PlayerHand::new();

        for (i, card) in self.cards.iter().enumerate() {
            if i % 2 == 0 {
                player1.add_card(*card);
            } else {
                player2.add_card(*card);
            }
        }

        (player1, player2)
    }
}

/// A player's hand using a ring buffer for efficient card management
#[derive(Debug)]
pub struct PlayerHand {
    cards: RingBuffer<Card, 52>,
}

impl PlayerHand {
    pub fn new() -> Self {
        Self {
            cards: RingBuffer::new(Card::new(Suit::Hearts, Rank::Two)),
        }
    }

    pub fn len(&self) -> usize {
        self.cards.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    /// Draw a card from the back of the hand (like drawing from a deck)
    pub fn draw_card(&mut self) -> Option<Card> {
        self.cards.pop_back()
    }

    /// Add a single card to the back of the hand
    pub fn add_card(&mut self, card: Card) {
        self.cards.push_back(card);
    }

    /// Transfer all cards from a battle buffer directly to the front of this hand
    /// This avoids creating any temporary Vec allocations
    pub fn take_battle_cards(&mut self, battle_buffer: &RingBuffer<Card, 52>) {
        // Add all cards from the battle buffer to the front of this hand
        for card in battle_buffer.iter() {
            self.cards.push_front(card);
        }
    }
}

mod cards;

use cards::{Card, Deck, Rank, Suit};
use clap::Parser;
use std::io::{self, Read, Write};

#[derive(Parser)]
#[command(name = "war-rust")]
#[command(about = "A War card game implementation in Rust")]
#[command(version = "0.1.0")]
struct Args {
    /// Enable test mode (game ends after 20 rounds)
    #[arg(short, long)]
    test: bool,

    /// Enable interactive mode (press SPACE to continue each round)
    #[arg(short, long)]
    interactive: bool,

    /// Show a guaranteed WAR scenario with banner
    #[arg(short, long)]
    war_demo: bool,
}

const WAR_BANNER: &str = r#"
  _____                  ____        _____
 |\    \   _____    ____|\   \   ___|\    \
 | |    | /    /|  /    /\    \ |    |\    \
 \/     / |    || |    |  |    ||    | |    |
 /     /_  \   \/ |    |__|    ||    |/____/
|     // \  \   \ |    .--.    ||    |\    \
|    |/   \ |    ||    |  |    ||    | |    |
|\ ___/\   \|   /||____|  |____||____| |____|
| |   | \______/ ||    |  |    ||    | |    |
 \|___|/\ |    | ||____|  |____||____| |____|
    \(   \|____|/   \(      )/    \(     )/
     '      )/       '      '      '     '
            '

"#;

struct WarGame {
    player1_cards: Vec<Card>,
    player2_cards: Vec<Card>,
    round: usize,
    test_mode: bool,
    interactive: bool,
}

impl WarGame {
    fn new(test_mode: bool, interactive: bool) -> Self {
        let mut deck = Deck::new();
        deck.shuffle();
        let (player1_cards, player2_cards) = deck.split();

        WarGame {
            player1_cards,
            player2_cards,
            round: 0,
            test_mode,
            interactive,
        }
    }

    fn wait_for_space(&self) {
        if self.interactive {
            print!("Press SPACE to continue...");
            io::stdout().flush().unwrap();

            let mut buffer = [0; 1];
            loop {
                if io::stdin().read_exact(&mut buffer).is_ok() {
                    if buffer[0] == b' ' {
                        break;
                    }
                }
            }
            println!(); // New line after space is pressed
        }
    }

    fn log_card_draw(&self, player: usize, card: Card) {
        println!(
            "🃏 Player {} draws: {}{:?} (value: {})",
            player,
            card.suit_symbol(),
            card.rank,
            card.value()
        );
    }

    fn draw_card(&mut self, player: usize) -> Option<Card> {
        match player {
            1 => self.player1_cards.pop(),
            2 => self.player2_cards.pop(),
            _ => None,
        }
    }

    fn add_cards_to_winner(&mut self, winner: usize, mut cards: Vec<Card>) {
        match winner {
            1 => {
                self.player1_cards.splice(0..0, cards.drain(..));
            }
            2 => {
                self.player2_cards.splice(0..0, cards.drain(..));
            }
            _ => {}
        }
    }

    fn play_round(&mut self) -> Option<usize> {
        self.round += 1;

        if self.player1_cards.is_empty() {
            return Some(2);
        }
        if self.player2_cards.is_empty() {
            return Some(1);
        }

        println!("\n--- Round {} ---", self.round);
        println!(
            "Player 1 has {} cards, Player 2 has {} cards",
            self.player1_cards.len(),
            self.player2_cards.len()
        );

        let mut battle_cards = Vec::new();

        // Draw initial cards
        let card1 = self.draw_card(1)?;
        let card2 = self.draw_card(2)?;
        self.log_card_draw(1, card1);
        self.log_card_draw(2, card2);
        battle_cards.push(card1);
        battle_cards.push(card2);

        println!(
            "Player 1 plays: {}{:?} (value: {})",
            card1.suit_symbol(),
            card1.rank,
            card1.value()
        );
        println!(
            "Player 2 plays: {}{:?} (value: {})",
            card2.suit_symbol(),
            card2.rank,
            card2.value()
        );

        if card1.value() > card2.value() {
            println!("Player 1 wins the round!");
            self.add_cards_to_winner(1, battle_cards);
        } else if card2.value() > card1.value() {
            println!("Player 2 wins the round!");
            self.add_cards_to_winner(2, battle_cards);
        } else {
            println!("WAR! Cards are equal ({})", card1.value());
            println!("{}", WAR_BANNER);
            self.wait_for_space();

            // War scenario - burn 3 cards each and draw another
            for i in 1..=3 {
                if let Some(burn1) = self.draw_card(1) {
                    self.log_card_draw(1, burn1);
                    battle_cards.push(burn1);
                    println!(
                        "Player 1 burns card {}: {}{:?}",
                        i,
                        burn1.suit_symbol(),
                        burn1.rank
                    );
                } else {
                    println!("Player 1 runs out of cards during war!");
                    return Some(2);
                }

                if let Some(burn2) = self.draw_card(2) {
                    self.log_card_draw(2, burn2);
                    battle_cards.push(burn2);
                    println!(
                        "Player 2 burns card {}: {}{:?}",
                        i,
                        burn2.suit_symbol(),
                        burn2.rank
                    );
                } else {
                    println!("Player 2 runs out of cards during war!");
                    return Some(1);
                }
            }

            // Draw the deciding cards
            if let Some(war_card1) = self.draw_card(1) {
                if let Some(war_card2) = self.draw_card(2) {
                    self.log_card_draw(1, war_card1);
                    self.log_card_draw(2, war_card2);
                    battle_cards.push(war_card1);
                    battle_cards.push(war_card2);

                    println!(
                        "War cards - Player 1: {}{:?} ({}), Player 2: {}{:?} ({})",
                        war_card1.suit_symbol(),
                        war_card1.rank,
                        war_card1.value(),
                        war_card2.suit_symbol(),
                        war_card2.rank,
                        war_card2.value()
                    );

                    if war_card1.value() > war_card2.value() {
                        println!("Player 1 wins the war!");
                        self.add_cards_to_winner(1, battle_cards);
                    } else if war_card2.value() > war_card1.value() {
                        println!("Player 2 wins the war!");
                        self.add_cards_to_winner(2, battle_cards);
                    } else {
                        println!(
                            "Another war would be needed, but for simplicity, Player 1 wins this tie!"
                        );
                        self.add_cards_to_winner(1, battle_cards);
                    }
                } else {
                    println!("Player 2 runs out of cards during war!");
                    return Some(1);
                }
            } else {
                println!("Player 1 runs out of cards during war!");
                return Some(2);
            }
        }

        self.wait_for_space();
        None // Game continues
    }

    fn play(&mut self) {
        println!("🎮 Starting War Card Game!");
        println!("Each player starts with 26 cards.");

        if self.test_mode {
            println!("🧪 TEST MODE: Game will end after 20 rounds.");
        }
        if self.interactive {
            println!("🎮 INTERACTIVE MODE: Press SPACE after each round to continue.");
        }
        println!();

        let max_rounds = if self.test_mode { 20 } else { 10000 };

        loop {
            if let Some(winner) = self.play_round() {
                println!("\n🎉 GAME OVER! 🎉");
                println!(
                    "Player {} wins the game after {} rounds!",
                    winner, self.round
                );
                println!(
                    "Final card counts - Player 1: {}, Player 2: {}",
                    self.player1_cards.len(),
                    self.player2_cards.len()
                );
                break;
            }

            // Check if we've reached the limit
            if self.round >= max_rounds {
                if self.test_mode {
                    println!("\n🧪 TEST MODE: Completed {} rounds!", self.round);
                    println!(
                        "Current card counts - Player 1: {}, Player 2: {}",
                        self.player1_cards.len(),
                        self.player2_cards.len()
                    );

                    if self.player1_cards.len() > self.player2_cards.len() {
                        println!("Player 1 is currently winning!");
                    } else if self.player2_cards.len() > self.player1_cards.len() {
                        println!("Player 2 is currently winning!");
                    } else {
                        println!("It's currently tied!");
                    }
                } else {
                    println!("\nGame limit reached! Declaring winner based on card count.");
                    if self.player1_cards.len() > self.player2_cards.len() {
                        println!("Player 1 wins with {} cards!", self.player1_cards.len());
                    } else if self.player2_cards.len() > self.player1_cards.len() {
                        println!("Player 2 wins with {} cards!", self.player2_cards.len());
                    } else {
                        println!("It's a tie!");
                    }
                }
                break;
            }
        }
    }

    fn create_war_scenario(&mut self) {
        println!("🎮 Creating WAR scenario for demonstration...");

        // Create a controlled scenario where both players have the same rank
        let war_card1 = Card {
            suit: Suit::Hearts,
            rank: Rank::King,
        };
        let war_card2 = Card {
            suit: Suit::Spades,
            rank: Rank::King,
        };

        // Set up cards so the first draw will be a war
        self.player1_cards = vec![
            Card {
                suit: Suit::Hearts,
                rank: Rank::Ace,
            }, // War deciding card
            Card {
                suit: Suit::Diamonds,
                rank: Rank::Four,
            }, // Burn card 3
            Card {
                suit: Suit::Clubs,
                rank: Rank::Three,
            }, // Burn card 2
            Card {
                suit: Suit::Spades,
                rank: Rank::Two,
            }, // Burn card 1
            war_card1, // Initial war card
        ];

        self.player2_cards = vec![
            Card {
                suit: Suit::Clubs,
                rank: Rank::Eight,
            }, // War deciding card
            Card {
                suit: Suit::Hearts,
                rank: Rank::Seven,
            }, // Burn card 3
            Card {
                suit: Suit::Spades,
                rank: Rank::Six,
            }, // Burn card 2
            Card {
                suit: Suit::Diamonds,
                rank: Rank::Five,
            }, // Burn card 1
            war_card2, // Initial war card
        ];

        self.round = 0;
    }
}

fn main() {
    let args = Args::parse();

    let mut game = WarGame::new(args.test, args.interactive);

    if args.war_demo {
        game.create_war_scenario();
    }

    game.play();
}

pub mod cards;
pub mod ring_buffer;

use cards::{Card, Deck, PlayerHand};
use clap::Parser;
use ring_buffer::RingBuffer;
use std::fmt;
use std::io::{self, Read, Write};
use std::mem;

#[derive(Debug)]
pub enum GameError {
    PlayerOutOfCards(usize),
    InvalidPlayerNumber(usize),
    BattleBufferFull,
    IoError(io::Error),
}

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GameError::PlayerOutOfCards(player) => write!(f, "Player {} is out of cards", player),
            GameError::InvalidPlayerNumber(player) => {
                write!(f, "Invalid player number: {}", player)
            }
            GameError::BattleBufferFull => write!(f, "Battle buffer is full - cannot continue war"),
            GameError::IoError(e) => write!(f, "I/O error: {}", e),
        }
    }
}

impl std::error::Error for GameError {}

impl From<io::Error> for GameError {
    fn from(error: io::Error) -> Self {
        GameError::IoError(error)
    }
}

type GameResult<T> = Result<T, GameError>;

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

    /// Set random seed for deterministic gameplay
    #[arg(short, long)]
    seed: Option<u64>,
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
    player1_cards: PlayerHand,
    player2_cards: PlayerHand,
    battle_buffer: RingBuffer<Card, 52>,
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
            battle_buffer: RingBuffer::new(Card::new(cards::Suit::Hearts, cards::Rank::Two)),
            round: 0,
            test_mode,
            interactive,
        }
    }

    fn new_with_seed(test_mode: bool, interactive: bool, seed: u64) -> Self {
        let mut deck = Deck::new();
        deck.shuffle_with_seed(seed);
        let (player1_cards, player2_cards) = deck.split();

        WarGame {
            player1_cards,
            player2_cards,
            battle_buffer: RingBuffer::new(Card::new(cards::Suit::Hearts, cards::Rank::Two)),
            round: 0,
            test_mode,
            interactive,
        }
    }

    fn wait_for_space(&self) -> GameResult<()> {
        if self.interactive {
            print!("Press SPACE to continue...");
            io::stdout().flush()?;

            let mut buffer = [0; 1];
            loop {
                match io::stdin().read_exact(&mut buffer) {
                    Ok(_) => {
                        if buffer[0] == b' ' {
                            break;
                        }
                    }
                    Err(e) => return Err(GameError::IoError(e)),
                }
            }
            println!(); // New line after space is pressed
        }
        Ok(())
    }

    fn log_card_draw(&self, player: usize, card: Card) {
        println!(
            "🃏 Player {} draws: {} {:?} (value: {})",
            player,
            card.suit_symbol(),
            card.rank(),
            card.value()
        );
    }

    fn draw_card(&mut self, player: usize) -> GameResult<Option<Card>> {
        match player {
            1 => Ok(self.player1_cards.draw_card()),
            2 => Ok(self.player2_cards.draw_card()),
            _ => Err(GameError::InvalidPlayerNumber(player)),
        }
    }

    fn add_cards_to_winner(&mut self, winner: usize) -> GameResult<()> {
        match winner {
            1 => {
                self.player1_cards.take_battle_cards(&self.battle_buffer);
            }
            2 => {
                self.player2_cards.take_battle_cards(&self.battle_buffer);
            }
            _ => return Err(GameError::InvalidPlayerNumber(winner)),
        }
        self.battle_buffer.clear();
        Ok(())
    }

    fn play_round(&mut self) -> GameResult<Option<usize>> {
        self.round += 1;

        if self.player1_cards.is_empty() {
            return Ok(Some(2));
        }
        if self.player2_cards.is_empty() {
            return Ok(Some(1));
        }

        println!("\n--- Round {} ---", self.round);
        println!(
            "Player 1 has {} cards, Player 2 has {} cards",
            self.player1_cards.len(),
            self.player2_cards.len()
        );

        // Clear and reuse the battle buffer
        self.battle_buffer.clear();

        // Draw initial cards
        let card1 = self.draw_card(1)?.ok_or(GameError::PlayerOutOfCards(1))?;
        let card2 = self.draw_card(2)?.ok_or(GameError::PlayerOutOfCards(2))?;
        self.log_card_draw(1, card1);
        self.log_card_draw(2, card2);
        self.battle_buffer.push_back(card1);
        self.battle_buffer.push_back(card2);

        println!(
            "Player 1 plays: {} {:?} (value: {})",
            card1.suit_symbol(),
            card1.rank(),
            card1.value()
        );
        println!(
            "Player 2 plays: {} {:?} (value: {})",
            card2.suit_symbol(),
            card2.rank(),
            card2.value()
        );

        if card1.value() > card2.value() {
            println!("Player 1 wins the round!");
            self.add_cards_to_winner(1)?;
        } else if card2.value() > card1.value() {
            println!("Player 2 wins the round!");
            self.add_cards_to_winner(2)?;
        } else {
            println!("WAR! Cards are equal ({})", card1.value());
            println!("{}", WAR_BANNER);
            self.wait_for_space()?;

            // War scenario - burn 3 cards each and draw another
            for i in 1..=3 {
                if let Some(burn1) = self.draw_card(1)? {
                    self.log_card_draw(1, burn1);
                    self.battle_buffer.push_back(burn1);
                    println!(
                        "Player 1 burns card {}: {} {:?}",
                        i,
                        burn1.suit_symbol(),
                        burn1.rank()
                    );
                } else {
                    println!("Player 1 runs out of cards during war!");
                    return Ok(Some(2));
                }

                if let Some(burn2) = self.draw_card(2)? {
                    self.log_card_draw(2, burn2);
                    self.battle_buffer.push_back(burn2);
                    println!(
                        "Player 2 burns card {}: {} {:?}",
                        i,
                        burn2.suit_symbol(),
                        burn2.rank()
                    );
                } else {
                    println!("Player 2 runs out of cards during war!");
                    return Ok(Some(1));
                }
            }

            // Draw the deciding cards
            if let Some(war_card1) = self.draw_card(1)? {
                if let Some(war_card2) = self.draw_card(2)? {
                    self.log_card_draw(1, war_card1);
                    self.log_card_draw(2, war_card2);
                    self.battle_buffer.push_back(war_card1);
                    self.battle_buffer.push_back(war_card2);

                    println!(
                        "War cards - Player 1: {} {:?} ({}), Player 2: {} {:?} ({})",
                        war_card1.suit_symbol(),
                        war_card1.rank(),
                        war_card1.value(),
                        war_card2.suit_symbol(),
                        war_card2.rank(),
                        war_card2.value()
                    );

                    if war_card1.value() > war_card2.value() {
                        println!("Player 1 wins the war!");
                        self.add_cards_to_winner(1)?;
                    } else if war_card2.value() > war_card1.value() {
                        println!("Player 2 wins the war!");
                        self.add_cards_to_winner(2)?;
                    } else {
                        println!(
                            "Another war would be needed, but for simplicity, Player 1 wins this tie!"
                        );
                        self.add_cards_to_winner(1)?;
                    }
                } else {
                    println!("Player 2 runs out of cards during war!");
                    return Ok(Some(1));
                }
            } else {
                println!("Player 1 runs out of cards during war!");
                return Ok(Some(2));
            }
        }

        self.wait_for_space()?;
        Ok(None) // Game continues
    }

    fn play(&mut self) -> GameResult<()> {
        println!("🎮 Starting War Card Game!");
        println!("Each player starts with 26 cards.");

        if self.test_mode {
            println!("🧪 TEST MODE: Game will end after 20 rounds.");
        }
        if self.interactive {
            println!("🎮 INTERACTIVE MODE: Press SPACE after each round to continue.");
        }
        println!();

        let max_rounds: usize = if self.test_mode { 20 } else { 10000 };

        loop {
            match self.play_round()? {
                Some(winner) => {
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
                None => {} // Game continues
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
        Ok(())
    }
}

fn show_memory_layout() {
    println!("\n📊 Memory Layout Information:");
    println!("Card size: {} bytes", mem::size_of::<Card>());
    println!("Card alignment: {} bytes", mem::align_of::<Card>());
    println!("Card needs drop: {}", mem::needs_drop::<Card>());

    println!("PlayerHand size: {} bytes", mem::size_of::<PlayerHand>());
    println!(
        "PlayerHand alignment: {} bytes",
        mem::align_of::<PlayerHand>()
    );
    println!("PlayerHand needs drop: {}", mem::needs_drop::<PlayerHand>());

    println!(
        "RingBuffer<Card, 52> size: {} bytes",
        mem::size_of::<RingBuffer<Card, 52>>()
    );
    println!(
        "RingBuffer<Card, 52> alignment: {} bytes",
        mem::align_of::<RingBuffer<Card, 52>>()
    );
    println!(
        "RingBuffer<Card, 52> needs drop: {}",
        mem::needs_drop::<RingBuffer<Card, 52>>()
    );

    println!("WarGame size: {} bytes", mem::size_of::<WarGame>());
    println!("WarGame alignment: {} bytes", mem::align_of::<WarGame>());
    println!("WarGame needs drop: {}", mem::needs_drop::<WarGame>());

    println!("\n🚀 ZERO HEAP ALLOCATIONS!");
    println!("✅ Entire game state lives on the stack");
    println!("✅ No Vec, no Box, no heap pointers");
    println!(
        "✅ Maximum predictable memory usage: {} bytes",
        mem::size_of::<WarGame>()
    );

    // For comparison, show what Vec<Card> would be like
    println!("\n📈 Comparison to Vec<Card>:");
    println!(
        "Vec<Card> size: {} bytes (just the pointer + metadata, data on heap)",
        mem::size_of::<Vec<Card>>()
    );
    println!(
        "Vec<Card> needs drop: {} (must manage heap memory)",
        mem::needs_drop::<Vec<Card>>()
    );
    println!();
}

fn main() {
    let args = Args::parse();

    show_memory_layout();

    let mut game = if let Some(seed) = args.seed {
        println!("🎲 Using seed: {}", seed);
        WarGame::new_with_seed(args.test, args.interactive, seed)
    } else {
        WarGame::new(args.test, args.interactive)
    };

    if let Err(e) = game.play() {
        eprintln!("❌ Game error: {}", e);
        std::process::exit(1);
    }
}

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use war_rust::cards::{Card, Deck, PlayerHand, Rank, Suit};

fn bench_card_creation(c: &mut Criterion) {
    c.bench_function("card_creation", |b| {
        b.iter(|| black_box(Card::new(Suit::Hearts, Rank::Ace)))
    });
}

fn bench_card_bitpacking_operations(c: &mut Criterion) {
    let card = Card::new(Suit::Spades, Rank::King);

    c.bench_function("card_suit_extraction", |b| {
        b.iter(|| black_box(card.suit()))
    });

    c.bench_function("card_rank_extraction", |b| {
        b.iter(|| black_box(card.rank()))
    });

    c.bench_function("card_value_extraction", |b| {
        b.iter(|| black_box(card.value()))
    });
}

fn bench_card_comparison(c: &mut Criterion) {
    let card1 = Card::new(Suit::Hearts, Rank::King);
    let card2 = Card::new(Suit::Spades, Rank::Queen);

    c.bench_function("card_value_comparison", |b| {
        b.iter(|| black_box(card1.value() > card2.value()))
    });
}

fn bench_deck_operations(c: &mut Criterion) {
    c.bench_function("deck_creation", |b| b.iter(|| black_box(Deck::new())));

    let mut group = c.benchmark_group("deck_shuffle");
    group.bench_function("shuffle", |b| {
        b.iter_batched(
            || Deck::new(),
            |mut deck| {
                deck.shuffle();
                black_box(deck)
            },
            criterion::BatchSize::SmallInput,
        )
    });

    group.bench_function("shuffle_with_seed", |b| {
        b.iter_batched(
            || Deck::new(),
            |mut deck| {
                deck.shuffle_with_seed(12345);
                black_box(deck)
            },
            criterion::BatchSize::SmallInput,
        )
    });
    group.finish();

    c.bench_function("deck_split", |b| {
        b.iter_batched(
            || {
                let mut deck = Deck::new();
                deck.shuffle_with_seed(42);
                deck
            },
            |deck| {
                let (player1, player2) = deck.split();
                black_box((player1, player2))
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

fn bench_player_hand_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("player_hand");

    group.bench_function("hand_creation", |b| b.iter(|| black_box(PlayerHand::new())));

    group.bench_function("add_card", |b| {
        b.iter_batched(
            || PlayerHand::new(),
            |mut hand| {
                let card = Card::new(Suit::Hearts, Rank::Ace);
                hand.add_card(card);
                black_box(hand)
            },
            criterion::BatchSize::SmallInput,
        )
    });

    group.bench_function("draw_card", |b| {
        b.iter_batched(
            || {
                let mut hand = PlayerHand::new();
                for i in 2..=14 {
                    if let Some(rank) = u8_to_rank(i as u8) {
                        hand.add_card(Card::new(Suit::Hearts, rank));
                    }
                }
                hand
            },
            |mut hand| black_box(hand.draw_card()),
            criterion::BatchSize::SmallInput,
        )
    });

    group.bench_function("hand_len", |b| {
        let hand = {
            let mut h = PlayerHand::new();
            for i in 2..=14 {
                if let Some(rank) = u8_to_rank(i as u8) {
                    h.add_card(Card::new(Suit::Hearts, rank));
                }
            }
            h
        };

        b.iter(|| black_box(hand.len()))
    });

    group.bench_function("hand_is_empty", |b| {
        let hand = PlayerHand::new();

        b.iter(|| black_box(hand.is_empty()))
    });

    group.finish();
}

fn bench_card_battle_simulation(c: &mut Criterion) {
    c.bench_function("simulate_card_battle", |b| {
        b.iter_batched(
            || {
                let mut deck = Deck::new();
                deck.shuffle_with_seed(42);
                let (mut player1, mut player2) = deck.split();

                // Set up a battle scenario
                let mut battle_cards = Vec::new();
                if let (Some(card1), Some(card2)) = (player1.draw_card(), player2.draw_card()) {
                    battle_cards.push(card1);
                    battle_cards.push(card2);
                }
                (player1, player2, battle_cards)
            },
            |(mut player1, mut player2, battle_cards)| {
                // Simulate winner taking cards
                if let (Some(card1), Some(card2)) = (battle_cards.get(0), battle_cards.get(1)) {
                    if card1.value() > card2.value() {
                        for &card in &battle_cards {
                            player1.add_card(card);
                        }
                    } else {
                        for &card in &battle_cards {
                            player2.add_card(card);
                        }
                    }
                }
                black_box((player1, player2))
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

// Helper function for converting u8 to Rank for benchmarking
fn u8_to_rank(value: u8) -> Option<Rank> {
    match value {
        2 => Some(Rank::Two),
        3 => Some(Rank::Three),
        4 => Some(Rank::Four),
        5 => Some(Rank::Five),
        6 => Some(Rank::Six),
        7 => Some(Rank::Seven),
        8 => Some(Rank::Eight),
        9 => Some(Rank::Nine),
        10 => Some(Rank::Ten),
        11 => Some(Rank::Jack),
        12 => Some(Rank::Queen),
        13 => Some(Rank::King),
        14 => Some(Rank::Ace),
        _ => None,
    }
}

criterion_group!(
    benches,
    bench_card_creation,
    bench_card_bitpacking_operations,
    bench_card_comparison,
    bench_deck_operations,
    bench_player_hand_operations,
    bench_card_battle_simulation
);

criterion_main!(benches);

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use war_rust::cards::{Card, Deck, PlayerHand, Rank, Suit};
use war_rust::ring_buffer::RingBuffer;

fn bench_full_game_simulation(c: &mut Criterion) {
    c.bench_function("complete_war_game_20_rounds", |b| {
        b.iter(|| black_box(simulate_war_game(42, 20)))
    });
}

fn bench_game_setup(c: &mut Criterion) {
    c.bench_function("game_setup_deck_shuffle_split", |b| {
        b.iter(|| {
            let mut deck = Deck::new();
            deck.shuffle_with_seed(42);
            let (player1, player2) = deck.split();
            black_box((player1, player2))
        })
    });
}

fn bench_battle_scenarios(c: &mut Criterion) {
    let mut group = c.benchmark_group("battle_scenarios");

    group.bench_function("simple_battle", |b| {
        b.iter_batched(
            setup_battle_scenario,
            |(mut p1, mut p2, mut battle_buffer)| {
                // Simulate a simple card battle
                if let (Some(card1), Some(card2)) = (p1.draw_card(), p2.draw_card()) {
                    battle_buffer.push_back(card1);
                    battle_buffer.push_back(card2);

                    if card1.value() > card2.value() {
                        p1.take_battle_cards(&battle_buffer);
                    } else {
                        p2.take_battle_cards(&battle_buffer);
                    }
                    battle_buffer.clear();
                }
                black_box((p1, p2, battle_buffer))
            },
            criterion::BatchSize::SmallInput,
        )
    });

    group.bench_function("war_scenario", |b| {
        b.iter_batched(
            setup_war_scenario,
            |(mut p1, mut p2, mut battle_buffer)| {
                // Simulate a war (equal cards)
                if let (Some(card1), Some(card2)) = (p1.draw_card(), p2.draw_card()) {
                    battle_buffer.push_back(card1);
                    battle_buffer.push_back(card2);

                    // Burn 3 cards each
                    for _ in 0..3 {
                        if let Some(burn1) = p1.draw_card() {
                            battle_buffer.push_back(burn1);
                        }
                        if let Some(burn2) = p2.draw_card() {
                            battle_buffer.push_back(burn2);
                        }
                    }

                    // Final battle cards
                    if let (Some(war_card1), Some(war_card2)) = (p1.draw_card(), p2.draw_card()) {
                        battle_buffer.push_back(war_card1);
                        battle_buffer.push_back(war_card2);

                        if war_card1.value() > war_card2.value() {
                            p1.take_battle_cards(&battle_buffer);
                        } else {
                            p2.take_battle_cards(&battle_buffer);
                        }
                        battle_buffer.clear();
                    }
                }
                black_box((p1, p2, battle_buffer))
            },
            criterion::BatchSize::SmallInput,
        )
    });

    group.finish();
}

fn bench_different_game_lengths(c: &mut Criterion) {
    let mut group = c.benchmark_group("game_lengths");

    for rounds in [5, 10, 20, 50].iter() {
        group.bench_with_input(BenchmarkId::new("rounds", rounds), rounds, |b, &rounds| {
            b.iter(|| black_box(simulate_war_game(12345, rounds)))
        });
    }

    group.finish();
}

fn bench_memory_operations(c: &mut Criterion) {
    c.bench_function("card_memory_operations", |b| {
        b.iter_batched(
            || {
                let mut deck = Deck::new();
                deck.shuffle_with_seed(999);
                let (player1, player2) = deck.split();
                (player1, player2)
            },
            |(mut p1, mut p2)| {
                // Simulate intensive card operations
                for _ in 0..10 {
                    // Draw cards
                    if let Some(card1) = p1.draw_card() {
                        if let Some(card2) = p2.draw_card() {
                            // Winner takes both cards (simulating battle result)
                            if card1.value() > card2.value() {
                                p1.add_card(card1);
                                p1.add_card(card2);
                            } else {
                                p2.add_card(card1);
                                p2.add_card(card2);
                            }
                        }
                    }

                    // Check lengths (common operation)
                    black_box(p1.len());
                    black_box(p2.len());
                    black_box(p1.is_empty());
                    black_box(p2.is_empty());
                }
                black_box((p1, p2))
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

// Helper functions

fn setup_battle_scenario() -> (PlayerHand, PlayerHand, RingBuffer<Card, 52>) {
    let mut deck = Deck::new();
    deck.shuffle_with_seed(777);
    let (player1, player2) = deck.split();
    let battle_buffer = RingBuffer::new(Card::new(Suit::Hearts, Rank::Two));
    (player1, player2, battle_buffer)
}

fn setup_war_scenario() -> (PlayerHand, PlayerHand, RingBuffer<Card, 52>) {
    // Create a scenario where war is likely
    let mut deck = Deck::new();
    deck.shuffle_with_seed(888);
    let (player1, player2) = deck.split();
    let battle_buffer = RingBuffer::new(Card::new(Suit::Hearts, Rank::Two));
    (player1, player2, battle_buffer)
}

fn simulate_war_game(seed: u64, max_rounds: usize) -> (usize, usize, usize) {
    let mut deck = Deck::new();
    deck.shuffle_with_seed(seed);
    let (mut player1, mut player2) = deck.split();
    let mut battle_buffer = RingBuffer::new(Card::new(Suit::Hearts, Rank::Two));

    let mut rounds = 0;

    for _ in 0..max_rounds {
        if player1.is_empty() || player2.is_empty() {
            break;
        }

        battle_buffer.clear();

        // Draw cards
        if let (Some(card1), Some(card2)) = (player1.draw_card(), player2.draw_card()) {
            battle_buffer.push_back(card1);
            battle_buffer.push_back(card2);

            if card1.value() > card2.value() {
                player1.take_battle_cards(&battle_buffer);
            } else if card2.value() > card1.value() {
                player2.take_battle_cards(&battle_buffer);
            } else {
                // War scenario - simplified for benchmarking
                for _ in 0..3 {
                    if let Some(burn1) = player1.draw_card() {
                        battle_buffer.push_back(burn1);
                    }
                    if let Some(burn2) = player2.draw_card() {
                        battle_buffer.push_back(burn2);
                    }
                }

                if let (Some(war_card1), Some(war_card2)) =
                    (player1.draw_card(), player2.draw_card())
                {
                    battle_buffer.push_back(war_card1);
                    battle_buffer.push_back(war_card2);

                    if war_card1.value() >= war_card2.value() {
                        player1.take_battle_cards(&battle_buffer);
                    } else {
                        player2.take_battle_cards(&battle_buffer);
                    }
                }
            }
        }

        rounds += 1;
    }

    (rounds, player1.len(), player2.len())
}

criterion_group!(
    benches,
    bench_full_game_simulation,
    bench_game_setup,
    bench_battle_scenarios,
    bench_different_game_lengths,
    bench_memory_operations
);

criterion_main!(benches);

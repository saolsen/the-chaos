use rayon::prelude::*;
use thiserror::Error;
use tracing::{span, Level};

const ROWS: usize = 6;
const COLS: usize = 7;

#[derive(Debug)]
pub struct Connect4Action {
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct Connect4State {
    pub board: Vec<Option<usize>>,
    pub next_player: usize,
}

impl Default for Connect4State {
    fn default() -> Self {
        Self {
            board: vec![None; ROWS * COLS],
            next_player: 0,
        }
    }
}

#[derive(Debug)]
pub enum Connect4Result {
    Winner(usize),
    Tie,
}

#[derive(Debug)]
pub enum Connect4Check {
    InProgress,
    Over(Connect4Result),
}

#[allow(clippy::identity_op)]
pub fn check_state(state: &Connect4State) -> Connect4Check {
    use Connect4Check::*;
    use Connect4Result::*;
    // Check vertical wins
    for col in 0..COLS {
        for row in 0..3 {
            match (
                state.board[col * ROWS + row + 0],
                state.board[col * ROWS + row + 1],
                state.board[col * ROWS + row + 2],
                state.board[col * ROWS + row + 3],
            ) {
                (Some(i), Some(j), Some(k), Some(l)) if i == j && j == k && k == l => {
                    return Over(Winner(i))
                }
                _ => (),
            }
        }
    }

    // Check horizontal wins
    for row in 0..ROWS {
        for col in 0..4 {
            match (
                state.board[(col + 0) * ROWS + row],
                state.board[(col + 1) * ROWS + row],
                state.board[(col + 2) * ROWS + row],
                state.board[(col + 3) * ROWS + row],
            ) {
                (Some(i), Some(j), Some(k), Some(l)) if i == j && j == k && k == l => {
                    return Over(Winner(i))
                }
                _ => (),
            }
        }
    }

    // Check diagonal up wins
    for col in 0..4 {
        for row in 0..3 {
            match (
                state.board[(col + 0) * ROWS + row + 0],
                state.board[(col + 1) * ROWS + row + 1],
                state.board[(col + 2) * ROWS + row + 2],
                state.board[(col + 3) * ROWS + row + 3],
            ) {
                (Some(i), Some(j), Some(k), Some(l)) if i == j && j == k && k == l => {
                    return Over(Winner(i))
                }
                _ => (),
            }
        }
    }

    // Check diagonal down wins
    for col in 0..4 {
        for row in 3..6 {
            match (
                state.board[(col + 0) * ROWS + row - 0],
                state.board[(col + 1) * ROWS + row - 1],
                state.board[(col + 2) * ROWS + row - 2],
                state.board[(col + 3) * ROWS + row - 3],
            ) {
                (Some(i), Some(j), Some(k), Some(l)) if i == j && j == k && k == l => {
                    return Over(Winner(i))
                }
                _ => (),
            }
        }
    }

    // Check for tie
    for col in 0..COLS {
        if state.board[col * ROWS + ROWS - 1].is_none() {
            return InProgress;
        }
    }

    Over(Tie)
}

#[derive(Error, Debug)]
pub enum ActionError {
    #[error("Column must be between 0 and 6. Got `{0}`.")]
    UnknownColumn(usize),
    #[error("Column `{0}` is full.")]
    FullColumn(usize),
}

pub fn check_action(state: &Connect4State, action: &Connect4Action) -> bool {
    if action.column >= COLS {
        return false;
    }
    state.board[action.column * ROWS + ROWS - 1].is_none()
}

pub fn apply_action(
    state: &mut Connect4State,
    action: &Connect4Action,
) -> Result<Connect4Check, ActionError> {
    use ActionError::*;
    if action.column >= COLS {
        return Err(UnknownColumn(action.column));
    }
    for row in 0..ROWS {
        let cell = &mut state.board[action.column * ROWS + row];
        if cell.is_none() {
            *cell = Some(state.next_player);
            state.next_player = 1 - state.next_player;
            return Ok(check_state(state));
        }
    }
    Err(FullColumn(action.column))
}

fn play(
    state: &mut Connect4State,
    blue_agent: fn(&Connect4State) -> Connect4Action,
    red_agent: fn(&Connect4State) -> Connect4Action,
) -> Result<Connect4Result, ActionError> {
    loop {
        let action = if state.next_player == 0 {
            blue_agent(state)
        } else {
            red_agent(state)
        };
        apply_action(state, &action)?;
        if let Connect4Check::Over(result) = check_state(state) {
            return Ok(result);
        }
    }
}

fn rand_agent(state: &Connect4State) -> Connect4Action {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    loop {
        // Generate random actions until one is valid.
        let action = Connect4Action {
            column: rng.gen_range(0..COLS),
        };
        if check_action(state, &action) {
            return action;
        }
    }
}

fn mcts_agent(state: &Connect4State) -> Connect4Action {
    // For each possible action, take the action and then simulate multiple random games from that
    // state.
    // Keep track of the number of wins for each action.
    // Pick the action with the highest win rate.
    let player = state.next_player;

    let _span = span!(Level::TRACE, "mcts agent turn").entered();

    (0..COLS)
        .into_par_iter()
        .map(|col| Connect4Action { column: col })
        .filter(|action| check_action(state, action))
        .map(|action| {
            let _span = span!(Level::TRACE, "mcts action", col = action.column).entered();
            let mut next_state = state.clone();
            apply_action(&mut next_state, &action).unwrap();

            // Simulate 10000 games from this action.
            let score = (0..100)
                .into_par_iter()
                .map(move |i| {
                    let _span = span!(Level::TRACE, "mcts simulation", i = i).entered();
                    let mut state = next_state.clone();
                    match play(&mut state, rand_agent, rand_agent).unwrap() {
                        Connect4Result::Winner(winner) => {
                            if winner == player {
                                1
                            } else {
                                -1
                            }
                        }
                        Connect4Result::Tie => 0,
                    }
                })
                .sum::<i32>() as f32
                / 100.;
            (action, score)
        })
        // Pick the action with the highest score.
        .max_by(|(_, score1), (_, score2)| score1.partial_cmp(score2).unwrap())
        .map(|(action, _)| action)
        .unwrap()
}

// 0.14s for 10 release
// 0.24s for 10 release with rayon... slower...
// 2.26s for 100 with rayon just the 0..100 loop
// 1.82s for 100 with rayon everything, kewl
// 1.8 if I make the main par too, obv output out of order now
fn main() {
    println!("begin");

    use tracing_chrome::ChromeLayerBuilder;
    use tracing_subscriber::{prelude::*, registry::Registry};

    let (chrome_layer, _guard) = ChromeLayerBuilder::new().build();
    tracing_subscriber::registry().with(chrome_layer).init();

    (0..100).into_par_iter().for_each(|i| {
        let span = span!(Level::TRACE, "Game", i = i);
        _ = span.enter();
        let mut state = Connect4State::default();
        let result = play(&mut state, rand_agent, mcts_agent).unwrap();
        println!("Game {}: {:?}", i, result);
    });
}

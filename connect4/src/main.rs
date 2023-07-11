use std::collections::HashMap;
use thiserror::Error;

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

impl Connect4State {
    pub fn new() -> Self {
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
    if action.column >= COLS || action.column < 0 {
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
            break;
        }
        if row == ROWS - 1 {
            return Err(FullColumn(action.column));
        }
    }
    state.next_player = 1 - state.next_player;
    Ok(check_state(state))
}

fn play(
    state: &mut Connect4State,
    blue_agent: fn(&Connect4State) -> Connect4Action,
    red_agent: fn(&Connect4State) -> Connect4Action,
) -> Result<Connect4Result, ActionError> {
    loop {
        let action = if state.next_player == 0 {
            blue_agent(&state)
        } else {
            red_agent(&state)
        };
        apply_action(state, &action)?;
        if let Connect4Check::Over(result) = check_state(&state) {
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
    //use rayon::prelude::*;

    // For each possible action, take the action and then simulate multiple random games from that
    // state.
    // Keep track of the number of wins for each action.
    // Pick the action with the highest win rate.
    let player = state.next_player;

    let mut actions = vec![];
    for col in 0..COLS {
        let action = Connect4Action { column: col };
        if check_action(state, &action) {
            actions.push(action);
        }
    }

    let mut action_scores = HashMap::new();
    for action in actions {
        let mut wins: f32 = 0.;
        let mut losses: f32 = 0.;
        let mut ties: f32 = 0.;
        for _ in 0..100 {
            let mut next_state = state.clone();
            apply_action(&mut next_state, &action).unwrap();
            match play(&mut next_state, rand_agent, rand_agent).unwrap() {
                Connect4Result::Winner(winner) => {
                    if winner == player {
                        wins += 1.;
                    } else {
                        losses += 1.;
                    }
                }
                Connect4Result::Tie => {
                    ties += 1.;
                }
            }
        }
        let score: f32 = (wins - losses) / (wins + losses + ties);
        action_scores.insert(action.column, score);
    }

    let mut best_score: f32 = -1.;
    let mut best_action = None;
    for (action, score) in action_scores {
        if score > best_score {
            best_score = score;
            best_action = Some(action);
        }
    }

    Connect4Action{column: best_action.unwrap()}
}

fn main() {
    for i in 0..10 {
        let mut state = Connect4State::new();
        let result = play(&mut state, rand_agent, mcts_agent).unwrap();
        println!("Game {}: {:?}", i, result);
    }
}

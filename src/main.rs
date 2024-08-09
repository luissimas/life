use core::fmt;
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    execute,
    style::Print,
    terminal::{
        disable_raw_mode, enable_raw_mode, size, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use std::{
    collections::{self, vec_deque},
    fmt::Debug,
    io::{self, stdout},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread, time, usize,
};

fn main() -> io::Result<()> {
    // Flag to handle SIGINT (Ctrl+C)
    let stop_signal = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&stop_signal))?;

    // Initialize game
    let (width, height) = size().unwrap();
    let mut game = Game::new(&BoardShape {
        width: width.into(),
        height: height.into(),
    });
    game.seed();

    // Enter alternate screen terminal buffer
    execute!(stdout(), EnterAlternateScreen, Hide)?;
    // enable_raw_mode()?;

    // While no stop signal was received, keep iterating
    while !stop_signal.load(Ordering::Relaxed) {
        execute!(stdout(), MoveTo(0, 0))?;
        game.display()?;
        game.next();
        thread::sleep(time::Duration::from_millis(200));
    }

    // Reset terminal screen
    // disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen, Show)
}

#[derive(Clone, Debug)]
enum CellState {
    Alive,
    Dead,
}

impl fmt::Display for CellState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = match self {
            CellState::Alive => "█",
            CellState::Dead => "⠀",
        };
        write!(f, "{}", c)
    }
}

#[derive(Debug)]
struct Game {
    // TODO: use a hashset for this
    board: [Vec<Vec<CellState>>; 2],
    generation: usize,
    population: u32,
}

struct BoardShape {
    width: usize,
    height: usize,
}

impl Game {
    fn new(shape: &BoardShape) -> Game {
        Game {
            board: [
                vec![vec![CellState::Dead; shape.width]; shape.height - 1],
                vec![vec![CellState::Dead; shape.width]; shape.height - 1],
            ],
            generation: 0,
            population: 0,
        }
    }

    fn seed(&mut self) {
        for i in 0..self.board[0].len() {
            for j in 0..self.board[0][0].len() {
                // A 50% chance of populating the cell
                if rand::random::<f32>() < 0.5 {
                    self.board[0][i][j] = CellState::Alive;
                    self.population += 1;
                }
            }
        }
    }

    fn next(&mut self) {
        let current = self.generation % 2;
        self.generation += 1;
        let next = self.generation % 2;
        let board = &mut self.board;
        for i in 0..board[current].len() {
            for j in 0..board[current][i].len() {
                let cell_state = &board[current][i][j];
                let row_range = if i > 0 { i - 1..i + 1 } else { i..i + 1 };
                let col_range = if j > 0 { j - 1..j + 1 } else { j..j + 1 };

                let neighbors: usize = row_range
                    .flat_map(|i| col_range.clone().map(move |j| (i, j)))
                    .fold(0, |acc, coords| {
                        let (i, j) = coords;
                        acc + match board[current].get(i) {
                            Some(row) => match row.get(j) {
                                Some(&CellState::Alive) => 1,
                                Some(&CellState::Dead) => 0,
                                None => 0,
                            },
                            None => 0,
                        }
                    });
                let next_state = match (cell_state, neighbors) {
                    (CellState::Alive, 2) => CellState::Alive,
                    (CellState::Alive, 3) => CellState::Alive,
                    (CellState::Dead, 3) => CellState::Alive,
                    _ => CellState::Dead,
                };
                board[next][i][j] = next_state;
            }
        }
        self.population = board[next]
            .iter()
            .flatten()
            .map(|cell| match cell {
                &CellState::Alive => 1,
                &CellState::Dead => 0,
            })
            .sum()
    }

    fn display(&self) -> io::Result<()> {
        let current = self.generation % 2;
        let current_board = &self.board[current];
        for row in current_board {
            for cell in row {
                execute!(stdout(), Print(cell))?;
            }
        }
        execute!(
            stdout(),
            Print(format!(
                "Generation: {}  Population: {}      ",
                self.generation, self.population
            ))
        )
    }
}

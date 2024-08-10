use crossterm::{
    cursor::{Hide, MoveTo, Show},
    execute,
    style::Print,
    terminal::{size, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    collections::HashSet,
    io::{self, stdout},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread, time,
};

fn main() -> io::Result<()> {
    // Flag to handle SIGINT (Ctrl+C)
    let stop_signal = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&stop_signal))?;

    // Initialize game
    let (width, height) = size().unwrap();
    let mut game = Game::new(BoardShape {
        width: width.into(),
        height: height.into(),
    });
    game.seed();

    // Enter alternate screen terminal buffer
    execute!(stdout(), EnterAlternateScreen, Hide)?;
    // enable_raw_mode()?;

    // While no stop signal was received, keep iterating
    while !stop_signal.load(Ordering::Relaxed) {
        execute!(stdout(), Clear(ClearType::All), MoveTo(0, 0))?;
        game.display()?;
        game.next();
        thread::sleep(time::Duration::from_millis(200));
    }

    // Reset terminal screen
    // disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen, Show)
}

type Cell = (u16, u16);

fn cell_neighbors(cell: &Cell) -> Vec<Cell> {
    let (i, j) = cell;
    let mut neighbors = Vec::new();
    let row_range = if *i > 0 { i - 1..=i + 1 } else { *i..=i + 1 };
    let col_range = if *j > 0 { j - 1..=j + 1 } else { *j..=j + 1 };

    for i in row_range {
        for j in col_range.clone() {
            if (i, j) != *cell {
                neighbors.push((i, j))
            }
        }
    }

    return neighbors;
}

#[derive(Debug)]
struct Game {
    cells: HashSet<Cell>,
    board_shape: BoardShape,
    generation: u32,
}

#[derive(Debug)]
struct BoardShape {
    width: u16,
    height: u16,
}

impl Game {
    fn new(shape: BoardShape) -> Game {
        Game {
            cells: HashSet::new(),
            board_shape: shape,
            generation: 0,
        }
    }

    fn seed(&mut self) {
        let BoardShape { width, height } = self.board_shape;
        for i in 0..width {
            for j in 0..height {
                // A 50% chance of populating the cell
                if rand::random::<f32>() < 0.2 {
                    self.cells.insert((i, j));
                }
            }
        }
    }

    fn next(&mut self) {
        let mut next_generation = HashSet::new();
        for cell in self.cells.iter() {
            let neighbors = cell_neighbors(cell);
            // Check if the current cell should live on to the next generation
            let alive_neighbors = neighbors
                .iter()
                .filter(|cell| self.cells.contains(cell))
                .count();
            if let 2 | 3 = alive_neighbors {
                next_generation.insert(*cell);
            }

            // Check if any of its dead neighbors should become alive
            let dead_neighbors = neighbors.iter().filter(|cell| !self.cells.contains(cell));
            for cell in dead_neighbors {
                let alive_neighbors = cell_neighbors(cell).iter().fold(0, |acc, cell| {
                    if self.cells.contains(cell) {
                        acc + 1
                    } else {
                        acc
                    }
                });
                if alive_neighbors == 3 {
                    next_generation.insert(*cell);
                }
            }
        }

        self.cells = next_generation;
        self.generation += 1;
    }

    fn display(&self) -> io::Result<()> {
        for (row, col) in &self.cells {
            execute!(stdout(), MoveTo(*row, *col), Print("█"))?;
        }
        execute!(
            stdout(),
            MoveTo(0, self.board_shape.height),
            Clear(ClearType::CurrentLine),
            Print(format!(
                "Generation: {}  Population: {}",
                self.generation,
                self.cells.len()
            ))
        )
    }
}

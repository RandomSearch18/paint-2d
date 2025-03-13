use std::{
    io::Write,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use crossterm::{
    ExecutableCommand, cursor,
    event::{self, Event, KeyCode},
    style::Print,
    terminal,
};

struct PaintCursor<'a> {
    row: u16,
    col: u16,
    screen_rows: &'a u16,
    screen_cols: &'a u16,
}

impl<'a> PaintCursor<'a> {
    fn new(row: u16, col: u16, screen_size: &'a (u16, u16)) -> Self {
        PaintCursor {
            row,
            col,
            screen_cols: &screen_size.0,
            screen_rows: &screen_size.1,
        }
    }

    fn left(&mut self) {
        if self.row > 0 {
            self.row -= 1;
        }
    }

    fn right(&mut self) {
        if self.row < self.screen_cols - 1 {
            self.row += 1;
        }
    }
}

/// All the state and main methods for the TUI program
struct Paint2D<'a> {
    stdout: std::io::Stdout,
    running: Arc<AtomicBool>,
    cursor: PaintCursor<'a>,
    /// `(height, width)` i.e. (cols, rows)
    screen_size: (u16, u16),
}

impl<'a> Paint2D<'a> {
    fn new(screen_size: &'a (u16, u16)) -> Self {
        Paint2D {
            stdout: std::io::stdout(),
            running: Arc::new(AtomicBool::new(true)),
            cursor: PaintCursor::new(0, 0, screen_size),
            screen_size: (1, 1),
        }
    }

    fn setup(&mut self) -> std::io::Result<()> {
        let is_running = self.running.clone();
        // We use a separate thread to handle Ctrl+C, just in case the main thread is blocked
        // Credit: Ken Salter, https://github.com/plecos/ctrlc-crossterm, original code MIT-licensed
        std::thread::spawn(move || -> std::io::Result<()> {
            loop {
                // 100 ms timeout is CPU-friendly, apparently
                if event::poll(std::time::Duration::from_millis(45))? {
                    if let Event::Key(key_event) = event::read()? {
                        if key_event.code == KeyCode::Char('c')
                            && key_event
                                .modifiers
                                .contains(crossterm::event::KeyModifiers::CONTROL)
                        {
                            print!("Ctrl+C pressed, exiting...\r\n");
                            is_running.store(false, Ordering::SeqCst);
                        }
                    }
                }
            }
        });

        terminal::enable_raw_mode()?;
        self.stdout.execute(terminal::EnterAlternateScreen)?;
        // Hide the cursor as much as we can
        self.stdout
            .execute(cursor::SetCursorStyle::SteadyUnderScore)?;
        self.stdout.execute(cursor::MoveTo(0, 0))?;
        self.stdout.execute(cursor::Hide)?;
        Ok(())
    }

    fn draw_cursor(&mut self) -> std::io::Result<()> {
        self.stdout
            .execute(cursor::MoveTo(self.cursor.row, self.cursor.col))?;
        self.stdout.execute(Print("X"))?;
        Ok(())
    }

    fn run(&mut self) -> std::io::Result<()> {
        while self.running.load(Ordering::SeqCst) {
            while event::poll(Duration::from_millis(50))? {
                match event::read()? {
                    Event::Key(key) => match key.code {
                        event::KeyCode::Char('q') => {
                            self.running.store(false, Ordering::SeqCst);
                        }
                        event::KeyCode::Left => {
                            self.cursor.left();
                        }
                        event::KeyCode::Right => {
                            self.cursor.right();
                        }
                        _ => {}
                    },
                    Event::Resize(cols, rows) => {
                        self.screen_size = (cols, rows);
                    }
                    _ => {}
                }
            }
            self.draw_cursor()?;
            self.stdout.flush()?;
        }
        Ok(())
    }
}

impl<'a> Drop for Paint2D<'a> {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
        let _ = self.stdout.execute(cursor::Show);
        let _ = self.stdout.execute(terminal::LeaveAlternateScreen);
        let _ = self
            .stdout
            .execute(cursor::SetCursorStyle::DefaultUserShape);
    }
}

fn main() -> std::io::Result<()> {
    let terminal_size: (u16, u16) = terminal::size().unwrap_or((1, 1));
    let mut app = Paint2D::new(&terminal_size);
    app.setup()?;
    app.run()?;
    Ok(())
}

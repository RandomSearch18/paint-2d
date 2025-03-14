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
    event::{self, Event},
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

    fn left(&mut self, by: u16) {
        if self.row >= by {
            self.row -= by;
        } else {
            let underflow = by - self.row;
            self.row = self.screen_cols - underflow;
        }
    }

    fn right(&mut self, by: u16) {
        if self.row < self.screen_cols - by {
            self.row += by;
        } else {
            let overflow = by - (self.screen_cols - self.row);
            self.row = 0 + overflow;
        }
    }

    fn up(&mut self, by: u16) {
        if self.col >= by {
            self.col -= by;
        } else {
            let underflow = by - self.col;
            self.col = self.screen_rows - underflow;
        }
    }

    fn down(&mut self, by: u16) {
        if self.col < self.screen_rows - by {
            self.col += by;
        } else {
            let overflow = by - (self.screen_rows - self.col);
            self.col = 0 + overflow;
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

    fn redraw_screen(&mut self) -> std::io::Result<()> {
        self.stdout
            .execute(terminal::Clear(terminal::ClearType::All))?;
        self.draw_cursor()?;
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
                        event::KeyCode::Char('c') => {
                            if key.modifiers.contains(event::KeyModifiers::CONTROL) {
                                // Ctrl+C has been pressed
                                self.running.store(false, Ordering::SeqCst);
                            }
                        }
                        event::KeyCode::Left => {
                            let is_fast = key.modifiers.contains(event::KeyModifiers::CONTROL);
                            let movement = if is_fast { 8 } else { 1 };
                            self.cursor.left(movement);
                        }
                        event::KeyCode::Right => {
                            let is_fast = key.modifiers.contains(event::KeyModifiers::CONTROL);
                            let movement = if is_fast { 8 } else { 1 };
                            self.cursor.right(movement);
                        }
                        event::KeyCode::Up => {
                            let is_fast = key.modifiers.contains(event::KeyModifiers::CONTROL);
                            let movement = if is_fast { 2 } else { 1 };
                            self.cursor.up(movement);
                        }
                        event::KeyCode::Down => {
                            let is_fast = key.modifiers.contains(event::KeyModifiers::CONTROL);
                            let movement = if is_fast { 2 } else { 1 };
                            self.cursor.down(movement);
                        }
                        _ => {}
                    },
                    Event::Resize(cols, rows) => {
                        self.screen_size = (cols, rows);
                    }
                    _ => {}
                }
            }
            self.redraw_screen()?;
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

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
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal,
};

struct PaintCursor {
    row: u16,
    col: u16,
    screen_rows: u16,
    screen_cols: u16,
    color: Color,
}

impl PaintCursor {
    fn new(row: u16, col: u16, screen_size: (u16, u16)) -> Self {
        PaintCursor {
            row,
            col,
            screen_cols: screen_size.0,
            screen_rows: screen_size.1,
            color: Color::White,
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

    fn set_canvas_size(&mut self, size: &(u16, u16)) {
        self.screen_cols = size.0;
        self.screen_rows = size.1;
    }
}

/// All the state and main methods for the TUI program
struct Paint2D {
    stdout: std::io::Stdout,
    running: Arc<AtomicBool>,
    cursor: PaintCursor,
    /// `(height, width)` i.e. (cols, rows)
    terminal_size: (u16, u16),
    color_canvas: Vec<Vec<Option<Color>>>,
}

impl Paint2D {
    fn new(terminal_size: &(u16, u16)) -> Self {
        let canvas_size = (terminal_size.0, terminal_size.1 - 1);
        Paint2D {
            stdout: std::io::stdout(),
            running: Arc::new(AtomicBool::new(true)),
            cursor: PaintCursor::new(0, 0, canvas_size),
            terminal_size: terminal_size.clone(),
            color_canvas: vec![vec![None; canvas_size.0.into()]; canvas_size.1.into()],
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
        // How many extra characters to the left are printed as part of the cursor
        let offset = 1;
        let row: i32 = (self.cursor.row as i32 - offset).into();
        execute!(
            self.stdout,
            cursor::MoveTo(row.try_into().unwrap_or(0), self.cursor.col),
            SetForegroundColor(Color::DarkGrey),
            Print('├'),
            ResetColor,
            Print("X"),
            SetForegroundColor(Color::DarkGrey),
            Print('┤'),
            ResetColor,
        )?;
        Ok(())
    }

    fn redraw_screen(&mut self) -> std::io::Result<()> {
        self.stdout
            .execute(terminal::Clear(terminal::ClearType::All))?;
        self.stdout.execute(cursor::MoveTo(0, 0))?;
        for r in 0..self.terminal_size.1 - 1 {
            self.stdout.execute(cursor::MoveTo(0, r))?;
            for c in 0..self.terminal_size.0 {
                // None if the access is out of bounds, or if the colour is transparent
                let color = self
                    .color_canvas
                    .get(r as usize)
                    .and_then(|row| row.get(c as usize))
                    .copied()
                    .flatten();

                if let Some(color) = color {
                    self.stdout.execute(SetBackgroundColor(color))?;
                    self.stdout.execute(Print(" "))?;
                    self.stdout.execute(ResetColor)?;
                } else {
                    // self.stdout.execute(cursor::MoveRight(1))?;
                    self.stdout.execute(Print(" "))?;
                }
            }
        }
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
                        event::KeyCode::Char(' ') => {
                            let row = self.cursor.row as usize;
                            let col = self.cursor.col as usize;
                            self.color_canvas[col][row] = Some(self.cursor.color);
                        }
                        _ => {}
                    },
                    Event::Resize(cols, rows) => {
                        self.terminal_size = (cols, rows);
                        self.cursor.set_canvas_size(&(cols, rows - 1));
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

impl Drop for Paint2D {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
        let _ = self.stdout.execute(cursor::Show);
        // let _ = self.stdout.execute(terminal::LeaveAlternateScreen);
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

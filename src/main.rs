use std::{
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

/// All the state and main methods for the TUI program
struct Paint2D {
    stdout: std::io::Stdout,
}

impl Paint2D {
    fn new() -> Self {
        Paint2D {
            stdout: std::io::stdout(),
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
        self.stdout.execute(Print("A"))?;
        Ok(())
    }

    fn run(&mut self) -> std::io::Result<()> {
        let running = Arc::new(AtomicBool::new(true));

        let is_running = running.clone();
        ctrlc::set_handler({
            move || {
                print!("AAH");
                is_running.store(false, Ordering::SeqCst);
            }
        })
        .expect("Ctrl+C handler did not initialise correctly");

        while running.load(Ordering::SeqCst) {
            while event::poll(Duration::from_millis(50))? {
                match event::read()? {
                    Event::Key(key) => match key.code {
                        event::KeyCode::Char('q') => {
                            running.store(false, Ordering::SeqCst);
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
        }
        Ok(())
    }
}

impl Drop for Paint2D {
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
    let mut app = Paint2D::new();
    app.setup()?;
    app.run()?;
    Ok(())
}

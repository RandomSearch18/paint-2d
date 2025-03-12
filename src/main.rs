use crossterm::{ExecutableCommand, cursor, terminal};

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
        self.stdout.execute(terminal::EnterAlternateScreen)?;
        self.stdout.execute(cursor::Hide)?;
        terminal::enable_raw_mode()?;
        Ok(())
    }

    fn run(&mut self) -> std::io::Result<()> {
        loop {}
    }
}

impl Drop for Paint2D {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
        let _ = self.stdout.execute(cursor::Show);
        let _ = self.stdout.execute(terminal::LeaveAlternateScreen);
    }
}

fn main() -> std::io::Result<()> {
    let mut app = Paint2D::new();
    app.setup()?;
    app.run()?;
    Ok(())
}

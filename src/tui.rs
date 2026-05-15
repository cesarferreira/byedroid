//! Terminal initialization and panic-safe teardown.
use anyhow::Result;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::{self, Stdout, stdout};
use std::panic;

pub fn init_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let _ = restore_terminal();
        original_hook(info);
    }));

    Ok(terminal)
}

pub fn restore_terminal() -> Result<()> {
    disable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, LeaveAlternateScreen)?;
    Ok(())
}

pub fn suspend_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    // Tear down TUI state
    disable_raw_mode()?;
    execute!(stdout(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    // Send SIGTSTP to ourselves — this blocks until SIGCONT (e.g. using `fg` in a shell)
    signal_hook::low_level::emulate_default_handler(signal_hook::consts::SIGTSTP)
        .expect("failed to send SIGTSTP");
    // Restore TUI state after the process resumes
    enable_raw_mode()?;
    terminal.hide_cursor()?;
    execute!(stdout(), EnterAlternateScreen)?;
    // Full redraw on restore
    terminal.clear()?;
    Ok(())
}

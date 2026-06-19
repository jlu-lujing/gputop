mod app;
mod cpu;
mod event;
mod gpu;
mod ui;

use app::App;
use ui::render;

fn main() {
    // Handle --version flag
    for arg in std::env::args().skip(1) {
        if arg == "--version" || arg == "-V" {
            println!("gputop {}", env!("CARGO_PKG_VERSION"));
            return;
        }
    }

    let result = run();

    if let Err(e) = result {
        eprintln!("gputop error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> std::io::Result<()> {
    let mut terminal = ratatui::init();
    let result = run_app(&mut terminal);
    ratatui::restore();
    result
}

fn run_app(terminal: &mut ratatui::DefaultTerminal) -> std::io::Result<()> {
    let mut app = App::new();

    // Initial data refresh
    app.refresh_data();

    loop {
        // Refresh data at intervals
        if app.last_refresh.elapsed() >= app.refresh_interval {
            app.refresh_data();
        }

        // Render
        terminal.draw(|frame| render(frame, &app))?;

        // Handle events
        app.handle_events();

        if !app.running {
            break;
        }
    }

    Ok(())
}

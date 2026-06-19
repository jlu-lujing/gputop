use crossterm::event::KeyCode;

use crate::app::{App, SortColumn};

pub fn handle_key(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Char('q') | KeyCode::Esc => {
            app.running = false;
        }
        KeyCode::Char('/') => {
            // Cycle sort column
            app.sort_column = next_sort_column(app.sort_column);
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if let Some(idx) = app.selected_process {
                if idx > 0 {
                    app.selected_process = Some(idx - 1);
                }
            } else if !app.processes.is_empty() {
                app.selected_process = Some(0);
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if let Some(idx) = app.selected_process {
                if idx < app.processes.len().saturating_sub(1) {
                    app.selected_process = Some(idx + 1);
                }
            } else if !app.processes.is_empty() {
                app.selected_process = Some(0);
            }
        }
        KeyCode::Char('c') => {
            // Send SIGINT to selected process
            if let Err(e) = kill_selected_process(app, "SIGINT") {
                app.error = Some(e);
            }
        }
        KeyCode::Char('t') => {
            // Send SIGTERM to selected process
            if let Err(e) = kill_selected_process(app, "SIGTERM") {
                app.error = Some(e);
            }
        }
        KeyCode::Char('K') => {
            // Send SIGKILL to selected process
            if let Err(e) = kill_selected_process(app, "SIGKILL") {
                app.error = Some(e);
            }
        }
        KeyCode::Char('e') => {
            // Clear error message
            app.error = None;
        }
        _ => {}
    }
}

fn kill_selected_process(app: &App, signal: &str) -> Result<(), String> {
    let pid = app
        .selected_process
        .and_then(|idx| app.processes.get(idx).map(|p| p.pid))
        .ok_or_else(|| "No process selected".to_string())?;

    let result = std::process::Command::new("kill")
        .arg(format!("-{}", signal))
        .arg(pid.to_string())
        .output()
        .map_err(|e| format!("Failed to kill PID {}: {}", pid, e))?;

    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr);
        Err(format!("Failed to kill PID {}: {}", pid, stderr.trim()))
    } else {
        Ok(())
    }
}

fn next_sort_column(current: SortColumn) -> SortColumn {
    match current {
        SortColumn::Pid => SortColumn::GpuMem,
        SortColumn::GpuMem => SortColumn::SmPercent,
        SortColumn::SmPercent => SortColumn::CpuPercent,
        SortColumn::CpuPercent => SortColumn::Pid,
    }
}

mod app;
mod system;
mod ui;

use std::io;
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use sysinfo::{Disks, Networks, System};

use app::{App, ConfirmDialog};

#[tokio::main]
async fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and system instances
    let mut app = App::new();
    let mut sys = System::new_all();
    let mut disks = Disks::new_with_refreshed_list();
    let mut networks = Networks::new_with_refreshed_list();

    // Initial refresh
    sys.refresh_all();

    let tick_rate = Duration::from_secs(1);
    let mut last_tick = Instant::now();

    loop {
        // Draw UI
        terminal.draw(|f| ui::draw(f, &app))?;

        // Handle events
        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                // Handle Ctrl+C always
                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                    app.should_quit = true;
                } else if app.is_dialog_open() {
                    // Dialog mode key handling
                    match key.code {
                        KeyCode::Tab | KeyCode::Left | KeyCode::Right | KeyCode::Char('h') | KeyCode::Char('l') => {
                            app.toggle_confirm_selection();
                        }
                        KeyCode::Enter => {
                            if let Some(pid) = app.confirm_dialog_action() {
                                system::kill_process(pid);
                                // Force refresh after kill
                                sys.refresh_all();
                                let ports = system::get_open_ports(&sys);
                                app.open_ports = ports;
                            }
                        }
                        KeyCode::Esc | KeyCode::Char('n') => {
                            app.cancel_dialog();
                        }
                        KeyCode::Char('y') => {
                            // Quick confirm with 'y'
                            if let ConfirmDialog::KillProcess { .. } = &app.confirm_dialog {
                                if let Some((_port, _name, pid)) = app.open_ports.get(app.selected_port_idx) {
                                    let pid = *pid;
                                    app.cancel_dialog();
                                    system::kill_process(pid);
                                    sys.refresh_all();
                                    let ports = system::get_open_ports(&sys);
                                    app.open_ports = ports;
                                }
                            }
                        }
                        _ => {}
                    }
                } else {
                    // Normal mode key handling
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Char('Q') => {
                            app.should_quit = true;
                        }
                        KeyCode::Esc => {
                            app.should_quit = true;
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            app.select_next_port();
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            app.select_prev_port();
                        }
                        KeyCode::Enter => {
                            app.request_kill_selected();
                        }
                        _ => {}
                    }
                }
            }
        }

        if app.should_quit {
            break;
        }

        // Update data every tick
        if last_tick.elapsed() >= tick_rate {
            sys.refresh_all();
            disks.refresh(true);
            networks.refresh(true);

            let cpu = system::get_cpu_usage(&sys);
            let memory = system::get_memory(&sys);
            let disk = system::get_disk_io(&disks);
            let network = system::get_network_io(&networks);
            let ports = system::get_open_ports(&sys);

            app.update(cpu, memory, disk, network, ports);

            last_tick = Instant::now();
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

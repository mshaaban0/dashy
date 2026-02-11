use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Gauge, Paragraph, Row, Sparkline, Table},
    Frame,
};

use crate::app::{App, ConfirmDialog};

pub fn draw(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(35),
            Constraint::Percentage(30),
            Constraint::Percentage(35),
        ])
        .split(frame.area());

    // Top row: CPU and Memory
    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[0]);

    draw_cpu_panel(frame, top_chunks[0], app);
    draw_memory_panel(frame, top_chunks[1], app);

    // Middle row: Disk and Network
    let middle_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    draw_disk_panel(frame, middle_chunks[0], app);
    draw_network_panel(frame, middle_chunks[1], app);

    // Bottom row: Ports (full width)
    draw_ports_panel(frame, chunks[2], app);

    // Draw confirmation dialog on top if active
    if let ConfirmDialog::KillProcess { port, process_name, selected_yes } = &app.confirm_dialog {
        draw_confirm_dialog(frame, *port, process_name, *selected_yes);
    }
}

fn draw_cpu_panel(frame: &mut Frame, area: Rect, app: &App) {
    let cpu_data: Vec<u64> = app.cpu_history.iter().map(|&v| v as u64).collect();
    let current_cpu = app.cpu_history.back().copied().unwrap_or(0.0);

    let sparkline = Sparkline::default()
        .block(
            Block::default()
                .title(format!(" CPU: {:.1}% ", current_cpu))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .data(&cpu_data)
        .style(Style::default().fg(Color::Green))
        .max(100);

    frame.render_widget(sparkline, area);
}

fn draw_memory_panel(frame: &mut Frame, area: Rect, app: &App) {
    let used_gb = app.memory_used as f64 / 1_073_741_824.0;
    let total_gb = app.memory_total as f64 / 1_073_741_824.0;
    let ratio = if app.memory_total > 0 {
        (app.memory_used as f64 / app.memory_total as f64).min(1.0)
    } else {
        0.0
    };

    let gauge = Gauge::default()
        .block(
            Block::default()
                .title(" Memory ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .gauge_style(Style::default().fg(Color::Magenta))
        .ratio(ratio)
        .label(format!("{:.1} GB / {:.1} GB", used_gb, total_gb));

    frame.render_widget(gauge, area);
}

fn draw_disk_panel(frame: &mut Frame, area: Rect, app: &App) {
    let read_str = format_bytes(app.disk_read);
    let write_str = format_bytes(app.disk_write);

    let text = vec![
        Line::from(vec![
            Span::styled(" Read:  ", Style::default().fg(Color::Gray)),
            Span::styled(format!("{}/s", read_str), Style::default().fg(Color::Green)),
        ]),
        Line::from(vec![
            Span::styled(" Write: ", Style::default().fg(Color::Gray)),
            Span::styled(format!("{}/s", write_str), Style::default().fg(Color::Red)),
        ]),
    ];

    let paragraph = Paragraph::new(text).block(
        Block::default()
            .title(" Disk I/O ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );

    frame.render_widget(paragraph, area);
}

fn draw_network_panel(frame: &mut Frame, area: Rect, app: &App) {
    let rx_str = format_bytes(app.network_rx);
    let tx_str = format_bytes(app.network_tx);

    let text = vec![
        Line::from(vec![
            Span::styled(" RX: ", Style::default().fg(Color::Gray)),
            Span::styled(format!("{}/s", rx_str), Style::default().fg(Color::Green)),
        ]),
        Line::from(vec![
            Span::styled(" TX: ", Style::default().fg(Color::Gray)),
            Span::styled(format!("{}/s", tx_str), Style::default().fg(Color::Yellow)),
        ]),
    ];

    let paragraph = Paragraph::new(text).block(
        Block::default()
            .title(" Network I/O ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );

    frame.render_widget(paragraph, area);
}

fn draw_ports_panel(frame: &mut Frame, area: Rect, app: &App) {
    let rows: Vec<Row> = app
        .open_ports
        .iter()
        .enumerate()
        .map(|(idx, (port, name, _pid))| {
            let style = if idx == app.selected_port_idx {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else {
                Style::default().fg(Color::White)
            };
            Row::new(vec![
                format!("{}", port),
                name.clone(),
            ])
            .style(style)
        })
        .collect();

    let header = Row::new(vec!["Port", "Process"])
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let widths = [Constraint::Length(10), Constraint::Fill(1)];

    let help_text = if app.open_ports.is_empty() {
        " Open Ports (0) "
    } else {
        " Open Ports - [k/j] navigate, [Enter] kill "
    };

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .title(help_text)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .style(Style::default().fg(Color::White));

    frame.render_widget(table, area);
}

fn draw_confirm_dialog(frame: &mut Frame, port: u16, process_name: &str, selected_yes: bool) {
    let area = frame.area();

    // Center the dialog
    let dialog_width = 50;
    let dialog_height = 7;
    let x = (area.width.saturating_sub(dialog_width)) / 2;
    let y = (area.height.saturating_sub(dialog_height)) / 2;

    let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

    // Clear the area behind the dialog
    frame.render_widget(Clear, dialog_area);

    // Build dialog content
    let yes_style = if selected_yes {
        Style::default().fg(Color::Black).bg(Color::Red).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    let no_style = if !selected_yes {
        Style::default().fg(Color::Black).bg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    let text = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("  Kill process "),
            Span::styled(process_name, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(format!(" on port {}?", port)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("        "),
            Span::styled(" No ", no_style),
            Span::raw("     "),
            Span::styled(" Yes ", yes_style),
        ]),
        Line::from(""),
        Line::from(Span::styled("  [Tab] switch  [Enter] confirm  [Esc] cancel", Style::default().fg(Color::DarkGray))),
    ];

    let paragraph = Paragraph::new(text).block(
        Block::default()
            .title(" Confirm Kill ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red)),
    );

    frame.render_widget(paragraph, dialog_area);
}

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

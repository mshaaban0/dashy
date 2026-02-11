use std::collections::VecDeque;

#[derive(Default)]
pub enum ConfirmDialog {
    #[default]
    None,
    KillProcess {
        port: u16,
        process_name: String,
        selected_yes: bool,
    },
}

pub struct App {
    pub cpu_history: VecDeque<f64>,
    pub memory_used: u64,
    pub memory_total: u64,
    pub disk_read: u64,
    pub disk_write: u64,
    pub network_rx: u64,
    pub network_tx: u64,
    pub open_ports: Vec<(u16, String, u32)>, // (port, process_name, pid)
    pub should_quit: bool,
    pub selected_port_idx: usize,
    pub confirm_dialog: ConfirmDialog,
    // Previous values for delta calculation
    prev_disk_read: u64,
    prev_disk_write: u64,
    prev_network_rx: u64,
    prev_network_tx: u64,
}

impl App {
    pub fn new() -> Self {
        Self {
            cpu_history: VecDeque::with_capacity(60),
            memory_used: 0,
            memory_total: 0,
            disk_read: 0,
            disk_write: 0,
            network_rx: 0,
            network_tx: 0,
            open_ports: Vec::new(),
            should_quit: false,
            selected_port_idx: 0,
            confirm_dialog: ConfirmDialog::None,
            prev_disk_read: 0,
            prev_disk_write: 0,
            prev_network_rx: 0,
            prev_network_tx: 0,
        }
    }

    pub fn update(&mut self, cpu: f64, memory: (u64, u64), disk: (u64, u64), network: (u64, u64), ports: Vec<(u16, String, u32)>) {
        // Update CPU history
        if self.cpu_history.len() >= 60 {
            self.cpu_history.pop_front();
        }
        self.cpu_history.push_back(cpu);

        // Update memory
        self.memory_used = memory.0;
        self.memory_total = memory.1;

        // Calculate disk delta (bytes/sec)
        let (curr_disk_read, curr_disk_write) = disk;
        if self.prev_disk_read > 0 {
            self.disk_read = curr_disk_read.saturating_sub(self.prev_disk_read);
            self.disk_write = curr_disk_write.saturating_sub(self.prev_disk_write);
        }
        self.prev_disk_read = curr_disk_read;
        self.prev_disk_write = curr_disk_write;

        // Calculate network delta (bytes/sec)
        let (curr_rx, curr_tx) = network;
        if self.prev_network_rx > 0 {
            self.network_rx = curr_rx.saturating_sub(self.prev_network_rx);
            self.network_tx = curr_tx.saturating_sub(self.prev_network_tx);
        }
        self.prev_network_rx = curr_rx;
        self.prev_network_tx = curr_tx;

        // Update ports and adjust selection if needed
        self.open_ports = ports;
        if self.selected_port_idx >= self.open_ports.len() && !self.open_ports.is_empty() {
            self.selected_port_idx = self.open_ports.len() - 1;
        }
    }

    pub fn select_next_port(&mut self) {
        if !self.open_ports.is_empty() {
            self.selected_port_idx = (self.selected_port_idx + 1) % self.open_ports.len();
        }
    }

    pub fn select_prev_port(&mut self) {
        if !self.open_ports.is_empty() {
            self.selected_port_idx = self.selected_port_idx
                .checked_sub(1)
                .unwrap_or(self.open_ports.len() - 1);
        }
    }

    pub fn request_kill_selected(&mut self) {
        if let Some((port, name, _pid)) = self.open_ports.get(self.selected_port_idx) {
            self.confirm_dialog = ConfirmDialog::KillProcess {
                port: *port,
                process_name: name.clone(),
                selected_yes: false, // Default to "No" for safety
            };
        }
    }

    pub fn toggle_confirm_selection(&mut self) {
        if let ConfirmDialog::KillProcess { selected_yes, .. } = &mut self.confirm_dialog {
            *selected_yes = !*selected_yes;
        }
    }

    pub fn cancel_dialog(&mut self) {
        self.confirm_dialog = ConfirmDialog::None;
    }

    pub fn confirm_dialog_action(&mut self) -> Option<u32> {
        if let ConfirmDialog::KillProcess { selected_yes, .. } = &self.confirm_dialog {
            if *selected_yes {
                // Get the PID to kill
                if let Some((_port, _name, pid)) = self.open_ports.get(self.selected_port_idx) {
                    let pid = *pid;
                    self.confirm_dialog = ConfirmDialog::None;
                    return Some(pid);
                }
            }
        }
        self.confirm_dialog = ConfirmDialog::None;
        None
    }

    pub fn is_dialog_open(&self) -> bool {
        !matches!(self.confirm_dialog, ConfirmDialog::None)
    }
}

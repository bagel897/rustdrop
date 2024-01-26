use ratatui::{backend::CrosstermBackend, Terminal};
use tokio::io::stdout;

use crate::UiHandle;

#[derive(Debug)]
pub struct TerminalUI {
    terminal: Terminal<CrosstermBackend>,
}
impl TerminalUI {
    pub fn new() -> Self {
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
        Self { terminal }
    }
}
impl UiHandle for TerminalUI {
    fn handle_error(&mut self, t: String) {}
    fn handle_pairing_request(&mut self, request: &crate::PairingRequest) -> bool {}
    fn pick_dest<'a>(&mut self, devices: &'a Vec<crate::Device>) -> Option<&'a crate::Device> {}
}

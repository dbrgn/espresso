//! Shared types.

/// The WiFi mode.
#[derive(Debug)]
pub enum WifiMode {
    /// Station mode (client)
    Station,
    /// Access point mode (server)
    Ap,
    /// Both station and AP mode
    Both,
}

impl WifiMode {
    pub(crate) fn as_at_str(&self) -> &'static str {
        match self {
            WifiMode::Station => "1",
            WifiMode::Ap => "2",
            WifiMode::Both => "3",
        }
    }
}

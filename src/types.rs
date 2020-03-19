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

/// Wraps both the current configuration and the default configuration.
pub struct ConfigWithDefault<T> {
    /// The current configuration.
    pub current: T,
    /// The default configuration, stored in flash memory.
    pub default: T,
}

/// The connection status.
#[derive(Debug, PartialEq)]
pub enum ConnectionStatus {
    /// The ESP8266 Station is connected to an AP and its IP is obtained
    ConnectedToAccessPoint,
    /// The ESP8266 Station has created a TCP or UDP transmission
    InTransmission,
    /// The TCP or UDP transmission of ESP8266 Station is disconnected
    TransmissionEnded,
    /// The ESP8266 Station does NOT connect to an AP
    Disconnected,
    /// Unknown status
    Other(u8),
}

/// The ESP8266 can manage up to five parallel connections with id 0..4.
#[derive(Debug)]
pub enum ConnectionId {
    Zero,
    One,
    Two,
    Three,
    Four,
}

impl ConnectionId {
    pub(crate) fn as_at_str(&self) -> &'static str {
        match self {
            ConnectionId::Zero => "0",
            ConnectionId::One => "1",
            ConnectionId::Two => "2",
            ConnectionId::Three => "3",
            ConnectionId::Four => "4",
        }
    }
}

/// The ESP8266 can either run in single-connection mode (`NonMultiplexed`) or
/// in multi-connection mode (`Multiplexed`).
#[derive(Debug)]
pub enum MultiplexingType {
    NonMultiplexed,
    Multiplexed(ConnectionId),
}

/// The connection protocol.
#[derive(Debug)]
pub enum Protocol {
    Tcp,
    Udp,
}

impl Protocol {
    pub(crate) fn as_at_str(&self) -> &'static str {
        match self {
            Protocol::Tcp => "TCP",
            Protocol::Udp => "UDP",
        }
    }
}

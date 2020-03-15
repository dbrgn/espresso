//! A crate to use ESP8266 WiFi modules over a serial connection.

use atat::AtatClient;
use embedded_hal::serial;
use embedded_hal::timer;
use heapless::String;

pub mod commands;
pub mod types;

use commands::{requests, responses};
use types::ConfigWithDefault;

/// Type alias for a result that may return an ATAT error.
pub type EspResult<T> = Result<T, nb::Error<atat::Error>>;

/// An ESP8266 client.
pub struct EspClient<TX, TIMER>
where
    TX: serial::Write<u8>,
    TIMER: timer::CountDown,
    TIMER::Time: From<u32>,
{
    client: atat::Client<TX, TIMER>,
}

impl<TX, TIMER> EspClient<TX, TIMER>
where
    TX: serial::Write<u8>,
    TIMER: timer::CountDown,
    TIMER::Time: From<u32>,
{
    /// Create a new ESP8266 client.
    ///
    /// Together with the client, an [`IngressManager`][IngressManager] will be
    /// returned. That needs to be hooked up with the incoming serial bytes.
    ///
    /// [IngressManager]: ../atat/istruct.IngressManager.html
    pub fn new(serial_tx: TX, timer: TIMER) -> (Self, atat::IngressManager) {
        let config = atat::Config::new(atat::Mode::Timeout);
        let (client, ingress) = atat::new(serial_tx, timer, config);
        (Self { client }, ingress)
    }

    /// Send a raw command to the device.
    pub fn send_command<T>(&mut self, command: &T) -> EspResult<T::Response>
    where
        T: atat::AtatCmd,
    {
        self.client.send(command)
    }

    /// Test whether the device is connected and able to communicate.
    pub fn selftest(&mut self) -> EspResult<()> {
        self.client
            .send(&requests::At)
            .map(|_: responses::EmptyResponse| ())
    }

    /// Query and return the firmware version.
    pub fn get_firmware_version(&mut self) -> EspResult<responses::FirmwareVersion> {
        self.client.send(&requests::GetFirmwareVersion)
    }

    /// Return the current WiFi mode.
    pub fn get_current_wifi_mode(&mut self) -> EspResult<types::WifiMode> {
        self.client.send(&requests::GetCurrentWifiMode)
    }

    /// Return the default WiFi mode.
    pub fn get_default_wifi_mode(&mut self) -> EspResult<types::WifiMode> {
        self.client.send(&requests::GetDefaultWifiMode)
    }

    /// Return the current and default WiFi mode.
    pub fn get_wifi_mode(&mut self) -> EspResult<ConfigWithDefault<types::WifiMode>> {
        Ok(ConfigWithDefault {
            current: self.client.send(&requests::GetCurrentWifiMode)?,
            default: self.client.send(&requests::GetDefaultWifiMode)?,
        })
    }

    /// Set the WiFi mode.
    pub fn set_wifi_mode(&mut self, mode: types::WifiMode, persist: bool) -> EspResult<()> {
        self.client
            .send(&requests::SetWifiMode::to(mode, persist))
            .map(|_: responses::EmptyResponse| ())
    }

    /// Join the specified access point.
    pub fn join_access_point(
        &mut self,
        ssid: impl Into<String<heapless::consts::U32>>,
        psk: impl Into<String<heapless::consts::U64>>,
        persist: bool,
    ) -> EspResult<responses::JoinResponse> {
        self.client
            .send(&requests::JoinAccessPoint::new(ssid, psk, persist))
    }

    /// Return the current connection status.
    pub fn get_connection_status(&mut self) -> EspResult<types::ConnectionStatus> {
        self.client.send(&requests::GetConnectionStatus)
    }
}

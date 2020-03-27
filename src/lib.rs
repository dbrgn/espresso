//! A crate to use ESP8266 WiFi modules over a serial connection.

use atat::AtatClient;
use core::marker::PhantomData;
use embedded_hal::serial;
use embedded_hal::timer;
use heapless::String;

pub mod commands;
pub mod types;

use commands::{requests, responses};
use types::ConfigWithDefault;

/// Type alias for a result that may return an ATAT error.
pub type EspResult<T> = Result<T, nb::Error<atat::Error>>;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Mode {
    StationMode,
    APMode,
    StationAndAPMode,
}

pub struct UnknownMode {}
pub struct StationMode<MODE> {
    _mode: PhantomData<MODE>,
}

pub struct APConnected<LINK> {
    _mode: PhantomData<LINK>,
}
pub struct APDisconnected {}

pub struct LinkConnected {}
pub struct LinkDisconnected {}

/// Create a new ESP8266 client.
///
/// Together with the client, an [`IngressManager`][IngressManager] will be
/// returned. That needs to be hooked up with the incoming serial bytes.
///
/// [IngressManager]: ../atat/istruct.IngressManager.html
pub fn client<TX, TIMER>(serial_tx: TX, timer: TIMER) -> (EspClient<TX, TIMER, UnknownMode>, atat::IngressManager)
where
    TX: serial::Write<u8>,
    TIMER: timer::CountDown,
    TIMER::Time: From<u32>,
{
    let config = atat::Config::new(atat::Mode::Timeout);
    let (client, ingress) = atat::new(serial_tx, timer, config);
    (EspClient { client, _mode: PhantomData }, ingress)
}


/// An ESP8266 client.
pub struct EspClient<TX, TIMER, MODE>
where
    TX: serial::Write<u8>,
    TIMER: timer::CountDown,
    TIMER::Time: From<u32>,
{
    client: atat::Client<TX, TIMER>,
    _mode: PhantomData<MODE>,
}

impl<TX, TIMER, MODE> EspClient<TX, TIMER, MODE>
where
    TX: serial::Write<u8>,
    TIMER: timer::CountDown,
    TIMER::Time: From<u32>,
{
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

    /// Sets  WiFi mode to station mode.
    pub fn set_station_mode(mut self, persist: bool) -> EspResult<EspClient<TX, TIMER, StationMode<APDisconnected>>> {
        self.client
            .send(&requests::SetWifiMode::to(types::WifiMode::Station, persist))
            .map(|_: responses::EmptyResponse| ())?;

        Ok(EspClient { client: self.client, _mode: PhantomData})
    }

    /// Return the current connection status.
    pub fn get_connection_status(&mut self) -> EspResult<types::ConnectionStatus> {
        self.client.send(&requests::GetConnectionStatus)
    }

    /// Return the locally assigned IP and MAC address.
    pub fn get_local_address(&mut self) -> EspResult<responses::LocalAddress> {
        self.client.send(&requests::GetLocalAddress)
    }
}

impl<TX, TIMER> EspClient<TX, TIMER, StationMode<APDisconnected>>
where
    TX: serial::Write<u8>,
    TIMER: timer::CountDown,
    TIMER::Time: From<u32>,
{
    /// Join the specified access point.
    pub fn join_access_point(
        mut self,
        ssid: impl Into<String<heapless::consts::U32>>,
        psk: impl Into<String<heapless::consts::U64>>,
        persist: bool,
    ) -> EspResult<EspClient<TX, TIMER, StationMode<APConnected<LinkDisconnected>>>> {
        self.client
            .send(&requests::JoinAccessPoint::new(ssid, psk, persist))?;

        Ok(EspClient { client: self.client, _mode: PhantomData })
    }
}

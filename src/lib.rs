//! A crate to use ESP8266 WiFi modules over a serial connection.

#![no_std]

use atat::{AtatClient, ClientBuilder, DefaultDigester, DefaultUrcMatcher, GenericError, Queues};
use embedded_hal::serial;
use embedded_hal::timer;
use heapless::{String, Vec, ArrayLength};

pub mod commands;
pub mod types;

use commands::{requests, responses};
use types::ConfigWithDefault;

/// Type alias for a result that may return an ATAT error.
pub type EspResult<T, E> = Result<T, nb::Error<atat::Error<E>>>;

/// An ESP8266 client.
pub struct EspClient<TX, TIMER, BufLen, UrcCapacity>
where
    TX: serial::Write<u8>,
    TIMER: timer::CountDown,
    TIMER::Time: From<u32>,
    BufLen: ArrayLength<u8>,
    UrcCapacity: ArrayLength<Vec<u8, BufLen>>, 
{
    client: atat::Client<TX, TIMER, BufLen, UrcCapacity>,
}

impl<TX, TIMER, BufLen, UrcCapacity> EspClient<TX, TIMER, BufLen, UrcCapacity>
where
    TX: serial::Write<u8>,
    TIMER: timer::CountDown,
    TIMER::Time: From<u32>,
    BufLen: ArrayLength<u8>,
    UrcCapacity: ArrayLength<Vec<u8, BufLen>>, 
{
    /// Create a new ESP8266 client.
    ///
    /// Together with the client, an [`IngressManager`][IngressManager] will be
    /// returned. That needs to be hooked up with the incoming serial bytes.
    ///
    /// [IngressManager]: ../atat/istruct.IngressManager.html
    pub fn new(serial_tx: TX, timer: TIMER, queues: Queues<BufLen, UrcCapacity>) -> (Self, atat::IngressManager<BufLen, DefaultDigester, DefaultUrcMatcher, UrcCapacity>) 
    where 
        BufLen: ArrayLength<u8>,
        UrcCapacity: ArrayLength<Vec<u8, BufLen>>, 
    {
        let config = atat::Config::new(atat::Mode::Blocking);
        let (client, ingress) = ClientBuilder::new(serial_tx, timer, config).build(queues);
        (Self { client }, ingress)
    }

    /// Send a raw command to the device.
    pub fn send_command<T>(&mut self, command: &T) -> EspResult<T::Response, T::Error>
    where
        T: atat::AtatCmd,
    {
        self.client.send(command)
    }

    /// Test whether the device is connected and able to communicate.
    pub fn selftest(&mut self) -> EspResult<(), GenericError> {
        self.client
            .send(&requests::At)
            .map(|_: responses::EmptyResponse| ())
    }

    /// Query and return the firmware version.
    pub fn get_firmware_version(&mut self) -> EspResult<responses::FirmwareVersion, GenericError> {
        self.client.send(&requests::GetFirmwareVersion)
    }

    /// Return the current WiFi mode.
    pub fn get_current_wifi_mode(&mut self) -> EspResult<types::WifiMode, GenericError> {
        self.client.send(&requests::GetCurrentWifiMode)
    }

    /// Return the default WiFi mode.
    pub fn get_default_wifi_mode(&mut self) -> EspResult<types::WifiMode, GenericError> {
        self.client.send(&requests::GetDefaultWifiMode)
    }

    /// Return the current and default WiFi mode.
    pub fn get_wifi_mode(&mut self) -> EspResult<ConfigWithDefault<types::WifiMode>, GenericError> {
        Ok(ConfigWithDefault {
            current: self.client.send(&requests::GetCurrentWifiMode)?,
            default: self.client.send(&requests::GetDefaultWifiMode)?,
        })
    }

    /// Set the WiFi mode.
    pub fn set_wifi_mode(&mut self, mode: types::WifiMode, persist: bool) -> EspResult<(), GenericError> {
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
    ) -> EspResult<responses::JoinResponse, GenericError> {
        self.client
            .send(&requests::JoinAccessPoint::new(ssid, psk, persist))
    }

    /// Return the current connection status.
    pub fn get_connection_status(&mut self) -> EspResult<types::ConnectionStatus, GenericError> {
        self.client.send(&requests::GetConnectionStatus)
    }

    /// Return the locally assigned IP and MAC address.
    pub fn get_local_address(&mut self) -> EspResult<responses::LocalAddress, GenericError> {
        self.client.send(&requests::GetLocalAddress)
    }
}

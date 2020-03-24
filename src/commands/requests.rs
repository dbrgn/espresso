//! Raw requests that can be sent from the driver to the ESP8266 device.

use atat::AtatCmd;
use heapless::String;
use no_std_net::SocketAddr;
use numtoa::NumToA;

use crate::commands::responses;
use crate::types;

/// An AT test command.
///
/// You will get an [`EmptyResponse`][EmptyResponse] if communication works
/// correctly.
///
/// [EmptyResponse]: ../responses/struct.EmptyResponse.html
#[derive(Debug)]
pub struct At;

impl AtatCmd for At {
    type CommandLen = heapless::consts::U4;
    type Response = responses::EmptyResponse;

    fn as_string(&self) -> String<Self::CommandLen> {
        String::from("AT\r\n")
    }

    fn parse(&self, resp: &str) -> Result<Self::Response, atat::Error> {
        if !resp.trim().is_empty() {
            Err(atat::Error::InvalidResponse)
        } else {
            Ok(responses::EmptyResponse)
        }
    }
}

/// Return information about the firmware version.
#[derive(Debug)]
pub struct GetFirmwareVersion;

impl AtatCmd for GetFirmwareVersion {
    type CommandLen = heapless::consts::U8;
    type Response = responses::FirmwareVersion;

    fn as_string(&self) -> String<Self::CommandLen> {
        String::from("AT+GMR\r\n")
    }

    fn parse(&self, resp: &str) -> Result<Self::Response, atat::Error> {
        let mut lines = resp.lines();

        // AT version (Example: "AT version:1.1.0.0(May 11 2016 18:09:56)")
        let at_version_raw = lines.next().ok_or(atat::Error::ParseString)?;
        if !at_version_raw.starts_with("AT version:") {
            return Err(atat::Error::ParseString);
        }
        let at_version = &at_version_raw[11..];

        // SDK version (example: "SDK version:1.5.4(baaeaebb)")
        let sdk_version_raw = lines.next().ok_or(atat::Error::ParseString)?;
        if !sdk_version_raw.starts_with("SDK version:") {
            return Err(atat::Error::ParseString);
        }
        let sdk_version = &sdk_version_raw[12..];

        // Compile time (example: "compile time:May 20 2016 15:08:19")
        let compile_time_raw = lines.next().ok_or(atat::Error::ParseString)?;
        if !compile_time_raw.starts_with("compile time:") {
            return Err(atat::Error::ParseString);
        }
        let compile_time = &compile_time_raw[13..];

        Ok(responses::FirmwareVersion {
            at_version: String::from(at_version),
            sdk_version: String::from(sdk_version),
            compile_time: String::from(compile_time),
        })
    }
}

/// Restart the module.
#[derive(Debug)]
pub struct Restart;

impl AtatCmd for Restart {
    type CommandLen = heapless::consts::U8;
    type Response = responses::EmptyResponse;

    fn as_string(&self) -> String<Self::CommandLen> {
        String::from("AT+RST\r\n")
    }

    fn parse(&self, resp: &str) -> Result<Self::Response, atat::Error> {
        if !resp.trim().is_empty() {
            Err(atat::Error::InvalidResponse)
        } else {
            Ok(responses::EmptyResponse)
        }
    }
}

/// Query the current WiFi mode.
#[derive(Debug)]
pub struct GetCurrentWifiMode;

impl AtatCmd for GetCurrentWifiMode {
    type CommandLen = heapless::consts::U16;
    type Response = types::WifiMode;

    fn as_string(&self) -> String<Self::CommandLen> {
        String::from("AT+CWMODE_CUR?\r\n")
    }

    fn parse(&self, resp: &str) -> Result<Self::Response, atat::Error> {
        if !resp.starts_with("+CWMODE_CUR:") {
            return Err(atat::Error::InvalidResponse);
        }
        match resp.get(12..13) {
            Some("1") => Ok(types::WifiMode::Station),
            Some("2") => Ok(types::WifiMode::Ap),
            Some("3") => Ok(types::WifiMode::Both),
            _ => Err(atat::Error::InvalidResponse),
        }
    }
}

/// Query the default WiFi mode.
///
/// TODO: Either merge this with `GetCurrentWifiMode`, or use macro to generate.
#[derive(Debug)]
pub struct GetDefaultWifiMode;

impl AtatCmd for GetDefaultWifiMode {
    type CommandLen = heapless::consts::U16;
    type Response = types::WifiMode;

    fn as_string(&self) -> String<Self::CommandLen> {
        String::from("AT+CWMODE_DEF?\r\n")
    }

    fn parse(&self, resp: &str) -> Result<Self::Response, atat::Error> {
        if !resp.starts_with("+CWMODE_DEF:") {
            return Err(atat::Error::InvalidResponse);
        }
        match resp.get(12..13) {
            Some("1") => Ok(types::WifiMode::Station),
            Some("2") => Ok(types::WifiMode::Ap),
            Some("3") => Ok(types::WifiMode::Both),
            _ => Err(atat::Error::InvalidResponse),
        }
    }
}

/// Set the WiFi mode.
///
/// If `persist` is set to `true`, then the configuration will be persisted
/// to flash.
#[derive(Debug)]
pub struct SetWifiMode {
    mode: types::WifiMode,
    persist: bool,
}

impl SetWifiMode {
    pub fn to(mode: types::WifiMode, persist: bool) -> Self {
        Self { mode, persist }
    }
}

impl AtatCmd for SetWifiMode {
    type CommandLen = heapless::consts::U17;
    type Response = responses::EmptyResponse;

    fn as_string(&self) -> String<Self::CommandLen> {
        let mut string = String::from(if self.persist {
            "AT+CWMODE_DEF="
        } else {
            "AT+CWMODE_CUR="
        });
        string.push_str(self.mode.as_at_str()).unwrap();
        string.push_str("\r\n").unwrap();
        string
    }

    fn parse(&self, resp: &str) -> Result<Self::Response, atat::Error> {
        // TODO: This code is used a lot, move it into helper function
        if !resp.trim().is_empty() {
            Err(atat::Error::InvalidResponse)
        } else {
            Ok(responses::EmptyResponse)
        }
    }
}

/// Query available Access Points.
#[derive(Debug)]
pub struct ListAccessPoints;

impl AtatCmd for ListAccessPoints {
    type CommandLen = heapless::consts::U10;
    type Response = responses::EmptyResponse;

    fn as_string(&self) -> String<Self::CommandLen> {
        String::from("AT+CWLAP\r\n")
    }

    fn parse(&self, resp: &str) -> Result<Self::Response, atat::Error> {
        println!("Parse: {:?}", resp);
        // TODO: This currently overflows
        Ok(responses::EmptyResponse)
    }

    fn max_timeout_ms(&self) -> u32 {
        10_000
    }
}

/// Join an Access Point.
///
/// If `persist` is set to `true`, then the credentials will be persisted to
/// flash.
#[derive(Debug)]
pub struct JoinAccessPoint {
    ssid: String<heapless::consts::U32>,
    psk: String<heapless::consts::U64>,
    persist: bool,
}

impl JoinAccessPoint {
    pub fn new(
        ssid: impl Into<String<heapless::consts::U32>>,
        psk: impl Into<String<heapless::consts::U64>>,
        persist: bool,
    ) -> Self {
        Self {
            ssid: ssid.into(),
            psk: psk.into(),
            persist,
        }
    }
}

impl AtatCmd for JoinAccessPoint {
    type CommandLen = heapless::consts::U116;
    type Response = responses::JoinResponse;

    fn as_string(&self) -> String<Self::CommandLen> {
        let mut string = String::from(if self.persist {
            "AT+CWJAP_DEF="
        } else {
            "AT+CWJAP_CUR="
        });
        // TODO: Proper quoting
        string.push('"').unwrap();
        string.push_str(self.ssid.as_str()).unwrap();
        string.push_str("\",\"").unwrap();
        string.push_str(self.psk.as_str()).unwrap();
        string.push('"').unwrap();
        string.push_str("\r\n").unwrap();
        string
    }

    fn parse(&self, resp: &str) -> Result<Self::Response, atat::Error> {
        let mut response = responses::JoinResponse {
            connected: false,
            got_ip: false,
        };
        for line in resp.lines() {
            match line {
                "WIFI DISCONNECTED" => response.connected = false,
                "WIFI CONNECTED" => response.connected = true,
                "WIFI GOT IP" => response.got_ip = true,
                _ => { /* throw away unknown lines for now */ }
            }
        }
        Ok(response)
    }

    fn max_timeout_ms(&self) -> u32 {
        // From experiments, it seems that a timeout is returned after ~15s
        25_000
    }
}

/// Query information about current connection.
#[derive(Debug)]
pub struct GetConnectionStatus;

impl AtatCmd for GetConnectionStatus {
    type CommandLen = heapless::consts::U14;
    type Response = types::ConnectionStatus;

    fn as_string(&self) -> String<Self::CommandLen> {
        String::from("AT+CIPSTATUS\r\n")
    }

    fn parse(&self, resp: &str) -> Result<Self::Response, atat::Error> {
        if !resp.starts_with("STATUS:") {
            return Err(atat::Error::InvalidResponse);
        }
        match resp.get(7..8) {
            Some("2") => Ok(types::ConnectionStatus::ConnectedToAccessPoint),
            Some("3") => Ok(types::ConnectionStatus::InTransmission),
            Some("4") => Ok(types::ConnectionStatus::TransmissionEnded),
            Some("5") => Ok(types::ConnectionStatus::Disconnected),
            Some(other) => Ok(types::ConnectionStatus::Other(
                other.parse().map_err(|_| atat::Error::ParseString)?,
            )),
            None => Err(atat::Error::InvalidResponse),
        }
    }
}

/// Query the local IP and MAC addresses.
#[derive(Debug)]
pub struct GetLocalAddress;

impl AtatCmd for GetLocalAddress {
    type CommandLen = heapless::consts::U10;
    type Response = responses::LocalAddress;

    fn as_string(&self) -> String<Self::CommandLen> {
        String::from("AT+CIFSR\r\n")
    }

    fn parse(&self, resp: &str) -> Result<Self::Response, atat::Error> {
        // Example: +CIFSR:STAIP,"10.0.99.164"\r\n+CIFSR:STAMAC,"dc:4f:22:7e:41:b4"
        let mut mac = None;
        let mut ip = None;
        for line in resp.lines() {
            if line.starts_with("+CIFSR:STAIP,") {
                let ip_raw = &line[14..line.len() - 1];
                ip = if ip_raw == "0.0.0.0" {
                    None
                } else {
                    Some(ip_raw.parse().map_err(|_| atat::Error::ParseString)?)
                };
            } else if line.starts_with("+CIFSR:STAMAC,") {
                mac = Some(String::from(&line[15..32]));
            }
        }
        Ok(responses::LocalAddress {
            ip,
            mac: mac.ok_or(atat::Error::ParseString)?,
        })
    }
}

/// Establish TCP Connection, UDP Transmission or SSL Connection.
#[derive(Debug)]
pub struct EstablishConnection {
    mux: types::MultiplexingType,
    protocol: types::Protocol,
    remote_addr: SocketAddr,
}

impl EstablishConnection {
    pub fn tcp(mux: types::MultiplexingType, remote_addr: SocketAddr) -> Self {
        Self {
            mux,
            protocol: types::Protocol::Tcp,
            remote_addr,
        }
    }

    pub fn udp(mux: types::MultiplexingType, remote_addr: SocketAddr) -> Self {
        Self {
            mux,
            protocol: types::Protocol::Udp,
            remote_addr,
        }
    }
}

impl AtatCmd for EstablishConnection {
    type CommandLen = heapless::consts::U42;
    type Response = responses::EmptyResponse;

    fn as_string(&self) -> String<Self::CommandLen> {
        // Single: AT+CIPSTART=<type>,<remote IP>,<remote port>[,<TCP keep alive>]
        // Multiple: AT+CIPSTART=<link ID>,<type>,<remote IP>,<remote port>[,<TCP keep alive>]
        let mut string = String::from("AT+CIPSTART=");
        match self.mux {
            types::MultiplexingType::NonMultiplexed => {}
            types::MultiplexingType::Multiplexed(ref id) => {
                string.push_str(id.as_at_str()).unwrap();
                string.push(',').unwrap();
            }
        }
        string.push('"').unwrap();
        string.push_str(self.protocol.as_at_str()).unwrap();
        string.push('"').unwrap();
        string.push(',').unwrap();
        match self.remote_addr {
            SocketAddr::V4(addr) => {
                let octets = addr.ip().octets();
                let mut buf = [0; 5];
                string.push('"').unwrap();
                for (i, octet) in octets.iter().enumerate() {
                    string.push_str(octet.numtoa_str(10, &mut buf)).unwrap();
                    if i != 3 {
                        string.push('.').unwrap();
                    }
                }
                string.push('"').unwrap();
                string.push(',').unwrap();
                string
                    .push_str(addr.port().numtoa_str(10, &mut buf))
                    .unwrap();
            }
            SocketAddr::V6(_addr) => {
                unimplemented!("IPv6 support is not implemented");
            }
        }
        string.push_str("\r\n").unwrap();
        string
    }

    fn parse(&self, _resp: &str) -> Result<Self::Response, atat::Error> {
        Ok(responses::EmptyResponse)
    }

    fn max_timeout_ms(&self) -> u32 {
        30_000
    }
}

/// Prepare to send `length` bytes of data.
///
/// This message MUST be followed by a `SendData` message.
#[derive(Debug)]
pub struct PrepareSendData {
    mux: types::MultiplexingType,
    length: u16,
}

impl PrepareSendData {
    pub fn new(mux: types::MultiplexingType, length: u16) -> Self {
        Self { mux, length }
    }
}

impl AtatCmd for PrepareSendData {
    type CommandLen = heapless::consts::U20;
    type Response = responses::EmptyResponse;

    fn as_string(&self) -> String<Self::CommandLen> {
        let mut string = String::from("AT+CIPSEND=");
        match self.mux {
            types::MultiplexingType::NonMultiplexed => {}
            types::MultiplexingType::Multiplexed(ref id) => {
                string.push_str(id.as_at_str()).unwrap();
                string.push(',').unwrap();
            }
        }
        {
            // Length can only be in the range 0-65535
            let mut buf = [0; 5];
            string
                .push_str(self.length.numtoa_str(10, &mut buf))
                .unwrap();
        }
        string.push_str("\r\n").unwrap();
        string
    }

    fn parse(&self, _resp: &str) -> Result<Self::Response, atat::Error> {
        Ok(responses::EmptyResponse)
    }

    fn max_timeout_ms(&self) -> u32 {
        5_000
    }
}

/// Send data.
///
/// This message MUST directly follow by a `PrepareSendData` message.
///
/// The type argument `L` must be at least as large as the data length.
#[derive(Debug)]
pub struct SendData<'a, L> {
    data: &'a str,
    length: core::marker::PhantomData<L>,
}

impl<'a, L> SendData<'a, L>
where
    L: heapless::ArrayLength<u8>,
{
    pub fn new(data: &'a str) -> Self {
        Self {
            data,
            length: core::marker::PhantomData,
        }
    }
}

impl<'a, L> AtatCmd for SendData<'a, L>
where
    L: heapless::ArrayLength<u8>,
{
    type CommandLen = L;
    type Response = responses::EmptyResponse;

    fn as_string(&self) -> String<Self::CommandLen> {
        String::from(self.data)
    }

    fn parse(&self, resp: &str) -> Result<Self::Response, atat::Error> {
        println!("Parse: {:?}", resp);
        Ok(responses::EmptyResponse)
    }

    fn max_timeout_ms(&self) -> u32 {
        30_000
    }
}

/// Close the TCP/UDP/SSL Connection.
#[derive(Debug)]
pub struct CloseConnection {
    mux: types::MultiplexingType,
}

impl CloseConnection {
    pub fn new(mux: types::MultiplexingType) -> Self {
        Self { mux }
    }
}

impl AtatCmd for CloseConnection {
    type CommandLen = heapless::consts::U15;
    type Response = responses::EmptyResponse;

    fn as_string(&self) -> String<Self::CommandLen> {
        let mut string = String::from("AT+CIPCLOSE");
        match self.mux {
            types::MultiplexingType::NonMultiplexed => {}
            types::MultiplexingType::Multiplexed(ref id) => {
                // TODO: Send connection id "5" to close all connections
                string.push('=').unwrap();
                string.push_str(id.as_at_str()).unwrap();
            }
        }
        string.push_str("\r\n").unwrap();
        string
    }

    fn parse(&self, _resp: &str) -> Result<Self::Response, atat::Error> {
        Ok(responses::EmptyResponse)
    }

    fn max_timeout_ms(&self) -> u32 {
        5_000
    }
}

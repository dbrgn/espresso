//! Raw requests that can be sent from the driver to the ESP8266 device.

use core::fmt::Write;

use atat::{AtatCmd, Error, GenericError, InternalError};
use heapless::{String, Vec};
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

impl AtatCmd<4> for At {
    type Response = responses::EmptyResponse;
    type Error = GenericError;

    fn as_bytes(&self) -> Vec<u8, 4> {
        Vec::from_slice(b"AT\r\n").unwrap()
    }

    fn parse(
        &self,
        resp: Result<&[u8], &InternalError>,
    ) -> Result<Self::Response, Error<Self::Error>> {
        let resp = core::str::from_utf8(resp?).unwrap();
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

impl AtatCmd<8> for GetFirmwareVersion {
    type Response = responses::FirmwareVersion;
    type Error = GenericError;

    fn as_bytes(&self) -> Vec<u8, 8> {
        Vec::from_slice(b"AT+GMR\r\n").unwrap()
    }

    fn parse(&self, resp: Result<&[u8], &InternalError>) -> Result<Self::Response, atat::Error> {
        let resp = core::str::from_utf8(resp?).unwrap();
        let mut lines = resp.lines();

        // AT version (Example: "AT version:1.1.0.0(May 11 2016 18:09:56)")
        let at_version_raw = lines.next().ok_or(atat::Error::Parse)?;
        if !at_version_raw.starts_with("AT version:") {
            return Err(atat::Error::Parse);
        }
        let at_version = &at_version_raw[11..];

        // SDK version (example: "SDK version:1.5.4(baaeaebb)")
        let sdk_version_raw = lines.next().ok_or(atat::Error::Parse)?;
        if !sdk_version_raw.starts_with("SDK version:") {
            return Err(atat::Error::Parse);
        }
        let sdk_version = &sdk_version_raw[12..];

        // Compile time (example: "compile time:May 20 2016 15:08:19")
        let compile_time_raw = lines.next().ok_or(atat::Error::Parse)?;
        if !compile_time_raw.starts_with("compile time:") {
            return Err(atat::Error::Parse);
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

impl AtatCmd<8> for Restart {
    type Response = responses::EmptyResponse;
    type Error = GenericError;

    fn as_bytes(&self) -> Vec<u8, 8> {
        Vec::from_slice(b"AT+RST\r\n").unwrap()
    }

    fn parse(&self, resp: Result<&[u8], &InternalError>) -> Result<Self::Response, atat::Error> {
        let resp = core::str::from_utf8(resp?).unwrap();
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

impl AtatCmd<16> for GetCurrentWifiMode {
    type Response = types::WifiMode;
    type Error = GenericError;

    fn as_bytes(&self) -> Vec<u8, 16> {
        Vec::from_slice(b"AT+CWMODE_CUR?\r\n").unwrap()
    }

    fn parse(&self, resp: Result<&[u8], &InternalError>) -> Result<Self::Response, atat::Error> {
        let resp = core::str::from_utf8(resp?).unwrap();
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

impl AtatCmd<16> for GetDefaultWifiMode {
    type Response = types::WifiMode;
    type Error = GenericError;

    fn as_bytes(&self) -> Vec<u8, 16> {
        Vec::from_slice(b"AT+CWMODE_DEF?\r\n").unwrap()
    }

    fn parse(&self, resp: Result<&[u8], &InternalError>) -> Result<Self::Response, atat::Error> {
        let resp = core::str::from_utf8(resp?).unwrap();
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

impl AtatCmd<17> for SetWifiMode {
    type Response = responses::EmptyResponse;
    type Error = GenericError;

    fn as_bytes(&self) -> Vec<u8, 17> {
        let mut buf: Vec<u8, 17> = Vec::new();
        let persist_str = if self.persist { "DEF" } else { "CUR" };
        write!(
            buf,
            "AT+CWMODE_{}={}\r\n",
            persist_str,
            self.mode.as_at_str()
        )
        .unwrap();
        buf
    }

    fn parse(&self, resp: Result<&[u8], &InternalError>) -> Result<Self::Response, atat::Error> {
        let resp = core::str::from_utf8(resp?).unwrap();
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

impl AtatCmd<10> for ListAccessPoints {
    type Response = responses::EmptyResponse;
    type Error = GenericError;
    const MAX_TIMEOUT_MS: u32 = 10_000;

    fn as_bytes(&self) -> Vec<u8, 10> {
        Vec::from_slice(b"AT+CWLAP\r\n").unwrap()
    }

    fn parse(&self, _resp: Result<&[u8], &InternalError>) -> Result<Self::Response, atat::Error> {
        // println!("Parse: {:?}", resp);
        // TODO: This currently overflows
        Ok(responses::EmptyResponse)
    }
}

/// Join an Access Point.
///
/// If `persist` is set to `true`, then the credentials will be persisted to
/// flash.
#[derive(Debug)]
pub struct JoinAccessPoint {
    ssid: String<32>,
    psk: String<64>,
    persist: bool,
}

impl JoinAccessPoint {
    pub fn new(ssid: impl Into<String<32>>, psk: impl Into<String<64>>, persist: bool) -> Self {
        Self {
            ssid: ssid.into(),
            psk: psk.into(),
            persist,
        }
    }
}

impl AtatCmd<116> for JoinAccessPoint {
    type Response = responses::JoinResponse;
    type Error = GenericError;
    const MAX_TIMEOUT_MS: u32 = 25_000;

    fn as_bytes(&self) -> Vec<u8, 116> {
        let mut buf: Vec<u8, 116> = Vec::new();
        let persist_str = if self.persist { "DEF" } else { "CUR" };
        // TODO: Proper quoting
        write!(
            buf,
            "AT+CWJAP_{}=\"{}\",\"{}\"\r\n",
            persist_str,
            self.ssid.as_str(),
            self.psk.as_str()
        )
        .unwrap();
        buf
    }

    fn parse(&self, resp: Result<&[u8], &InternalError>) -> Result<Self::Response, atat::Error> {
        let resp = core::str::from_utf8(resp?).unwrap();
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
}

/// Query information about current connection.
#[derive(Debug)]
pub struct GetConnectionStatus;

impl AtatCmd<14> for GetConnectionStatus {
    type Response = types::ConnectionStatus;
    type Error = GenericError;

    fn as_bytes(&self) -> Vec<u8, 14> {
        Vec::from_slice(b"AT+CIPSTATUS\r\n").unwrap()
    }

    fn parse(&self, resp: Result<&[u8], &InternalError>) -> Result<Self::Response, atat::Error> {
        let resp = core::str::from_utf8(resp?).unwrap();
        if !resp.starts_with("STATUS:") {
            return Err(atat::Error::InvalidResponse);
        }
        match resp.get(7..8) {
            Some("2") => Ok(types::ConnectionStatus::ConnectedToAccessPoint),
            Some("3") => Ok(types::ConnectionStatus::InTransmission),
            Some("4") => Ok(types::ConnectionStatus::TransmissionEnded),
            Some("5") => Ok(types::ConnectionStatus::Disconnected),
            Some(other) => Ok(types::ConnectionStatus::Other(
                other.parse().map_err(|_| atat::Error::Parse)?,
            )),
            None => Err(atat::Error::InvalidResponse),
        }
    }
}

/// Query the local IP and MAC addresses.
#[derive(Debug)]
pub struct GetLocalAddress;

impl AtatCmd<10> for GetLocalAddress {
    type Response = responses::LocalAddress;
    type Error = GenericError;

    fn as_bytes(&self) -> Vec<u8, 10> {
        Vec::from_slice(b"AT+CIFSR\r\n").unwrap()
    }

    fn parse(&self, resp: Result<&[u8], &InternalError>) -> Result<Self::Response, atat::Error> {
        let resp = core::str::from_utf8(resp?).unwrap();
        // Example: +CIFSR:STAIP,"10.0.99.164"\r\n+CIFSR:STAMAC,"dc:4f:22:7e:41:b4"
        let mut mac = None;
        let mut ip = None;
        for line in resp.lines() {
            if line.starts_with("+CIFSR:STAIP,") {
                let ip_raw = &line[14..line.len() - 1];
                ip = if ip_raw == "0.0.0.0" {
                    None
                } else {
                    Some(ip_raw.parse().map_err(|_| atat::Error::Parse)?)
                };
            } else if line.starts_with("+CIFSR:STAMAC,") {
                mac = Some(String::from(&line[15..32]));
            }
        }
        Ok(responses::LocalAddress {
            ip,
            mac: mac.ok_or(atat::Error::Parse)?,
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

impl AtatCmd<42> for EstablishConnection {
    type Response = responses::EmptyResponse;
    type Error = GenericError;
    const MAX_TIMEOUT_MS: u32 = 30_000;

    fn as_bytes(&self) -> Vec<u8, 42> {
        // Single: AT+CIPSTART=<type>,<remote IP>,<remote port>[,<TCP keep alive>]
        // Multiple: AT+CIPSTART=<link ID>,<type>,<remote IP>,<remote port>[,<TCP keep alive>]
        let mut buf: Vec<u8, 42> = Vec::new();
        write!(buf, "AT+CIPSTART=").unwrap();
        if let types::MultiplexingType::Multiplexed(ref id) = self.mux {
            write!(buf, "{},", id.as_at_str()).unwrap();
        }
        write!(buf, "\"{}\",", self.protocol.as_at_str()).unwrap();
        match self.remote_addr {
            SocketAddr::V4(addr) => {
                let octets = addr.ip().octets();
                let mut num_buf = [0; 5];
                write!(buf, "\"").unwrap();
                for (i, octet) in octets.iter().enumerate() {
                    write!(buf, "{}", octet.numtoa_str(10, &mut num_buf)).unwrap();
                    if i != 3 {
                        write!(buf, ".").unwrap();
                    }
                }
                write!(buf, "\",{}", addr.port().numtoa_str(10, &mut num_buf)).unwrap();
            }
            SocketAddr::V6(_addr) => {
                unimplemented!("IPv6 support is not implemented");
            }
        }
        write!(buf, "\r\n").unwrap();
        buf
    }

    fn parse(&self, _resp: Result<&[u8], &InternalError>) -> Result<Self::Response, atat::Error> {
        Ok(responses::EmptyResponse)
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

impl AtatCmd<20> for PrepareSendData {
    type Response = responses::EmptyResponse;
    type Error = GenericError;
    const MAX_TIMEOUT_MS: u32 = 5_000;

    fn as_bytes(&self) -> Vec<u8, 20> {
        let mut buf: Vec<u8, 20> = Vec::new();
        write!(buf, "AT+CIPSEND=").unwrap();
        if let types::MultiplexingType::Multiplexed(ref id) = self.mux {
            write!(buf, "{},", id.as_at_str()).unwrap();
        }
        {
            // Length can only be in the range 0-65535
            let mut num_buf = [0; 5];
            write!(buf, "{}\r\n", self.length.numtoa_str(10, &mut num_buf)).unwrap();
        }
        buf
    }

    fn parse(&self, _resp: Result<&[u8], &InternalError>) -> Result<Self::Response, atat::Error> {
        Ok(responses::EmptyResponse)
    }
}

/// Send data.
///
/// This message MUST directly follow by a `PrepareSendData` message.
///
/// The type argument `L` must be at least as large as the data length.
#[derive(Debug)]
pub struct SendData<'a, const L: usize> {
    data: &'a str,
}

impl<'a, const L: usize> SendData<'a, L> {
    pub fn new(data: &'a str) -> Self {
        Self { data }
    }
}

impl<'a, const L: usize> AtatCmd<L> for SendData<'a, L> {
    type Response = responses::EmptyResponse;
    type Error = GenericError;
    const MAX_TIMEOUT_MS: u32 = 30_000;

    fn as_bytes(&self) -> Vec<u8, L> {
        Vec::from_slice(self.data.as_bytes()).unwrap()
    }

    fn parse(&self, _resp: Result<&[u8], &InternalError>) -> Result<Self::Response, atat::Error> {
        // println!("Parse: {:?}", resp);
        Ok(responses::EmptyResponse)
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

impl AtatCmd<15> for CloseConnection {
    type Response = responses::EmptyResponse;
    type Error = GenericError;
    const MAX_TIMEOUT_MS: u32 = 5_000;

    fn as_bytes(&self) -> Vec<u8, 15> {
        let mut buf: Vec<u8, 15> = Vec::new();
        write!(buf, "AT+CIPCLOSE").unwrap();
        if let types::MultiplexingType::Multiplexed(ref id) = self.mux {
            write!(buf, "={}", id.as_at_str()).unwrap();
        }
        write!(buf, "\r\n").unwrap();
        buf
    }

    fn parse(&self, _resp: Result<&[u8], &InternalError>) -> Result<Self::Response, atat::Error> {
        Ok(responses::EmptyResponse)
    }
}

//! Requests that can be sent from the driver to the ESP8266 device.

use atat::AtatCmd;
use heapless::String;

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
/// If `persistent` is set to `true`, then the configuration will be persisted
/// to flash.
#[derive(Debug)]
pub struct SetWifiMode {
    mode: types::WifiMode,
    persistent: bool,
}

impl SetWifiMode {
    pub fn to(mode: types::WifiMode, persistent: bool) -> Self {
        Self{ mode, persistent }
    }
}

impl AtatCmd for SetWifiMode {
    type CommandLen = heapless::consts::U17;
    type Response = responses::EmptyResponse;

    fn as_string(&self) -> String<Self::CommandLen> {
        let mut string = String::from(match self.persistent {
            true => "AT+CWMODE_DEF=",
            false => "AT+CWMODE_CUR=",
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

/// Query available APs.
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

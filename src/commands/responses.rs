//! Responses from the ESP8266 device.

use atat::{AtatResp, Error, GenericError, InternalError};
use heapless::String;
use no_std_net::Ipv4Addr;

use crate::types;

/// An empty response, no body.
#[derive(Debug)]
pub struct EmptyResponse;

impl AtatResp for EmptyResponse {}

impl EmptyResponse {
    pub(crate) fn from_resp(
        resp: Result<&[u8], InternalError>,
    ) -> Result<Self, Error<GenericError>> {
        let bytes = resp?;
        if !bytes.is_empty() {
            Err(atat::Error::InvalidResponse)
        } else {
            Ok(Self)
        }
    }
}

/// Firmware version.
#[derive(Debug)]
pub struct FirmwareVersion {
    pub at_version: String<32>,
    pub sdk_version: String<32>,
    pub compile_time: String<32>,
}

impl AtatResp for FirmwareVersion {}

/// Generic string response.
#[derive(Debug)]
pub struct StringResponse<const L: usize>(pub(crate) String<L>);

impl<const L: usize> AtatResp for StringResponse<L> {}

impl AtatResp for types::WifiMode {}

/// AP join result.
#[derive(Debug)]
pub struct JoinResponse {
    pub connected: bool,
    pub got_ip: bool,
}

impl AtatResp for JoinResponse {}

impl AtatResp for types::ConnectionStatus {}

#[derive(Debug)]
pub struct LocalAddress {
    pub ip: Option<Ipv4Addr>,
    pub mac: String<17>,
}

impl AtatResp for LocalAddress {}

#[derive(Debug, PartialEq)]
pub enum ConnectResponse {
    /// The connection was opened
    Connected,
    /// The connection is already open
    AlreadyConnected,
}

impl AtatResp for ConnectResponse {}

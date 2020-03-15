//! Responses from the ESP8266 device.

use atat::AtatResp;
use heapless::consts::U32;
use heapless::String;

use crate::types;

/// An empty response, no body.
#[derive(Debug)]
pub struct EmptyResponse;

impl AtatResp for EmptyResponse {}

/// Firmware version.
#[derive(Debug)]
pub struct FirmwareVersion {
    pub at_version: String<U32>,
    pub sdk_version: String<U32>,
    pub compile_time: String<U32>,
}

impl AtatResp for FirmwareVersion {}

/// Generic string response.
#[derive(Debug)]
pub struct StringResponse<L: heapless::ArrayLength<u8>>(pub(crate) String<L>);

impl<L: heapless::ArrayLength<u8>> AtatResp for StringResponse<L> {}

impl AtatResp for types::WifiMode {}
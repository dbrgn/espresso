use atat::{ATATCmd, ATATResp};
use heapless::String;

pub struct At;

impl ATATCmd for At {
    type CommandLen = heapless::consts::U4;
    type Response = EmptyResponse;

    fn as_str(&self) -> String<Self::CommandLen> {
        String::from("AT\r\n")
    }

    fn parse(&self, resp: &str) -> Result<Self::Response, atat::Error> {
        println!("Parsing: {}", resp);
        Ok(EmptyResponse)
    }
}

#[derive(Debug)]
pub struct EmptyResponse;

impl ATATResp for EmptyResponse { }

pub struct GetFirmwareVersion;

impl ATATCmd for GetFirmwareVersion {
    type CommandLen = heapless::consts::U8;
    type Response = FirmwareVersion;

    fn as_str(&self) -> String<Self::CommandLen> {
        String::from("AT+GMR\r\n")
    }

    fn parse(&self, resp: &str) -> Result<Self::Response, atat::Error> {
        println!("Parsing: {}", resp);
        Ok(FirmwareVersion(String::from(resp)))
    }
}

#[derive(Debug)]
pub struct FirmwareVersion(heapless::String<heapless::consts::U256>);

impl ATATResp for FirmwareVersion { }

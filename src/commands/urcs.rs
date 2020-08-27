//! Raw URCs (Unsolicied Result Codes) that can be received from the ESP8266 device.

use core::convert::TryFrom;
use core::iter::FromIterator;

use atat::{AtatUrc, Error};
use heapless::{consts, Vec};

use crate::types::ConnectionId;

#[derive(Debug)]
pub enum EspUrc {
    /// Incoming data from the network
    NetworkData(NetworkData),
    Other(Vec<u8, consts::U2048>),
}

/// Incoming data from the network (+IPD).
#[derive(Debug)]
pub struct NetworkData {
    /// The connection ID. Only set in multiplexed mode.
    pub connection_id: Option<ConnectionId>,

    /// The incoming bytes.
    pub data: Vec<u8, consts::U2048>,
}

impl NetworkData {
    const PREFIX: &'static str = "+IPD,";

    fn from_urc(urc: &str) -> Result<Self, Error> {
        if !urc.starts_with(Self::PREFIX) {
            return Err(Error::ParseString);
        }
        let urc = urc.trim_start_matches(Self::PREFIX);
        let (params, data) = match urc.find(':') {
            Some(index) => urc.split_at(index),
            None => return Err(Error::ParseString),
        };
        let connection_id = match params.bytes().filter(|b| *b == b',').count() {
            0 | 2 => {
                // Single connection, non multiplexed
                // TODO: Parse IP / Port (and test parsing)
                None
            }
            1 | 3 => {
                // Multiplexed connection
                // TODO: Parse IP / Port (and test parsing)
                let connection_id_raw = params.split(',').next().unwrap();
                Some(ConnectionId::try_from(connection_id_raw)?)
            }
            _ => return Err(Error::ParseString),
        };
        Ok(Self {
            connection_id,
            data: Vec::from_iter(data.bytes()),
        })
    }
}

impl AtatUrc for EspUrc {
    type Response = Self;

    fn parse(urc: &str) -> Result<Self::Response, Error> {
        if urc.starts_with(NetworkData::PREFIX) {
            Ok(Self::NetworkData(NetworkData::from_urc(urc)?))
        } else {
            Ok(Self::Other(Vec::from_iter(urc.bytes())))
        }
    }
}

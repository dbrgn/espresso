//! A crate to use ESP8266 WiFi modules over a serial connection.

pub mod commands;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

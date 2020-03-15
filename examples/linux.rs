use std::env;
use std::io;
use std::thread;
use std::time::Duration;

use atat::AtatClient;
use serialport::{DataBits, FlowControl, Parity, SerialPortSettings, StopBits};

use espresso::{commands::requests, types::WifiMode};

fn main() {
    env_logger::init();

    // Parse args
    let args: Vec<String> = env::args().collect();
    if args.len() != 5 {
        println!("Usage: {} <path-to-serial> <baudrate> <ssid> <psk>", args[0]);
        println!("Example: {} /dev/ttyUSB0 115200 mywifi hellopasswd123", args[0]);
        std::process::exit(1);
    }
    let dev = &args[1];
    let baud_rate: u32 = args[2].parse().unwrap();
    let ssid = &args[3];
    let psk = &args[4];

    println!("Starting (dev={}, baud={:?})…", dev, baud_rate);

    // Serial port settings
    let settings = SerialPortSettings {
        baud_rate,
        data_bits: DataBits::Eight,
        flow_control: FlowControl::None,
        parity: Parity::None,
        stop_bits: StopBits::One,
        timeout: Duration::from_millis(5000),
    };

    // Open serial port
    let serial_tx =
        serialport::open_with_settings(dev, &settings).expect("Could not open serial port");
    let mut serial_rx = serial_tx.try_clone().expect("Could not clone serial port");

    // Initialize
    let timer = timer::SysTimer::new();
    let config = atat::Config::new(atat::Mode::Timeout);
    let (mut client, mut ingress) = atat::new(serial_tx, timer, config);

    // Launch reading thread
    thread::Builder::new()
        .name("serial_read".to_string())
        .spawn(move || loop {
            let mut buffer = [0; 32];
            match serial_rx.read(&mut buffer[..]) {
                Ok(0) => {}
                Ok(bytes_read) => {
                    ingress.write(&buffer[0..bytes_read]);
                    ingress.digest();
                    ingress.digest();
                }
                Err(e) => match e.kind() {
                    io::ErrorKind::WouldBlock
                    | io::ErrorKind::TimedOut
                    | io::ErrorKind::Interrupted => {
                        // Ignore
                    }
                    _ => {
                        log::error!("Serial reading thread error while reading: {}", e);
                    }
                },
            }
        })
        .unwrap();

    // Reset module
    print!("Testing whether device is online… ");
    client
        .send(&requests::At)
        .expect("Could not send AT command");
    println!("OK");

    // Get firmware information
    let version = client
        .send(&requests::GetFirmwareVersion)
        .expect("Could not get firmware version");
    println!("{:?}", version);

    // Show current config
    println!(
        "Wifi mode:\n  Current: {:?}\n  Default: {:?}",
        client
            .send(&requests::GetCurrentWifiMode)
            .expect("Could not get current wifi mode"),
        client
            .send(&requests::GetDefaultWifiMode)
            .expect("Could not get default wifi mode"),
    );

    println!();
    print!("Setting current Wifi mode to Station… ");
    client
        .send(&requests::SetWifiMode::to(WifiMode::Station, false))
        .expect("Could not set current wifi mode");
    println!("OK");

    //println!();
    //println!("Available APs:");
    //client
    //    .send(&requests::ListAccessPoints)
    //    .expect("Could not set current wifi mode");

    println!();
    println!("Connect to access point with SSID {:?}…", ssid);
    let result = client
        .send(&requests::JoinAccessPoint::new(ssid.as_str(), psk.as_str(), false))
        .expect("Could not connect to access point");
    println!("{:?}", result);

    loop {}
}

mod timer {
    use std::time::{Duration, Instant};

    use embedded_hal::timer::{CountDown, Periodic};

    /// A timer with milliseconds as unit of time.
    pub struct SysTimer {
        start: Instant,
        duration_ms: u32,
    }

    impl SysTimer {
        pub fn new() -> SysTimer {
            SysTimer {
                start: Instant::now(),
                duration_ms: 0,
            }
        }
    }

    impl CountDown for SysTimer {
        type Time = u32;

        fn start<T>(&mut self, count: T)
        where
            T: Into<Self::Time>,
        {
            self.start = Instant::now();
            self.duration_ms = count.into();
        }

        fn wait(&mut self) -> nb::Result<(), void::Void> {
            if (Instant::now() - self.start) > Duration::from_millis(self.duration_ms as u64) {
                // Restart the timer to fulfil the contract by `Periodic`
                self.start = Instant::now();
                Ok(())
            } else {
                Err(nb::Error::WouldBlock)
            }
        }
    }

    impl Periodic for SysTimer {}

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_delay() {
            let mut timer = SysTimer::new();
            let before = Instant::now();
            timer.start(500u32);
            nb::block!(timer.wait()).unwrap();
            let after = Instant::now();
            let duration_ms = (after - before).as_millis();
            assert!(duration_ms >= 500);
            assert!(duration_ms < 1000);
        }
    }
}

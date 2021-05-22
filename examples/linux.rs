use std::convert::TryInto;
use std::env;
use std::io;
use std::thread;
use std::time::Duration;

use atat::{ComQueue, Queues, ResQueue, UrcQueue};
use espresso::commands::requests;
use espresso::types::{ConnectionStatus, MultiplexingType, WifiMode};
use heapless::spsc::Queue;
use no_std_net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use serialport::{DataBits, FlowControl, Parity, SerialPortSettings, StopBits};

fn main() {
    env_logger::init();

    // Parse args
    let args: Vec<String> = env::args().collect();
    if args.len() != 5 {
        println!(
            "Usage: {} <path-to-serial> <baudrate> <ssid> <psk>",
            args[0]
        );
        println!(
            "Example: {} /dev/ttyUSB0 115200 mywifi hellopasswd123",
            args[0]
        );
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
    static mut RES_QUEUE: ResQueue<256> = Queue::new();
    static mut URC_QUEUE: UrcQueue<256, 10> = Queue::new();
    static mut COM_QUEUE: ComQueue = Queue::new();

    let queues = Queues {
        res_queue: unsafe { RES_QUEUE.split() },
        urc_queue: unsafe { URC_QUEUE.split() },
        com_queue: unsafe { COM_QUEUE.split() },
    };

    let timer = timer::SysTimer::new();
    let (mut client, mut ingress) = espresso::EspClient::new(serial_tx, timer, queues);

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

    print!("Testing whether device is online… ");
    client.selftest().expect("Self test failed");
    println!("OK");

    // Get firmware information
    let version = client
        .get_firmware_version()
        .expect("Could not get firmware version");
    println!("{:?}", version);

    // Show current config
    let wifi_mode = client.get_wifi_mode().expect("Could not get wifi mode");
    println!(
        "Wifi mode:\n  Current: {:?}\n  Default: {:?}",
        wifi_mode.current, wifi_mode.default,
    );

    println!();
    print!("Setting current Wifi mode to Station… ");
    client
        .set_wifi_mode(WifiMode::Station, false)
        .expect("Could not set current wifi mode");
    println!("OK");

    println!();
    let status = client
        .get_connection_status()
        .expect("Could not get connection status");
    println!("Connection status: {:?}", status);
    let local_addr = client
        .get_local_address()
        .expect("Could not get local address");
    println!("Local MAC: {}", local_addr.mac);
    println!("Local IP:  {:?}", local_addr.ip);

    match status {
        ConnectionStatus::ConnectedToAccessPoint | ConnectionStatus::TransmissionEnded => {
            println!("Already connected!");
        }
        _ => {
            println!();
            println!("Connecting to access point with SSID {:?}…", ssid);
            let result = client
                .join_access_point(ssid.as_str(), psk.as_str(), false)
                .expect("Could not connect to access point");
            println!("{:?}", result);
            let status = client
                .get_connection_status()
                .expect("Could not get connection status");
            println!("Connection status: {:?}", status);
        }
    }
    println!(
        "Local IP: {:?}",
        client
            .get_local_address()
            .expect("Could not get local IP address")
            .ip
    );

    println!();
    println!("Creating TCP connection to ipify.com…");
    let remote_ip = Ipv4Addr::new(184, 73, 165, 106);
    let remote_port = 80;
    client
        .send_command(&requests::EstablishConnection::tcp(
            MultiplexingType::NonMultiplexed,
            SocketAddr::V4(SocketAddrV4::new(remote_ip, remote_port)),
        ))
        .expect("Could not establish a TCP connection");

    println!();
    println!("Sending HTTP request…");
    let data = "GET /?format=text HTTP/1.1\r\nHost: api.ipify.org\r\nUser-Agent: ESP8266\r\n\r\n";
    client
        .send_command(&requests::PrepareSendData::new(
            MultiplexingType::NonMultiplexed,
            data.len().try_into().unwrap(),
        ))
        .expect("Could not prepare sending data");
    client
        .send_command(&requests::SendData::<72>::new(&data))
        .expect("Could not send data");
    client
        .send_command(&requests::CloseConnection::new(
            MultiplexingType::NonMultiplexed,
        ))
        .expect("Could not close connection");

    println!("\nStarting main loop, use Ctrl+C to abort…");
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
        type Error = ();

        fn try_start<T>(&mut self, count: T) -> Result<(), Self::Error>
        where
            T: Into<Self::Time>,
        {
            self.start = Instant::now();
            self.duration_ms = count.into();
            Ok(())
        }

        fn try_wait(&mut self) -> nb::Result<(), Self::Error> {
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

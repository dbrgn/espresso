use std::{convert::TryInto, env, io, net::ToSocketAddrs, thread, time::Duration};

use atat::{bbqueue::BBBuffer, ComQueue, Queues};
use espresso::{
    commands::requests,
    types::{ConnectionStatus, MultiplexingType, WifiMode},
};
use heapless::spsc::Queue;
use serialport::{DataBits, FlowControl, Parity, StopBits};

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
        println!("\nNote: To run the example with debug logging, run it like this:");
        println!("\n  RUST_LOG=trace cargo run --example linux --features \"atat/log\" -- /dev/ttyUSB0 115200 mywifi hellopasswd123");
        std::process::exit(1);
    }
    let dev = &args[1];
    let baud_rate: u32 = args[2].parse().unwrap();
    let ssid = &args[3];
    let psk = &args[4];

    println!("Starting (dev={}, baud={:?})…", dev, baud_rate);

    // Open serial port
    let serial_tx = serialport::new(dev, baud_rate)
        .data_bits(DataBits::Eight)
        .flow_control(FlowControl::None)
        .parity(Parity::None)
        .stop_bits(StopBits::One)
        .timeout(Duration::from_millis(5000))
        .open()
        .expect("Could not open serial port");
    let mut serial_rx = serial_tx.try_clone().expect("Could not clone serial port");

    // Initialize
    static mut RES_QUEUE: BBBuffer<1024> = BBBuffer::new();
    static mut URC_QUEUE: BBBuffer<512> = BBBuffer::new();
    static mut COM_QUEUE: ComQueue = Queue::new();
    let queues = Queues {
        res_queue: unsafe { RES_QUEUE.try_split_framed().unwrap() },
        urc_queue: unsafe { URC_QUEUE.try_split_framed().unwrap() },
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
    println!("Firmware version:");
    println!("  AT version: {}", version.at_version);
    println!("  SDK version: {}", version.sdk_version);
    println!("  Compile time: {}", version.compile_time);

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
    println!("Looking up IP for api.my-ip.io…");
    let socket_addr = "api.my-ip.io:80"
        .to_socket_addrs()
        .unwrap()
        .next()
        .unwrap();
    print!("Creating TCP connection to {}…", socket_addr);
    let connect_response = client
        .send_command(&requests::EstablishConnection::tcp(
            MultiplexingType::NonMultiplexed,
            socket_addr.into(),
        ))
        .expect("Could not establish a TCP connection");
    println!(" {:?}", connect_response);

    println!();
    println!("Sending HTTP request…");
    let data = "GET /ip.txt HTTP/1.1\r\nHost: api.my-ip.io\r\nUser-Agent: ESP8266\r\n\r\n";
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
    use std::{convert::TryInto, time::Instant as StdInstant};

    use atat::Clock;
    use fugit::Instant;

    /// A timer with millisecond precision.
    pub struct SysTimer {
        start: StdInstant,
        duration_ms: u32,
        started: bool,
    }

    impl SysTimer {
        pub fn new() -> SysTimer {
            SysTimer {
                start: StdInstant::now(),
                duration_ms: 0,
                started: false,
            }
        }
    }

    impl Clock<1000> for SysTimer {
        type Error = &'static str;

        /// Return current time `Instant`
        fn now(&mut self) -> fugit::TimerInstantU32<1000> {
            let milliseconds = (StdInstant::now() - self.start).as_millis();
            let ticks: u32 = milliseconds.try_into().expect("u32 timer overflow");
            Instant::<u32, 1, 1000>::from_ticks(ticks)
        }

        /// Start timer with a `duration`
        fn start(&mut self, duration: fugit::TimerDurationU32<1000>) -> Result<(), Self::Error> {
            // (Re)set start and duration
            self.start = StdInstant::now();
            self.duration_ms = duration.ticks();

            // Set started flag
            self.started = true;

            Ok(())
        }

        /// Tries to stop this timer.
        ///
        /// An error will be returned if the timer has already been canceled or was never started.
        /// An error is also returned if the timer is not `Periodic` and has already expired.
        fn cancel(&mut self) -> Result<(), Self::Error> {
            if !self.started {
                Err("cannot cancel stopped timer")
            } else {
                self.started = false;
                Ok(())
            }
        }

        /// Wait until timer `duration` has expired.
        /// Must return `nb::Error::WouldBlock` if timer `duration` is not yet over.
        /// Must return `OK(())` as soon as timer `duration` has expired.
        fn wait(&mut self) -> nb::Result<(), Self::Error> {
            let now = StdInstant::now();
            if (now - self.start).as_millis() > self.duration_ms.into() {
                Ok(())
            } else {
                Err(nb::Error::WouldBlock)
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_delay() {
            let mut timer = SysTimer::new();

            // Wait 500 ms
            let before = StdInstant::now();
            timer
                .start(fugit::Duration::<u32, 1, 1000>::from_ticks(500))
                .unwrap();
            nb::block!(timer.wait()).unwrap();
            let after = StdInstant::now();

            let duration_ms = (after - before).as_millis();
            assert!(duration_ms >= 500);
            assert!(duration_ms < 1000);
        }
    }
}

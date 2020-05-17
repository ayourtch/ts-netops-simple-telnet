extern crate telnet;
use telnet::Telnet;

fn main() {
    let ts_netops_host = std::env::var("TS_NETOPS_HOST").unwrap();
    let tcp_target = format!("{}:23", ts_netops_host);

    let mut connection =
        Telnet::connect(tcp_target, 256).expect("Couldn't connect to the server...");
    loop {
        use telnet::TelnetEvent;
        let event = connection.read().expect("Read Error");
        match &event {
            TelnetEvent::Data(bytes) => {
                let string = String::from_utf8_lossy(&bytes);
                println!("Data: {}", &string);
            }
            _ => {
                println!("{:?}", &event);
            }
        }
    }
}

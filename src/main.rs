extern crate regex;
extern crate telnet;
use regex::Regex;
use telnet::Telnet;

enum LoginState {
    Initial,
    SentUsername,
    SentPassword,
    Established,
}

fn main() {
    let ts_netops_host = std::env::var("TS_NETOPS_HOST").unwrap();
    let tcp_target = format!("{}:23", ts_netops_host);

    let mut connection =
        Telnet::connect(tcp_target, 256).expect("Couldn't connect to the server...");

    let mut data_buffer = String::new();
    let mut login_state = LoginState::Initial;
    let username_regex = Regex::new(r"(?m)^[Uu]sername:").unwrap();

    loop {
        use telnet::TelnetEvent;
        let event = connection.read().expect("Read Error");
        match &event {
            TelnetEvent::Data(bytes) => {
                let string = String::from_utf8_lossy(&bytes);
                data_buffer.push_str(&string);
                match login_state {
                    LoginState::Initial => {
                        println!("In initial state");
                        if username_regex.is_match(&data_buffer) {
                            println!("Matchched username prompt! Data buffer: {}", &data_buffer);
                        }
                    }
                    _ => {
                        println!("Other login state. data buffer: {}", &data_buffer);
                    }
                }
            }
            _ => {
                println!("{:?}", &event);
            }
        }
    }
}

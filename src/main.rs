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
    let ts_netops_user = std::env::var("TS_NETOPS_USER").unwrap();
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
                        let maybe_user_match = username_regex.find(&data_buffer);
                        if let Some(user_match) = maybe_user_match {
                            println!("Matchched username prompt! Data buffer: {}", &data_buffer);
                            connection.write(&format!("{}\n", &ts_netops_user).as_bytes());
                            let (_, remainder) = data_buffer.split_at(user_match.end());
                            data_buffer = remainder.to_string();
                            login_state = LoginState::SentUsername;
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

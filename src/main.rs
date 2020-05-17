extern crate regex;
extern crate telnet;
use regex::Regex;
use telnet::Telnet;

#[derive(Debug)]
enum LoginState {
    Initial,
    SentUsername,
    SentPassword,
    Established,
}

fn main() {
    let ts_netops_host = std::env::var("TS_NETOPS_HOST").unwrap();
    let ts_netops_user = std::env::var("TS_NETOPS_USER").unwrap();
    let ts_netops_pass = std::env::var("TS_NETOPS_PASS").unwrap();
    let tcp_target = format!("{}:23", ts_netops_host);

    let mut connection =
        Telnet::connect(tcp_target, 256).expect("Couldn't connect to the server...");

    let mut data_buffer = String::new();
    let mut login_state = LoginState::Initial;
    let username_regex = Regex::new(r"(?m)^[Uu]sername:").unwrap();
    let password_regex = Regex::new(r"(?m)^[Pp]assword:").unwrap();

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
                    LoginState::SentUsername => {
                        let maybe_password_match = password_regex.find(&data_buffer);
                        if let Some(password_match) = maybe_password_match {
                            println!("Matchched password prompt! Data buffer: {}", &data_buffer);
                            connection.write(&format!("{}\n", &ts_netops_pass).as_bytes());
                            let (_, remainder) = data_buffer.split_at(password_match.end());
                            data_buffer = remainder.to_string();
                            login_state = LoginState::SentPassword;
                        }
                    }
                    _ => {
                        println!("Other state: {:?}. data buffer: {}", &login_state, &data_buffer);
                    }
                }
            }
            _ => {
                println!("{:?}", &event);
            }
        }
    }
}

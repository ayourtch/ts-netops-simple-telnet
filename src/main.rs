extern crate env_logger;
extern crate regex;
extern crate telnet;
#[macro_use]
extern crate log;

use regex::Regex;
use telnet::Telnet;

#[derive(Debug)]
enum LoginState {
    Initial,
    SentUsername,
    SentPassword,
    Established,
    ReadingOutput,
}

fn main() {
    env_logger::init();
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
    let author_failed_regex = Regex::new(r"(?m)^% Authorization failed.").unwrap();
    let privexec_regex = Regex::new(r"(?m)^([-_a-z0-9A-Z()]+)#").unwrap();

    loop {
        use telnet::TelnetEvent;
        let event = connection.read().expect("Read Error");
        match &event {
            TelnetEvent::Data(bytes) => {
                let string = String::from_utf8_lossy(&bytes);
                data_buffer.push_str(&string);
                trace!("State: {:?}. data buffer: '{}'", &login_state, &data_buffer);
                match login_state {
                    LoginState::Initial => {
                        let maybe_user_match = username_regex.find(&data_buffer);
                        if let Some(user_match) = maybe_user_match {
                            debug!("Matched username prompt! Data buffer: {}", &data_buffer);
                            connection
                                .write(&format!("{}\n", &ts_netops_user).as_bytes())
                                .unwrap();
                            let (_, remainder) = data_buffer.split_at(user_match.end());
                            data_buffer = remainder.to_string();
                            login_state = LoginState::SentUsername;
                        }
                        if let Some(author_failed_match) = author_failed_regex.find(&data_buffer) {
                            panic!("Authorization failed!");
                        }
                        if let Some(privexec_match) = privexec_regex.find(&data_buffer) {
                            debug!("Matched privexec prompt! Session is open!");
                            let (_, remainder) = data_buffer.split_at(privexec_match.end());
                            data_buffer = remainder.to_string();
                            login_state = LoginState::Established;
                            /* Session established */
                            break;
                        }
                    }
                    LoginState::SentUsername => {
                        let maybe_password_match = password_regex.find(&data_buffer);
                        if let Some(password_match) = maybe_password_match {
                            debug!("Matched password prompt! Data buffer: {}", &data_buffer);
                            connection
                                .write(&format!("{}\n", &ts_netops_pass).as_bytes())
                                .unwrap();
                            let (_, remainder) = data_buffer.split_at(password_match.end());
                            data_buffer = remainder.to_string();
                            login_state = LoginState::SentPassword;
                        }
                    }
                    LoginState::SentPassword => {
                        let maybe_privexec_match = privexec_regex.find(&data_buffer);
                        if let Some(privexec_match) = maybe_privexec_match {
                            debug!("Matched privexec prompt! Session is open!");
                            let (_, remainder) = data_buffer.split_at(privexec_match.end());
                            data_buffer = remainder.to_string();
                            login_state = LoginState::Established;
                            /* Session established */
                            break;
                        }
                    }
                    _ => {
                        debug!(
                            "Other state: {:?}. data buffer: {}",
                            &login_state, &data_buffer
                        );
                    }
                }
            }
            _ => {
                debug!("{:?}", &event);
            }
        }
    }
    debug!("Showtime!");
    connection.write(b"term len 0\n").unwrap();
    login_state = LoginState::ReadingOutput;

    loop {
        use telnet::TelnetEvent;
        let event = connection.read().expect("Read Error");
        match &event {
            TelnetEvent::Data(bytes) => {
                let string = String::from_utf8_lossy(&bytes);
                data_buffer.push_str(&string);
                match login_state {
                    LoginState::ReadingOutput => {
                        let maybe_privexec_match = privexec_regex.find(&data_buffer);
                        if let Some(privexec_match) = maybe_privexec_match {
                            debug!("Matched privexec prompt! Data buffer: {}", &data_buffer);
                            let (_, remainder) = data_buffer.split_at(privexec_match.end());
                            data_buffer = remainder.to_string();
                            login_state = LoginState::Established;
                            break;
                        }
                    }
                    _ => {
                        debug!(
                            "Other state: {:?}. data buffer: {}",
                            &login_state, &data_buffer
                        );
                    }
                }
            }
            _ => {
                debug!("{:?}", &event);
            }
        }
    }

    for command_to_run in std::env::args().skip(1) {
        let command_to_run = command_to_run.replace("\\n", "\n");
        connection
            .write(&format!("{}\n", command_to_run).as_bytes())
            .unwrap();
        login_state = LoginState::ReadingOutput;
        let mut result = "".to_string();

        loop {
            use telnet::TelnetEvent;
            let event = connection.read().expect("Read Error");
            match &event {
                TelnetEvent::Data(bytes) => {
                    let string = String::from_utf8_lossy(&bytes);
                    data_buffer.push_str(&string);
                    debug!("Got output:'{}'", &string);
                    match login_state {
                        LoginState::ReadingOutput => {
                            let maybe_privexec_match = privexec_regex.find(&data_buffer);
                            if let Some(privexec_match) = maybe_privexec_match {
                                debug!("Matched privexec prompt! Data buffer: {}", &data_buffer);
                                let (command_output, remainder) =
                                    data_buffer.split_at(privexec_match.end());
                                result = command_output.to_string();
                                data_buffer = remainder.to_string();
                                login_state = LoginState::Established;
                                break;
                            }
                        }
                        _ => {
                            debug!(
                                "Other state: {:?}. data buffer: {}",
                                &login_state, &data_buffer
                            );
                        }
                    }
                }
                _ => {
                    debug!("{:?}", &event);
                }
            }
        }
        println!("Result: {}", &result);
    }
}

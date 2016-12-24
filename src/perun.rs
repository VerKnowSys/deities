use std::os::unix::net::UnixStream;
use std::io::prelude::*;

use service::Service;


/*
 * Perun is a supervisor deity
 */
pub struct Perun;


impl Perun {


    fn try_unix_socket(path: String) -> Result<String, String> {
        match UnixStream::connect(path.clone()) {
            Ok(mut stream) => {
                match stream.write_all(b"version") {
                    Err(cause) =>
                        Err(format!("Service is not listening on UNIX socket: {:?}! Err cause: {:?}", path, cause)),

                    Ok(_) => {
                        // let mut response = String::new();
                        // stream.read_to_string(&mut response).unwrap();
                        Ok(format!("Service is listening on UNIX socket: {:?}.", path))
                    },
                }
            },
            Err(cause) =>
                Err(format!("Service has no UNIX socket: {:?}! Err cause: {:?}", path, cause)),
        }
    }


    pub fn checks_for(service: Service) -> Result<String, String> {
        debug!("Perun::checks_for: {:?}", service);

        // perform Service checks:
        match service.unix_socket {
            Some(unix_socket) => {
                match Perun::try_unix_socket(unix_socket.clone()) {
                    Ok(_) => {
                        debug!("Successfully opened UNIX socket: {:?}", unix_socket);
                    },
                    Err(err) => return Err(err),
                }
            },
            None => {},
        }

        // match service.pid_file {

        // }

        Ok("Checks passed".to_string())
    }

    // fn check_service(service: Service) -> Result<Service, String>;

    // fn check_name(service: Service) -> Result<String, String> {
    //     match service.name.as_ref() {
    //         "" => None,
    //         an => Some(String::from(an)),
    //     }
    // }


    // fn check_unix_socket(file_name: String) -> Option<String> {
    //     match service.unix.as_ref() {
    //         "" => None,
    //         an => Some(String::from(an)),
    //     }
    // }

    // fn check_pid_file(file_name: String) -> Option<String>;
    // fn check_tcp_port(file_name: String) -> Option<usize>;
    // fn check_udp_port(file_name: String) -> Option<usize>;
    // fn check_domains(file_name: String) -> Option<Vec<String>>;
}

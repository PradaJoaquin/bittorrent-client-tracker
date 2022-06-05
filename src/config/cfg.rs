use std::fs::File;
use std::io;
use std::io::BufRead;
use std::io::BufReader;

use super::constants;

/// Cfg struct containing the config file information, previusly created with Cfg::new.
///
/// tcp_port: u16,
/// log_directory: String,
/// download_directory: String,
#[derive(Debug)]
pub struct Cfg {
    pub tcp_port: u16,
    pub log_directory: String,
    pub download_directory: String,
}

impl Cfg {
    /// Builds a Cfg struct containing the config file information by the given path.
    /// The format of the config file must be: {config_name}={config_value} (without brackets).
    /// In case of success it returns a Cfg struct.
    ///
    /// It returns an io::Error if:
    /// - The path to the config file does not exist or could not be open/readed.
    /// - The confing file has wrong format.
    /// - A wrong config_name was in the config file.
    /// - tcp_port setting is not a number in the config file.
    /// - Minimum number of correct settings were not reached.
    pub fn new(path: &str) -> io::Result<Self> {
        let mut cfg = Self {
            tcp_port: 0,
            log_directory: String::from(""),
            download_directory: String::from(""),
        };

        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut settings_loaded = 0;

        for line in reader.lines() {
            let current_line = line?;
            let setting: Vec<&str> = current_line.split('=').collect();

            if setting.len() != 2 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Invalid config input: {}", current_line),
                ));
            }
            cfg = Self::load_setting(cfg, setting[0], setting[1])?;
            settings_loaded += 1;
        }
        if settings_loaded < constants::MIN_SETTINGS {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "Minimum number of correct settings were not reached: {}",
                    settings_loaded
                ),
            ));
        }
        Ok(cfg)
    }

    fn load_setting(mut self, name: &str, value: &str) -> io::Result<Self> {
        match name {
            constants::TCP_PORT => {
                let parse = value.parse::<u16>();
                match parse {
                    Err(_) => {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            format!("Invalid config, TCP_PORT is not a number: {}", value),
                        ));
                    }
                    Ok(parse) => {
                        self.tcp_port = parse;
                    }
                }
            }
            constants::LOG_DIRECTORY => self.log_directory = String::from(value),

            constants::DOWNLOAD_DIRECTORY => self.download_directory = String::from(value),

            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Invalid config setting name: {}", name),
                ))
            }
        }
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, io::Write};

    // tests:
    //  1- test todo ok
    //  2- test archivo de config no existe
    //  3- test archivo vacio
    //  4- test setting que no existe
    //  5- test solo 2 settings
    //  6- test tcp_port no es numero
    //  7- test no importa el orden de los settings en el archivo
    //  8- test mal formato

    #[test]
    fn test_good_config() {
        let path = "./test_good_config.txt";
        let contents = b"TCP_PORT=1000\nLOG_DIRECTORY=./log\nDOWNLOAD_DIRECTORY=./download";
        create_and_write_file(path, contents);

        create_and_assert_config_is_ok(path, 1000, "./log", "./download");
    }

    #[test]
    fn test_bad_path() {
        let path = "bad path";
        let config = Cfg::new(path);
        assert!(config.is_err());
    }

    #[test]
    fn test_empty_file() {
        let path = "./test_empty_file.txt";
        let contents = b"";
        create_and_write_file(path, contents);

        create_and_assert_config_is_not_ok(path);
    }

    #[test]
    fn test_setting_doesnt_exist() {
        let path = "./test_setting_doesnt_exist.txt";
        let contents = b"WRONG_SETTING=1000";
        create_and_write_file(path, contents);

        create_and_assert_config_is_not_ok(path);
    }

    #[test]
    fn test_bad_number_of_settings() {
        let path = "./test_bad_number_of_settings.txt";
        let contents = b"TCP_PORT=1000\nLOG_DIRECTORY=./log";
        create_and_write_file(path, contents);

        create_and_assert_config_is_not_ok(path);
    }

    #[test]
    fn test_tcp_port_not_a_number() {
        let path = "./test_tcp_port_not_a_number.txt";
        let contents = b"TCP_PORT=abcd\nLOG_DIRECTORY=./log\nDOWNLOAD_DIRECTORY=./download";
        create_and_write_file(path, contents);

        create_and_assert_config_is_not_ok(path);
    }

    #[test]
    fn test_order_doesnt_matter() {
        let path = "./test_order_doesnt_matter.txt";
        let contents = b"LOG_DIRECTORY=./log2\nDOWNLOAD_DIRECTORY=./download2\nTCP_PORT=2500";
        create_and_write_file(path, contents);

        create_and_assert_config_is_ok(path, 2500, "./log2", "./download2");
    }

    #[test]
    fn test_bad_format() {
        let path = "./test_bad_format.txt";
        let contents = b"TCP_PORT=abcd=1234\nLOG_DIRECTORY=./log\nDOWNLOAD_DIRECTORY=./download";
        create_and_write_file(path, contents);

        create_and_assert_config_is_not_ok(path);
    }

    // Auxiliary functions

    fn create_and_write_file(path: &str, contents: &[u8]) -> () {
        let mut file =
            File::create(path).expect(&format!("Error creating file in path: {}", &path));
        file.write_all(contents)
            .expect(&format!("Error writing file in path: {}", &path));
    }

    fn create_and_assert_config_is_ok(
        path: &str,
        tcp_port: u16,
        log_directory: &str,
        download_directory: &str,
    ) {
        let config = Cfg::new(path);

        assert!(config.is_ok());

        let config = config.expect(&format!("Error creating config in path: {}", &path));

        assert_eq!(config.tcp_port, tcp_port);
        assert_eq!(config.log_directory, log_directory);
        assert_eq!(config.download_directory, download_directory);

        fs::remove_file(path).expect(&format!("Error removing file in path: {}", &path));
    }

    fn create_and_assert_config_is_not_ok(path: &str) {
        let config = Cfg::new(path);
        assert!(config.is_err());
        fs::remove_file(path).expect(&format!("Error removing file in path: {}", &path));
    }
}

use std::sync::RwLock;
use config::Config;

lazy_static! {
    pub static ref CONFIGURATION: RwLock<Config> = RwLock::new(Config::new());
}

pub fn load_configuration_file() {
    match CONFIGURATION.write() {
        Ok(mut config_file) => {
            config_file.merge(config::File::with_name("config.toml")).unwrap();
        }
        Err(e) => panic!("Error loading configuration file in main: {}", e)
    }
}

pub fn read_debug() -> bool {
    match CONFIGURATION.read() {
        Ok(config) => {
            config.get_bool("debug").unwrap()
        },
        Err(e) => {
            println!("Error occurred when reading global config \"debug\" {}", e);
            false
        }
    }
}
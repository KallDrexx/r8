use clap::App;

pub struct Settings {
    pub rom_file: String,
    pub start_paused: bool,
    pub instructions_per_second: u16,
    pub frames_per_second: u8,
}

impl Settings {
    pub fn from_cli_arguments() -> Self {
        let yaml = load_yaml!("cli.yml");
        let matches = App::from_yaml(yaml).get_matches();
        Self {
            rom_file: matches.value_of("INPUT").unwrap().to_owned(),
            start_paused: matches.is_present("paused"),
            instructions_per_second: value_t!(matches, "ips", u16).unwrap_or(60 * 5),
            frames_per_second: value_t!(matches, "fps", u8).unwrap_or(60),
        }
    }
}
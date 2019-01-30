use clap::App;

pub struct Settings {
    pub rom_file: String,
    pub start_paused: bool,
    pub instructions_per_frame: u8,
}

impl Settings {
    pub fn from_cli_arguments() -> Self {
        let yaml = load_yaml!("cli.yml");
        let matches = App::from_yaml(yaml).get_matches();
        Self {
            rom_file: matches.value_of("INPUT").unwrap().to_owned(),
            start_paused: matches.is_present("paused"),
            instructions_per_frame: value_t!(matches, "ipf", u8).unwrap_or(1),
        }
    }
}
extern crate yaml_rust;
use yaml_rust::{Yaml, YamlLoader};

use std::env;
use std::fs;

pub struct Settings {
    pub data_folder: String,
}

pub fn get_settings() -> Settings {
    let mut path = env::current_dir().expect("unable to get current directory");
    path.push("settings.yml");

    let contents =
        fs::read_to_string(path).expect("settings.yml file is expected in the execution directory");

    let docs =
        YamlLoader::load_from_str(&contents).expect("unable to parse settings file as a YAML file");
    let yaml = &docs[0];

    if let Yaml::String(data_folder) = &yaml["Data Folder"] {
        return Settings {
            data_folder: data_folder.to_string(),
        };
    } else {
        panic!("unable to find 'Data Folder' in settings file.")
    }
}

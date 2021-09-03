use rofi::pango;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use std::env::var;
use thiserror::Error;
use yaml_rust::{yaml::Yaml, YamlLoader};

const DEFAULT_STYLE_FG_COLOR: &'static str = "#eeeeee";
const DEFAULT_STYLE_FONT_SIZE: pango::FontSize = pango::FontSize::Small;

#[derive(Debug)]
pub struct Config {
    pub mappings: HashMap<String, String>,
    pub style: Style,
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("config io error: {0}")]
    Io(std::io::Error),

    #[error("yaml scanning error: {0}")]
    YamlScan(yaml_rust::ScanError),

    #[error("error creating mappings")]
    Map,

    #[error("error reading style")]
    Style,

    #[error("invalid font size: {0}")]
    InvalidFontSize(String),

    #[error("unable to automatically determine config path")]
    SmartPath,
}

#[derive(Debug)]
pub struct Style {
    pub fg_color: String,
    pub size: pango::FontSize,
}

pub fn load_config<'a, P: AsRef<Path> + std::fmt::Debug>(path: P) -> Result<Config, ConfigError> {
    let mut yaml_string = String::new();

    let file = File::open(&path).map_err(|err| ConfigError::Io(err))?;
    let mut reader = BufReader::new(file);
    reader
        .read_to_string(&mut yaml_string)
        .map_err(|err| ConfigError::Io(err))?;

    let yaml = YamlLoader::load_from_str(&yaml_string).map_err(|err| ConfigError::YamlScan(err))?;
    let doc = &yaml[0];

    Ok(Config {
        mappings: get_mappings(doc)?,
        style: get_style(doc)?,
    })
}

pub fn smart() -> Result<Config, ConfigError> {
    let config_dir = match var("XDG_CONFIG_HOME") {
        Ok(dir) => dir,
        Err(_) => format!("{}/.config", match var("HOME") {
            Ok(home) => home,
            Err(_) => match var("USER") {
                Ok(user) => format!("/home/{}", user),
                Err(_) => return Err(ConfigError::SmartPath),
            },
        }),
    };
    let path = format!("{}/emoti/config.yaml", config_dir);
    println!("path: {}",path);
    load_config(path)
}

fn get_mappings(doc: &Yaml) -> Result<HashMap<String, String>, ConfigError> {
    if let Yaml::Hash(map) = &doc["mappings"] {
        // far from efficient, but it'll do. it's worth the DRY imo
        let mappings = yaml_map_to_hashmap(map)
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        Ok(mappings)
    } else {
        Err(ConfigError::Map)
    }
}

fn get_style(doc: &Yaml) -> Result<Style, ConfigError> {
    if let Yaml::Hash(map) = &doc["style"] {
        let map = yaml_map_to_hashmap(map);

        let size = if let Some(font_size) = map.get("size") {
            use pango::FontSize::*;

            match font_size.to_lowercase().as_ref() {
                "verytiny" => VeryTiny,
                "tiny" => Tiny,
                "small" => Small,
                "normal" => Normal,
                "large" => Large,
                "huge" => Huge,
                "veryhuge" => VeryHuge,

                // included just because. might be useful
                "smaller" => Smaller, // relative: smaller than parent
                "larger" => Larger,   // relative: larger than parent
                _ => return Err(ConfigError::InvalidFontSize(font_size.to_string())),
            }
        } else {
            DEFAULT_STYLE_FONT_SIZE
        };

        Ok(Style {
            fg_color: map
                .get("fg_color")
                .unwrap_or(&DEFAULT_STYLE_FG_COLOR)
                .to_string(),
            size,
        })
    } else {
        Err(ConfigError::Style)
    }
}

fn yaml_map_to_hashmap<'a>(map: &'a yaml_rust::yaml::Hash) -> HashMap<&'a str, &'a str> {
    let mut mappings = HashMap::new();
    map.iter()
        .filter_map(|(k, v)| match (k, v) {
            (Yaml::String(key), Yaml::String(value)) => Some((key, value)),
            _ => None,
        })
        .for_each(|(k, v)| {
            mappings.insert(k.as_ref(), v.as_ref());
        });

    mappings
}

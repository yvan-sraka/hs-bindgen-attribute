use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
pub(crate) struct Config {
    pub(crate) default: Option<String>,
    pub(crate) version: Option<String>,
}

pub(crate) fn config() -> Config {
    toml::from_str(&fs::read_to_string(".hsbindgen").expect(
        "fail to read content of `.hsbindgen` configuration file
n.b. you have to run the command `hackage-pack` to generate it",
    ))
    .expect("fail to parse TOML content of `hsbindgen` file")
    // FIXME: compatibility constraints should be written on version of
    // `hackage-pack` used
}

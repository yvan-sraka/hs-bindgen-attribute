use semver::{Version, VersionReq};
use serde::Deserialize;
use std::fs;

/// Struct that map the content of `hsbindgen.toml` config file
#[derive(Deserialize)]
pub(crate) struct Config {
    pub(crate) default: Option<String>,
    pub(crate) version: Option<String>,
}

/// Read `hsbindgen.toml` config file generated by `cargo-cabal`
pub(crate) fn config() -> Config {
    let config = toml::from_str(
        &fs::read_to_string("hsbindgen.toml")
            .or_else(|_| fs::read_to_string(".hsbindgen")) // FIXME: legacy ...
            .expect(
                "fail to read content of `hsbindgen.toml` configuration file
n.b. you have to run the command `cargo-cabal` to generate it",
            ),
    )
    .expect("fail to parse TOML content of `hsbindgen.toml` file");
    check_version(&config);
    config
}

/// Compatibility constraints on `cargo-cabal` version used
fn check_version(config: &Config) {
    let req = VersionReq::parse("<=0.7").unwrap();
    let version = config
        .version
        .as_ref()
        .expect("a version field is required in `hsbindgen.toml`");
    let version = Version::parse(version)
        .expect("version field of `hsbindgen.toml` does not follow SemVer format");
    assert!(
        req.matches(&version),
        "incompatible versions of `cargo-cabal`/`hs-bindgen` used, please update"
    );
}

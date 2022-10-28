use std::borrow::Cow;
use std::fs::File;
use std::fs::read_to_string;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use directories::BaseDirs;
use serde::{Serialize, Deserialize};

use skim::*;

/// Holds all state required by a runner to execute a command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Runner {
    /// The name to call the runner in the TUI, for searching / selecting
    pub name: String,

    /// The command to execute at run-time
    pub cmd: String,

    /// False if runfast should prompt for an extra ENTER press before exiting.
    pub quit_fast: bool,
}

impl Runner {
    /// Returns a `Runner`, filling in any blanks with defaults.
    fn new_from_config(conf: &RunnerConfig) -> Runner {
        Runner {
            name: match &conf.name {
                Some(n) => n.clone(),
                None => "Default Runner Name".to_string(),
            },
            cmd: match &conf.cmd {
                Some(c) => c.clone(),
                None => "echo 'command not set'".to_string(),
            },
            quit_fast: conf.quit_fast.unwrap_or(false)
        }
    }

    /// Uses this runner to execute the run command
    pub fn run(&self) {
        let mut c = Command::new("bash");
        c.arg("-c");
        c.arg(&self.cmd);
        let result = c.status();
        if result.is_err() {
            println!("Error Running Command: {:#?}", result);
        }
        if !self.quit_fast {
            println!("Press ENTER to exit...");
            let _ = Command::new("bash").arg("-c").arg("read").status();
        }
    }
}

impl SkimItem for Runner {
    fn text(&self) -> prelude::Cow<str> {
        Cow::Borrowed(&self.name)
    }

    fn preview(&self, _context: PreviewContext) -> ItemPreview {
        let mut prev = String::new();
        prev.push_str("[PARAMS]\n");

        prev.push_str("name=");
        prev.push_str(self.name.as_str());
        prev.push('\n');

        prev.push_str("cmd=");
        prev.push_str(self.cmd.as_str());
        prev.push('\n');

        prev.push_str("quit_fast=");
        prev.push_str(&self.quit_fast.to_string());
        prev.push('\n');

        ItemPreview::Text(prev)
    }
}


/// Defines config structure for reading
#[derive(Debug, Deserialize)]
struct Config {
    runners: Option<Vec<RunnerConfig>>,
}

/// Struct to use for parsing toml, since each runner in the toml may not have
/// a complete config defined, but we can construct one out of RUNNER_DEFAULTS
/// and the defaults in both config files
#[derive(Debug, Deserialize)]
struct RunnerConfig {
    name: Option<String>,
    cmd: Option<String>,
    quit_fast: Option<bool>,
}

pub fn load_runners() -> Vec<Runner> {
    // try to load ~/.config/runfast/defaults.toml and ~/.config/runfast/runners.toml
    // prefer values in runners.toml if there are clashes
    let base_dirs = BaseDirs::new().unwrap();

    // get default config directory (usually ~/.config/)
    let confdir = base_dirs.config_dir();

    // load default config
    let default_path = confdir.join("runfast/defaults.toml");
    if !default_path.exists() {
        generate_default_config(&default_path);
    }
    let default_confstring = read_to_string(default_path).unwrap();
    let default_configs = match toml::from_str::<Config>(&default_confstring) {
        Ok(conf) => Some(conf),
        Err(e) => panic!("Could not parse default config: {}", e),
    };

    // load user config
    let userconf_path = confdir.join("runfast/runners.toml");
    let mut user_configs: Option<Config> = None;
    if userconf_path.exists() {
        let user_confstring = read_to_string(userconf_path).unwrap();
        match toml::from_str::<Config>(&user_confstring) {
            Ok(conf) => user_configs = Some(conf),
            Err(e) => panic!("Could not parse user config: {}", e),
        }
    }

    let mut runners = get_runners_from_config(&user_configs);
    let mut default_runners = get_runners_from_config(&default_configs);

    while !default_runners.is_empty() {
        let dr = default_runners.pop().unwrap();
        let mut already_exists = false;
        for r in &runners {
            if dr.name == r.name {
                already_exists = true;
                break;
            }
        }
        if !already_exists {
            runners.push(dr);
        }
    }
    runners
}

fn get_runners_from_config(conf: &Option<Config>) -> Vec<Runner> {
    let mut runners:Vec<Runner> = Vec::new();

    if let Some(c) = conf {
        if let Some(r) = &c.runners {
            for runc in r {
                runners.push(Runner::new_from_config(runc))
            }
        }
    }

    runners
}

fn generate_default_config(default_path: &Path) {
    let dirpath = default_path.parent().unwrap();
    match std::fs::create_dir_all(default_path.parent().unwrap()) {
        Ok(_) => (),
        Err(e) => panic!("{:?} does not exist and could not be created, with\
            error {}", dirpath, e),
    }
    let default_conf = File::create(default_path);
    match default_conf {
        Ok(mut conf_file) => {
            conf_file.write_all(include_bytes!("defaults.toml"))
                .expect("couldn't write default conf file");
        },
        Err(e) => {
            println!("Could not create file at: {}, error: {:#?}",
                default_path.display(), e);
            panic!("No default config could be created, panicing");
        },
    }
}

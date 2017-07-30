#[macro_use]
extern crate clap;
extern crate mod_example;

mod args;
#[macro_use] mod macros;

use mod_example::run;

fn main() {
    let matches = args::get_args();

    // Gets a value for config if supplied by user, or defaults to "default.conf"
    let config_path = matches.value_of("config").unwrap_or("settings.toml");
    let server = matches.value_of("server").unwrap();
    let project_name = matches.value_of("project").unwrap_or("default");
    let watch = matches.occurrences_of("watch") == 1;
    let verbose = matches.occurrences_of("verbose");

    // more program logic goes here...
    run(config_path, project_name, server, watch);
}

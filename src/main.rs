mod args;
#[macro_use]
extern crate clap;
#[macro_use(slog_log, slog_error, slog_info, slog_record, slog_b, slog_record_static, slog_kv)]
extern crate slog;
#[macro_use(error, info)]
extern crate slog_scope;

extern crate rn;
extern crate slog_term;
extern crate slog_async;
extern crate slog_json;

use rn::run;
use rn::my_logger;
use std::path::PathBuf;

fn main() {

    let matches = args::get_args();

    // Gets a value for config if supplied by user, or defaults to "default.conf"
    let config_path = matches.value_of("config").unwrap_or("~/bin/settings.toml");
    let server = matches.value_of("server").unwrap();
    let project_name = matches.value_of("project").unwrap_or("default");
    let watch = matches.occurrences_of("watch") == 1;
    let user = matches.value_of("user");
    let password = matches.value_of("password");
    let identity = matches.value_of("identity");
    // TODO: set log path from args, and log_level
    let path_str = "/tmp/app.log";
    //let path_str = "";
    let log_path = match path_str.len() {
        0 => None,
        _ => Some(PathBuf::from(path_str)),
    };
    //let log_path = Some(PathBuf::from(r"/tmp/app.log"));
    let log_level = 1;
    let log = my_logger::get_global_log(log_level, log_path).unwrap();
    let _guard = slog_scope::set_global_logger(log);

    println!("user: {:?}, passowrd: {:?}, identity: {:?}", user, password, identity);

    if let Err(ref e) = run(config_path, project_name, server, watch, user, password, identity) {
        error!("error: {}", e);

        for e in e.iter().skip(1) {
            error!("caused by: {}", e);
        }

        // The backtrace is not always generated. Try to run this example
        // with `RUST_BACKTRACE=1`.
        if let Some(backtrace) = e.backtrace() {
            info!("backtrace: {:?}", backtrace);
        }

        std::process::exit(1);
    }
}

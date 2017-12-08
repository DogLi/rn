mod args;
#[macro_use]
extern crate clap;
#[macro_use(slog_log, slog_error, slog_debug, slog_info, slog_record, slog_b, slog_record_static, slog_kv)]
extern crate slog;
#[macro_use(error, info, debug)]
extern crate slog_scope;

extern crate rn;
extern crate slog_term;
extern crate slog_async;
extern crate slog_json;
extern crate shellexpand;

use rn::run;
use rn::my_logger;
use std::path::PathBuf;
use shellexpand::tilde;

fn main() {

    let matches = args::get_args();

    // Gets a value for config if supplied by user, or defaults to "~/bin/settings.toml"
    let config_path = tilde(matches.value_of("config").unwrap_or("~/bin/settings.toml")).into_owned();
    let config_path = config_path.as_str();
    let config_path = "~/bin/settings.toml";
    let server = matches.value_of("server").unwrap();
    let project_name = matches.value_of("project").unwrap_or("default");
    let watch = matches.occurrences_of("watch") == 1;
    let user = matches.value_of("user");
    let password = matches.value_of("password");
    let identity = matches.value_of("identity");
    let log_path = matches.value_of("log");

    debug!("config path: {:?}", config_path);
    // TODO: set log path from args, and log_level
    let log_path = match log_path{
        None => None,
        Some(path_str)=> Some(PathBuf::from(path_str)),
    };
    let log_level = matches.occurrences_of("v") as i8;
    debug!("log level: {:?}", log_level);
    let log = my_logger::get_global_log(log_level, log_path).unwrap();
    // 必须明确写出这一句
    let _guard = slog_scope::set_global_logger(log);
    info!("user: {:?}, password: {:?}, identity: {:?}", user, password, identity);
    let config_path_buf = &PathBuf::from(config_path);

    if let Err(ref e) = run(config_path_buf, project_name, server, watch, user, password, identity) {
        error!("error: {}", e);
        for e in e.iter().skip(1) {
            error!("caused by: {}", e);
        }
        // The backtrace is not always generated. Try to run this example
        // with `RUST_BACKTRACE=1`.
        if let Some(backtrace) = e.backtrace() {
            error!("backtrace: {:?}", backtrace);
        }
    }
}

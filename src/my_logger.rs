use errors::*;
use slog;
use slog_async;
use slog_term;
use slog_json;

use std::path::PathBuf;
use std::fs::OpenOptions;
use slog::{Level, Drain};



pub fn get_global_log(log_level: i8, log_path: Option<PathBuf>) -> Result<slog::Logger>
{
    let log_level = match log_level {
        0 => Level::Critical,
        1 => Level::Error,
        2 => Level::Warning,
        3 => Level::Info,
        4 => Level::Debug,
        n if n > 5 => Level::Trace,
        n if n < 0 => Level::Critical,
        _ => Level::Debug,
    };

    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let console_drain = slog_async::Async::new(drain).build().fuse();
    let global_info = slog_o!("version" => "0.5",
                        "location" => slog::FnValue(move |info| {
                            format!("{}:{} {}",
                                    info.file(),
                                    info.line(),
                                    info.module(),
                                    )
                        }));

    if log_path.is_some() {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(false)
            .open(log_path.unwrap())?;

        let builder = slog_json::Json::new(file).add_key_value(slog_o!("type"=> "json")).add_default_keys();
        let drain = builder.build().map(slog::Fuse);
        let file_drain = slog_async::Async::new(drain).build().fuse();
        // join together all drains
        let drains = slog::Duplicate::new(console_drain, file_drain).fuse();
        let drains = slog::LevelFilter::new(drains, log_level).map(slog::Fuse);
        let log = slog::Logger::root(drains, global_info);
        return Ok(log);
    } else {
        let drains = slog::LevelFilter::new(console_drain, log_level).map(slog::Fuse);
        let log = slog::Logger::root(drains, global_info);
        return Ok(log);
    }
}

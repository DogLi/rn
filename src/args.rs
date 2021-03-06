use clap;

pub fn get_args() -> clap::ArgMatches<'static> {
    clap::App::new("rn")
        .global_settings(&[clap::AppSettings::ColoredHelp])
        .version(crate_version!())
        .author(crate_authors!())
        .about("a realtime file transformer.")
        .arg(clap::Arg::with_name("project")
                 .short("p")
                 .long("project")
                 .value_name("PROJECT")
                 .takes_value(true)
                 .default_value("default")
                 .help("set the project name to be deployed!"))
        .arg(clap::Arg::with_name("server")
                 .required(true)
                 .index(1)
                 .help("set the remote server name which comes from ~/.ssh/config or inner rule."))
        .arg(clap::Arg::with_name("watch")
                 .short("w")
                 .long("watch")
                 .help("keep watching for file change!"))
        .arg(clap::Arg::with_name("config")
                 .short("c")
                 .long("config")
                 .takes_value(true)
                 .required(false)
                 .default_value("~/bin/settings.toml")
                 .help("Config for rn's variables."))
        .arg(clap::Arg::with_name("user")
                 .long("user")
                 .required(false)
                 .takes_value(true)
                 .help("set ssh username for remote host."))
        .arg(clap::Arg::with_name("password")
                 .long("password")
                 .takes_value(true)
                 .required(false)
                 .help("set ssh password for remote host."))
        .arg(clap::Arg::with_name("port")
            .long("port")
            .takes_value(true)
            .required(false)
            .help("set ssh port for remote host."))
        .arg(clap::Arg::with_name("identity")
                 .short("i")
                 .long("indentity")
                 .takes_value(true)
                 .required(false)
                 .help("set ssh identity file path for remote host."))
        .arg(clap::Arg::with_name("log")
            .long("log")
            .takes_value(true)
            .required(false)
            .help("set log path"))
        .arg(clap::Arg::with_name("delete")
            .long("delete")
            .short("d")
            .help("delete the remote file in not exits in current folder or not."))
        .arg(clap::Arg::with_name("v")
            .short("v")
            .multiple(true)
            .help("see detail information")
        )
        .get_matches()
}

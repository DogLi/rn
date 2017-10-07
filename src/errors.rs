extern crate regex;
extern crate toml;
extern crate ssh2;
extern crate notify;

use std::{io, num};
use std::convert::From;
use std::path::StripPrefixError;

error_chain! {
    foreign_links {
        Format(regex::Error);
        Io(io::Error);
        Toml(toml::de::Error);
        SSHError(ssh2::Error);
        PathError(StripPrefixError);
        NumParseError(num::ParseIntError);
        NotifyError(notify::Error);
    }
}

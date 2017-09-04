extern crate regex;
extern crate toml;
extern crate ssh2;
use std::{io, num};
use std::path::PathBuf;
use std::convert::From;
use std::path::StripPrefixError;

/// Create the Error, ErrorKind, ResultExt, and Result types
error_chain! {
    foreign_links {
        Format(regex::Error);
        Io(io::Error);
        Toml(toml::de::Error);
        SSHError(ssh2::Error);
        PathError(StripPrefixError);
        NumParseError(num::ParseIntError);
    }
}

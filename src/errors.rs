extern crate toml;
extern crate notify;
extern crate regex;

use std::{self, io, num};
use std::convert::From;
use std::path::StripPrefixError;

error_chain! {
    foreign_links {
        FfiNulError(std::ffi::NulError);
//        OptionNone(std::option::NoneError);
        Regex(regex::Error);
        Io(io::Error);
        Toml(toml::de::Error);
        PathError(StripPrefixError);
        NumParseError(num::ParseIntError);
        NotifyError(notify::Error);
    }
}

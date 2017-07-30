extern crate regex;
extern crate toml;
extern crate ssh2;
use std::io;
use std::path::PathBuf;
use std::convert::From;
use std::path::StripPrefixError;

// Create the Error, ErrorKind, ResultExt, and Result types
error_chain! {
    foreign_links {
        Format(regex::Error);
        Io(io::Error);
        Toml(toml::de::Error);
        SSHError(ssh2::Error);
        PathError(StripPrefixError);
    }
    errors {
        ConditionFormat(spec: String) {
            display("{} is not a valid condition spec: format is FILENAME,REGEX", spec)
        }
        KnownHostFormat(path: PathBuf, lineno: usize, line: String) {
            display("{} line {}: {:?}", path.to_str().unwrap_or("(unprintable path)"), lineno, line)
        }
        NameError(name: String, protocol: String) {
            display("{} with protocol {} would result in a bad filename", name, protocol)
        }
    }
}

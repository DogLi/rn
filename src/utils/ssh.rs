use ssh2;
use ssh2::{Sftp, Session, Error, RenameFlags};
use std::io::prelude::*;
use std::io;
use std::net;
use std::path::Path;
use std::net::TcpStream;
use errors::*;

pub struct SSHClient {
    session: ssh2::Session,
    stream: net::TcpStream,
}

pub struct SftpClient<'a> {
    ssh_client: &'a SSHClient,
    sftp: Sftp<'a>
}

impl SSHClient {
    pub fn new<S, P>(ip: S, port: u16, username: S, password: Option<S>, priv_key: Option<P>) -> Self
    where S: AsRef<str>, P: AsRef<Path> {
        let mut tcp = TcpStream::connect(&(ip.as_ref(), port)).unwrap();
        // nightly-only experimental API: https://doc.rust-lang.org/nightly/std/net/struct.TcpStream.html#method.connect_timeout
        // let mut tcp = TcpStream::connect_timeout(ipaddress, timeout).unwrap();
        let mut session = Session::new().unwrap();
        session.set_timeout(2000);

        session.handshake(&tcp).unwrap();
        // private key优先级高
        match priv_key{
            None => {
                match password{
                    None => {
                        println!("no passowrd or priv_key");
                        panic!();
                    },
                    Some(password) => {
                        session.userauth_password(username.as_ref(), password.as_ref()).unwrap();
                    }
                }
            },
            Some(priv_key) => {
                println!("use private key '{:?}' in ssh", priv_key.as_ref());
                // TODO: use priv key to access
                session.userauth_pubkey_file(username.as_ref(), None, priv_key.as_ref(), None);
            }
        }
        assert!(session.authenticated());

        SSHClient {
            session: session,
            stream: tcp,
        }
    }

    pub fn run_cmd(&self, cmd: &str) {
        let mut channel = self.session.channel_session().unwrap();
        channel.exec(cmd).unwrap();
        let mut s = String::new();
        channel.read_to_string(&mut s).unwrap();
        println!("{}", s);
        channel.wait_close();
        println!("{}", channel.exit_status().unwrap());
    }

}

impl <'a> SftpClient<'a> {
    pub fn new(ssh_client: &'a SSHClient) -> Self {
        let sftp = ssh_client.session.sftp().unwrap();
        SftpClient {
            sftp: sftp,
            ssh_client: ssh_client,
        }
    }

    pub fn mkdir<T: AsRef<Path>>(&self, path: T){
        // TODO: get mode from src path
        // use std::fs;
        // let permissions = fs::metadata(src_path)?.permissions()
        // let mode = permissions.mode() // return u32
        let mode = 0o755;
        self.sftp.mkdir(path.as_ref(), mode).unwrap();
    }

    pub fn rmdir<T: AsRef<Path>>(&self, path: T){
        self.sftp.rmdir(path.as_ref()).unwrap();
    }

    pub fn create<T: AsRef<Path>>(&self, path: T, content: &String)-> Result<()> {
        self.sftp.create(path.as_ref())?.write_all(content.as_bytes())?;
        Ok(())
    }

    pub fn unlink<T: AsRef<Path>>(&self, path: T) {
        self.sftp.unlink(path.as_ref()).unwrap();
    }

    pub fn rename<T: AsRef<Path>>(&self, src: T, dest: T) {
        self.sftp.rename(src.as_ref(), dest.as_ref(), Some(ssh2::OVERWRITE)).unwrap();
    }

}

pub fn test() {
    let ssh_client = SSHClient::new("192.168.75.129", 22, "ubuntu", Some("nogame"), None::<&Path>);
    ssh_client.run_cmd("ls /tmp");
    let client = SftpClient::new(&ssh_client);
    client.mkdir("/tmp/abc");
}

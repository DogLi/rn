use ssh2;
use ssh2::{Sftp, Session, Error, RenameFlags};
use std::io::prelude::*;
use std::io;
use std::net;
use std::fs::{self, File};
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

    // http://alexcrichton.com/ssh2-rs/ssh2/index.html
    pub fn upload_file<P>(&self, remote_path: &P, local_path: &P) -> Result<()>
    where P: AsRef<Path>{
        let metadata = fs::metadata(local_path)?;
        let mode = metadata.permissions().mode() as i32;
        let size =  metadata.len(); // u64

        let mut contents = String::new();
        File::open(local_path)?.read_to_string(&mut contents)?;
        let mut remote_file = self.ssh_client.session.scp_send(remote_path, mode, size, None)?;
        remote_file.write(contents.as_bytes())?;
        Ok(())
    }

    pub fn mkdir<P: AsRef<Path>>(&self, path: P, mode: i32)-> Result<()>{
        self.sftp.mkdir(path.as_ref(), mode)?;
        Ok(())
    }

    pub fn rmdir<P: AsRef<Path>>(&self, path: P) -> Result<()>{
        self.sftp.rmdir(path.as_ref());
        Ok(())
    }

    pub fn create<P: AsRef<Path>>(&self, path: P, content: &String) -> Result<()> {
        self.sftp.create(path.as_ref())?.write_all(content.as_bytes())?;
        Ok(())
    }

    pub fn unlink<P: AsRef<Path>>(&self, path: P) -> Result<()>{
        self.sftp.unlink(path.as_ref()).unwrap();
        Ok(())
    }

    pub fn rename<P: AsRef<Path>>(&self, src: P, dest: P) -> Result<()>{
        self.sftp.rename(src.as_ref(), dest.as_ref(), Some(ssh2::OVERWRITE))?;
        Ok(())
    }

    pub fn symlink<T: AsRef<Path>>(&self, src: P, dest: P) -> Result<()>{
        self.sftp.symlink(src, dest)?;
        Ok(())
    }

}

pub fn test() {
    let ssh_client = SSHClient::new("ubuntu", 22, "ubuntu", Some("nogame"), None::<&Path>);
    ssh_client.run_cmd("ls /tmp");
    let client = SftpClient::new(&ssh_client);
    client.mkdir("/tmp/abc", 0755);
}

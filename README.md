[![LICENSE](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
# rn
`rn` is a tool used for transforming file/folder to remote server when the file/folder change in real time based on `rsync`. Currently `linux` and `MacOS` support only!

# usage

```
USAGE:
     rn [FLAGS] [OPTIONS] <server>

 FLAGS:
     -h, --help       Prints help information
     -v               see detail information
     -V, --version    Prints version information
     -w, --watch      keep watching for file change!

 OPTIONS:
     -c, --config <config>         Config for rn's variables. [default: ~/bin/settings.toml]
     -i, --indentity <identity>    set ssh identity file path for remote host.
         --log <log>               set log path
         --password <password>     set ssh password for remote host.
         --port <port>             set ssh port for remote host.
     -p, --project <PROJECT>       set the project name to be deployed! [default: default]
         --user <user>             set ssh username for remote host.

 ARGS:
     <server>    set the remote server name which comes from ~/.ssh/config or inner rule.
```

## `-c --config <config>`
The default config file is `~/bin/settings.toml`, which contains the `project` settings and default server asscess key/password. See `example/settings.toml`:

```toml
global_user = "root"
global_key = "~/.ssh/id_rsa"
#global_password = "nogame"

[[projects]]
name = "default"
src = "~/Desktop/default/"
dest = "~/default/"
exclude = [
            ".git",
            ".idea",
            ".vscode",
            "test",
        ]

[[projects]]
name = "test"
src = "~/Desktop/test/"
dest = "~/test/"
exclude = [
            ".git",
            ".idea",
            ".vscode",
        ]
```

* `name`: give a name to a project
* `exclude`: file in exclude list will not be transformed, support `glob` mode such as `*.png`, `a/*/b`
*  `src`: the local folder or file, if folder, it can be ends with `/` or not
* `dest`: the location on the remote server

## `-p, --project <PROJECT> `
`PROJECT` is the project name set in config file, if not set, use the `default` project. For example:

```
rn 10.10.20.1 -p test
rn 10.10.20.1 -p default
rn 10.10.20.1
```


## `<server>`
The server name of you want to transform file to. You can use server name settings in `~/.ssh/config` directly, for example, `~/.ssh/config` contains:

```
# ~/.ssh/config
Host ubuntu
    HostName 192.168.75.129
    User ubuntu
    Port 2222
    PreferredAuthentications publickey
    IdentityFile ~/.ssh/id_rsa
```

or alias `HostName` in `/etc/hosts`

```
# ~/.ssh/config
Host ubuntu
    HostName ubuntu
    User ubuntu
    Port 2222
    PreferredAuthentications publickey
    IdentityFile ~/.ssh/id_rsa
```

```
# /etc/hosts
192.168.75.129 ubuntu
```

You can use `rn ubuntu` directly, `rn` will know how to ssh login `ubuntu`.

You can also set your own rule alias to a server, currently for myself for example is:

```
q123 -> 192.168.1.123
20 -> 10.10.20.20
30.20 -> 10.10.30.20
other_dns -> other_dns
```
Now,  the if remote server is `10.10.20.20`, you can use `rn 20 ` for short, the login username and password/key is set in `global_user` `global_password`/`global_key` in config file.
you can set you own rule in function `get_ip` in `src/utils/sshconfig.rs`

## `-w, --watch`
By default, `rn` will exit after transformed the file. When `-w` set, `rn` will watch file change and transform it to remote server when file changed.

# requirements
You should install `rsync` and `sshpass` on local host and `rsync` on remote host.
```
brew install rsync  
brew install https://raw.githubusercontent.com/kadwanev/bigboybrew/master/Library/Formula/sshpass.rb
```

#

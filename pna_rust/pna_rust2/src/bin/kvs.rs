use clap::{crate_version, Clap};
use kvs::KvStore;
use std::process::exit;

#[derive(Clap)]
#[clap(version = crate_version!())]
struct Opts {
    #[clap(subcommand)]
    command: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    Set(SetCommand),
    Get(GetCommand),
    Rm(RmCommand),
}

#[derive(Clap)]
struct SetCommand {
    key: String,
    value: String,
}

#[derive(Clap)]
struct GetCommand {
    key: String,
}

#[derive(Clap)]
struct RmCommand {
    key: String,
}
fn main() {
    let opts: Opts = Opts::parse();

    use SubCommand::*;
    match opts.command {
        Set(cmd) => {
            let kvs = KvStore::open(".");
            let kvs = kvs.map(|mut x| x.set(cmd.key, cmd.value));
            if kvs.is_err() {
                eprintln!("Error: {:?}", kvs.err());
                exit(1);
            }
            let kvs = kvs.unwrap();
            if kvs.is_err() {
                eprintln!("Error: {:?}", kvs.err());
                exit(1);
            }
        }
        Get(cmd) => {
            let kvs = KvStore::open(".");
            let kvs = kvs.map(|x| x.get(cmd.key));
            if kvs.is_err() {
                eprintln!("Error: {:?}", kvs.err());
                exit(1);
            }
            let kvs = kvs.unwrap();
            if kvs.is_err() {
                eprintln!("Error: {:?}", kvs.err());
                exit(1);
            }
            let kvs = kvs.unwrap();
            if let Some(v) = kvs {
                println!("{}", v);
            } else {
                println!("Key not found");
            }
        }
        Rm(cmd) => {
            let kvs = KvStore::open(".");
            if kvs.is_err() {
                eprintln!("Error: {:?}", kvs.err());
                exit(1);
            }

            let mut kvs = kvs.unwrap();
            let kvs = kvs.remove(cmd.key);
            match kvs {
                Err(kvs::KvError::KeyNoExist) => {
                    println!("Key not found");
                    exit(1);
                }
                Err(e) => {
                    eprintln!("Error: {:?}", e);
                    exit(1);
                }
                Ok(_) => {}
            }
        }
    };
}

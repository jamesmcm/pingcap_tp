use clap::Subcommand;
use clap::{crate_version, Clap};
use kvs::KvStore;
use std::process::exit;
use std::str::FromStr;

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
            eprintln!("unimplemented");
            exit(1);
        }
        Get(cmd) => {
            eprintln!("unimplemented");
            exit(1);
        }
        Rm(cmd) => {
            eprintln!("unimplemented");
            exit(1);
        }
    };
}

use std::io::prelude::*;
use std::fs;
use clap::{Parser, Subcommand};
use flate2::read::ZlibDecoder;
use anyhow::Result;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init,

    CatFile {
        #[arg(short)]
        print: bool,

        #[clap(required = true)]
        blob_sha: String,
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => {
            fs::create_dir(".git").unwrap();
            fs::create_dir(".git/objects").unwrap();
            fs::create_dir(".git/refs").unwrap();
            fs::write(".git/HEAD", "ref: refs/heads/master\n").unwrap();
            println!("Initialized git directory");
        }

        Commands::CatFile { print: _, blob_sha } => {
            let (dir, file) = blob_sha.split_at(2);
            let object = fs::read(format!(".git/objects/{}/{}", dir, file))?;

            let mut z = ZlibDecoder::new(&object[..]);
            let mut s = String::new();
            z.read_to_string(&mut s)?;

            print!("{}", s.split_once('\x00').nth(1).unwrap());
        }
    }

    Ok(())
}

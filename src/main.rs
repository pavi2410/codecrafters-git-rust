use std::io::prelude::*;
use std::fs;
use std::fs::File;
use clap::{Parser, Subcommand};
use flate2::read::ZlibDecoder;
use anyhow::Result;
use sha1::{Sha1, Digest};

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
    },

    HashObject {
        #[arg(short)]
        write: bool,

        #[clap(required = true)]
        filename: String,
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

            print!("{}", s.split('\x00').nth(1).unwrap());
        }

        Commands::HashObject { write, filename } => {
            let content = fs::read(filename)?;

            let mut hasher = Sha1::new();
            hasher.update(&content[..]);
            let sha = hex::encode(hasher.finalize());

            println!("{}", sha);

            if write {
                let (dir, file) = sha.split_at(2);
                let mut file = File::create(format!(".git/objects/{}/{}", dir, file))?;
                file.write_all(&content[..])?;
            }
        }
    }

    Ok(())
}

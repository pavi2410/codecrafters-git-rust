mod utils;
mod git;

use std::io::prelude::*;
use std::fs;
use std::path::Path;
use clap::{Parser, Subcommand};
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use anyhow::Result;
use sha1::{Sha1, Digest};

use crate::git::Object;

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
    },

    LsTree {
        #[arg(long)]
        name_only: bool,

        #[clap(required = true)]
        tree_sha: String,
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
            let obj = Object::read_object(&blob_sha);

            print!("{}", String::from_utf8_lossy(&obj).split('\0').nth(1).unwrap());
        }

        Commands::HashObject { write, filename } => {
            let content = fs::read(filename)?;

            let blob = Object::Blob { data: content };

            println!("{}", blob.sha());

            if write {
                blob.write_object();
            }
        }

        Commands::LsTree { name_only: _, tree_sha } => {
            let obj = Object::parse_from_file(&tree_sha);

            match obj {
                Object::Tree { entries } => {
                    for entry in entries {
                        println!("{}", entry.filename);
                    }
                }
                _ => panic!("Not a tree object"),
            }
        }
    }

    Ok(())
}

use std::io::prelude::*;
use std::fs;
use std::path::Path;
use clap::{Parser, Subcommand};
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
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
            let mut content = fs::read(filename)?;

            let blob = {
                let mut l1 = format!("blob {}\0", content.len()).as_bytes().to_vec();
                l1.append(&mut content);
                l1
            };

            let mut hasher = Sha1::new();
            hasher.update(&blob[..]);
            let sha = hex::encode(hasher.finalize());

            println!("{}", sha);

            if write {
                let (dir, file) = sha.split_at(2);
                let object_filename = format!(".git/objects/{}/{}", dir, file);
                let object_path = Path::new(&object_filename);
                fs::create_dir_all(object_path.parent().unwrap())?;

                let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
                e.write_all(&blob[..])?;
                let compressed = e.finish()?; 
                
                fs::write(object_path, compressed)?;
            }
        }
    }

    Ok(())
}

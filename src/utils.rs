use std::io::{Read, Write};

use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use sha1::{Digest, Sha1};

pub fn sha_from_bytes(bytes: &[u8]) -> String {
    let hash = Sha1::digest(bytes);
    hex::encode(hash)
}

pub fn compress_bytes(bytes: &[u8]) -> Vec<u8> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(bytes).unwrap();
    encoder.finish().unwrap()
}

pub fn decompress_bytes(bytes: &[u8]) -> Vec<u8> {
    let mut decoder = ZlibDecoder::new(bytes);
    let mut bytes = Vec::new();
    decoder.read_to_end(&mut bytes).unwrap();
    bytes
}

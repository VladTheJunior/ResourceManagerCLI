// TODO

// alz4, gzip compession/decompression
// xmb<->xml suppot
// ddt<->tga<->png support
// multithreding extraction
// writing struct to file
// json
// fast hash tool
// code optimization
// test for every shit
use crate::consts::BAR_VERSION_AOE3DE;
mod ddt;

mod bar;
mod loc;
mod consts;

use crate::bar::{
    BarFile,
};
use std::{
    fs,
    env,
    process::Command,
};

use std::{
    error::Error,
    time::{Instant,SystemTime},
};
use crate::ddt::DdtFile;

use crate::consts::BINARY_SIGNATURE_DDT;
use crate::consts::BINARY_SIGNATURE_ALZ4;
use crate::consts::BINARY_SIGNATURE_L33T;

use crate::consts::BINARY_SIGNATURE_WAV_ENCODED;
use crate::consts::BINARY_SIGNATURE_BAR;

use std::io::BufReader;
use std::io::stdin;
use std::fs::File;
use std::io::Read;
use std::io::BufWriter;
use std::io::Seek;
use std::io::Write;
use std::io::SeekFrom;
use std::path::PathBuf;

fn print_help(){
    println!("Resource Manager Command Line Tool v.5.0, developed by © VladTheJunior, 2021");
    println!("Uses given string as path argument and automatically checks it for action:");
    println!("    {:<12} {}", "BAR file", "Extract, decode and convert all entries. Gives info about BAR structure and entries.");
    println!("    {:<12} {}", "Directory", "Archive all items in directory to selected version of BAR file.");
    println!("    {:<12} {}", "XMB file", "Decode and convert it to XML file.");
    println!("    {:<12} {}", "XML file", "Convert and encode it to XMB file.");
    println!("    {:<12} {}", "DDT file", "Decode and convert it to TGA and PNG file.");
    println!("    {:<12} {}", "TGA file", "Convert and decode it to DDT file.");
    println!("    {:<12} {}", "WAV file", "Encode/decode it to decoded/encoded WAV file.");
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

/*
    Managed file structure

    managed:
    |---12:12:2021 12:44:21:
        |---extracted
        |---converted
        |---created
*/

    println!("{:?}", env::current_exe()?.parent().unwrap());

    let start = Instant::now();
    match args.len() {
        2 => {
            let path = PathBuf::from(&args[1]);
            if path.exists() {
                if path.is_dir() {
                    BarFile::create(path, BAR_VERSION_AOE3DE)?;
                }
                else {
                    let file = File::open(&path)?;
                    let mut reader = BufReader::new(file);
                    let mut signature = [0u8; 4];
                    reader.read_exact(&mut signature)?;
                    let signature: u32 = u32::from_le_bytes(signature);
                    reader.seek(SeekFrom::Start(0))?;
                    match signature {
                        BINARY_SIGNATURE_BAR => {
                            let bar = BarFile::open(path)?;
                            bar.extract()?;
                        },
                        BINARY_SIGNATURE_DDT => {
                            let managed_path = env::current_dir()?.join("managed").join("timing");
                            let mut converted_path = managed_path.join("converted").join(path.file_name().unwrap());
                            let prefix = converted_path.parent().unwrap();
                            fs::create_dir_all(prefix)?;
                            let mut data: Vec<u8> = Vec::new();
                            reader.read_to_end(&mut data)?;
                            converted_path.set_extension("tga");
                            let ddt_file = DdtFile::read(&data)?;
                            let tga_file = ddt_file.to_tga()?;
                            tga_file.save(converted_path)?;

                            Command::new("explorer")
                                .arg(managed_path)
                                .spawn()
                                .unwrap();
                        },
                        BINARY_SIGNATURE_WAV_ENCODED => {
                            let managed_path = env::current_dir()?.join("managed").join("timing");
                            let converted_path = managed_path.join("converted").join(path.file_name().unwrap());
                            let prefix = converted_path.parent().unwrap();
                            fs::create_dir_all(prefix)?;
                            let mut data: Vec<u8> = Vec::new();
                            reader.read_to_end(&mut data)?;
                            let decoded_data: Vec<u8> = BarFile::decode_sound(&data)?;
                            let mut writer = BufWriter::new(File::create(&converted_path)?);
                            writer.write(&decoded_data)?; 
                            Command::new("explorer")
                            .arg(managed_path)
                            .spawn()
                            .unwrap(); 
                        },
                        BINARY_SIGNATURE_ALZ4 => {

                        },
                        BINARY_SIGNATURE_L33T => {

                        },
                        _ => {
                            print_help();
                        }
                    }
                }
            }
            else {
                print_help();
            }
        },
        _ => {
            print_help();
        }
    }
    let end = Instant::now();
    println!("Elapsed time: {:?}", end - start);
    println!("{}", "Press any key to exit...");
    let mut line: String = String::new();
    stdin().read_line(&mut line)?;
    Ok(())
}

mod dxt;
mod tga;

use crate::ddt::dxt::DxtImage;
use crate::ddt::tga::TgaFile;

use crate::loc::ERR_NOT_VALID_DDT_SIGNATURE;
use crate::loc::ERR_NOT_VALID_DDT_FORMAT;
use std::fs::File;
use std::{
    cmp,
    error::Error,
};
use std::io::SeekFrom;
use std::io::Cursor;
use std::io::BufReader;
use std::io::Read;
use std::io::Seek;
use std::path::PathBuf;
use crate::consts::DDT_USAGE_CUBE;
use crate::consts::DDT_FORMAT_BGRA;
use crate::consts::BINARY_SIGNATURE_DDT;
use crate::consts::DDT_FORMAT_GREY;
use crate::consts::DDT_FORMAT_DXT1;
use crate::consts::DDT_FORMAT_DXT1DE;
use crate::consts::DDT_FORMAT_DXT3;
use crate::consts::DDT_FORMAT_DXT5;


pub struct DdtFile {
    signature: u32,
    usage: u8,
    alpha: u8,
    format: u8,
    mipmap_levels: u8,
    pub base_width: u32,
    pub base_height: u32,
    images: Vec<DxtImage>,
}

impl DdtFile {

    fn decode(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        let ddt_image = self.images.first().unwrap();
        let decoded_ddt_image: Vec<u8>;

        match self.format {
            DDT_FORMAT_DXT1 | DDT_FORMAT_DXT1DE | DDT_FORMAT_DXT3 | DDT_FORMAT_DXT5 => {
                decoded_ddt_image = ddt_image.decompress(self.format, self.usage)?;
            },
            DDT_FORMAT_BGRA | DDT_FORMAT_GREY => {
                decoded_ddt_image = ddt_image.raw_data.to_vec();
            },
            _ => {
                panic!("{}", ERR_NOT_VALID_DDT_FORMAT);
            }
        }
        return Ok(decoded_ddt_image);
    }


    fn encode(source: &[u8], width: u16, height: u16, usage: u8, format: u8) -> Result<Vec<u8>, Box<dyn Error>> {
        let encoded_ddt_image: Vec<u8>;

        match format {
            DDT_FORMAT_DXT1 | DDT_FORMAT_DXT1DE | DDT_FORMAT_DXT3 | DDT_FORMAT_DXT5 => {
                encoded_ddt_image = DxtImage::compress(source, format, usage, width as u32, height as u32);
            },
            DDT_FORMAT_BGRA | DDT_FORMAT_GREY => {
                encoded_ddt_image = source.to_vec();
            },
            _ => {
                panic!("{}", ERR_NOT_VALID_DDT_FORMAT);
            }
        }
        return Ok(encoded_ddt_image);
    }

    pub fn to_tga(&self) -> Result<TgaFile, Box<dyn Error>> {
        return Ok(TgaFile::new(self.base_width as u16, self.base_height as u16, self.usage, self.alpha, self.format, self.mipmap_levels, self.decode()?));
    }

    pub fn from_tga(path: PathBuf) -> Result<DdtFile, Box<dyn Error>> {
        let tga_file = TgaFile::open(path)?;
        let dxt_image_vec = DdtFile::encode(&tga_file.raw_data, tga_file.image_width, tga_file.image_height, tga_file.image_id[0], tga_file.image_id[2])?;


        return Ok(DdtFile {
            signature: BINARY_SIGNATURE_DDT, 
            usage: tga_file.image_id[0],
            alpha: tga_file.image_id[1],
            format: tga_file.image_id[2],
            mipmap_levels: tga_file.image_id[3], 
            base_height: tga_file.image_height as u32,
            base_width: tga_file.image_width as u32,
            images: vec![DxtImage {
                width: tga_file.image_width as u32, 
                height: tga_file.image_height as u32, 
                offset: 0, 
                length: dxt_image_vec.len() as u32, 
                raw_data: dxt_image_vec
            }]            
        });

    }

    pub fn save(&self, path: PathBuf) -> Result<(), Box<dyn Error>> {
        let mut writer = BufWriter::new(File::create(&path)?);
        writer.write(&self.to_bytes())?;
        return Ok(());
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();

        bytes.extend_from_slice(&self.signature.to_le_bytes());
        bytes.extend_from_slice(&self.usage.to_le_bytes());
        bytes.extend_from_slice(&self.alpha.to_le_bytes());
        bytes.extend_from_slice(&self.format.to_le_bytes());
        bytes.extend_from_slice(&self.mipmap_levels.to_le_bytes());
        bytes.extend_from_slice(&self.base_width.to_le_bytes());
        bytes.extend_from_slice(&self.base_height.to_le_bytes());
        bytes.extend_from_slice(&self.images.to_bytes());
        return bytes;
    }

    pub fn read(data: &[u8]) -> Result<DdtFile, Box<dyn Error>> {
        let mut reader = BufReader::new(Cursor::new(data));
        let mut signature = [0u8; 4];
        reader.read_exact (&mut signature)?;
        let signature: u32 = u32::from_le_bytes(signature);

        assert_eq!(signature, BINARY_SIGNATURE_DDT, "{}", ERR_NOT_VALID_DDT_SIGNATURE);

        let mut usage = [0u8; 1];
        reader.read_exact (&mut usage)?;
        let usage: u8 = u8::from_le_bytes(usage);

        let mut alpha = [0u8; 1];
        reader.read_exact (&mut alpha)?;
        let alpha: u8 = u8::from_le_bytes(alpha);

        let mut format = [0u8; 1];
        reader.read_exact (&mut format)?;
        let format: u8 = u8::from_le_bytes(format);

        let mut mipmap_levels = [0u8; 1];
        reader.read_exact (&mut mipmap_levels)?;
        let mipmap_levels: u8 = u8::from_le_bytes(mipmap_levels);

        let mut base_width = [0u8; 4];
        reader.read_exact (&mut base_width)?;
        let base_width: u32 = u32::from_le_bytes(base_width);

        let mut base_height = [0u8; 4];
        reader.read_exact (&mut base_height)?;
        let base_height: u32 = u32::from_le_bytes(base_height);

        let mut images: Vec<DxtImage> = Vec::new();

        let images_per_level: u32 = if usage & DDT_USAGE_CUBE == DDT_USAGE_CUBE {6} else {1};

        for i in 0..(mipmap_levels as u32) * images_per_level {
            reader.seek(SeekFrom::Start(16 + 8 * (i as u64)))?;
            let width = cmp::max(1, base_width >> (i / (images_per_level as u32)));
            let height = cmp::max(1, base_height >> (i / (images_per_level as u32)));

            let mut offset = [0u8; 4];
            reader.read_exact (&mut offset)?;
            let offset: u32 = u32::from_le_bytes(offset);

            let mut length = [0u8; 4];
            reader.read_exact (&mut length)?;
            let length: u32 = u32::from_le_bytes(length);    
            
            reader.seek(SeekFrom::Start(offset as u64))?;

            let mut raw_data = vec![0u8; length as usize];
            reader.read_exact (&mut raw_data)?;


            let image = DxtImage {
                width: width, 
                height: height, 
                offset: offset, 
                length: length, 
                raw_data: raw_data
            };
            
            images.push(image);
            
        }

        return Ok(DdtFile {
            signature: signature, 
            usage: usage,
            alpha: alpha,
            format: format,
            mipmap_levels: mipmap_levels, 
            base_height: base_height,
            base_width: base_width,
            images: images
        });
        
    }
}


const TEST_DDT_PATH: &str = r"C:\Users\NOKOMPL\Desktop\ResourceManager\managed\timing\extracted\Art\homecity\dutch\sky1.ddt";
#[test]
fn convert_ddt_to_tga() {
    let file = File::open(TEST_DDT_PATH).unwrap();
    let mut reader = BufReader::new(file);
    
    let mut data: Vec<u8> = Vec::new();
    reader.read_to_end(&mut data).unwrap();
    let ddt_file = DdtFile::read(&data).unwrap();
    let tga_file = ddt_file.to_tga().unwrap();
    tga_file.save(PathBuf::from(TEST_DDT_PATH)).unwrap();       
}

const TEST_TGA_PATH: &str = r"C:\Users\NOKOMPL\Desktop\ResourceManager\managed\timing\extracted\Art\homecity\dutch\sky1.(0,0,4,8).tga";
#[test]
fn convert_tga_to_ddt() {
    let ddt_file = DdtFile::from_tga(PathBuf::from(TEST_TGA_PATH)).unwrap();
         
}
 
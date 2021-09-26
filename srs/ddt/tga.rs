use crate::consts::TGA_ALPHA_BITS_MASK;
use crate::consts::TGA_SCREEN_ORIGIN_BIT_MASK;
use crate::consts::TGA_UNCOMPRESSED_TRUE_COLOR;
use crate::consts::DDT_FORMAT_GREY;

use std::{
    error::Error,
    path::{PathBuf},
    fs::{File},
    io::{Write, BufWriter, Read, BufReader},
};

pub struct TgaFile {
    id_length: u8,
    map_type: u8,
    image_type: u8,
    map_origin: u16,
    map_length: u16,
    map_entry_size: u8,
    x_origin: u16,
    y_origin: u16,
    pub image_width: u16,
    pub image_height: u16,
    pixel_depth: u8,
    image_desc: u8,
    pub raw_data: Vec<u8>,
    pub image_id: [u8; 4],
}

impl TgaFile {
    pub fn new(width: u16, height: u16, usage: u8, alpha: u8, format: u8, mipmap_levels: u8, raw_data: Vec<u8>) -> TgaFile{
        let num_alpha_bits: u8;
        let other_channel_bits: u8;
        if format == DDT_FORMAT_GREY {
            num_alpha_bits = 0;
            other_channel_bits = 8;
        }
        else {
            num_alpha_bits = 8;
            other_channel_bits = 24;                
        }

        let pixel_depth: u8 = num_alpha_bits + other_channel_bits;
        let mut image_desc: u8 = num_alpha_bits & TGA_ALPHA_BITS_MASK;
        image_desc |= TGA_SCREEN_ORIGIN_BIT_MASK;

        return TgaFile {
            id_length: 0, 
            map_type: 0, 
            image_type: TGA_UNCOMPRESSED_TRUE_COLOR, 
            map_origin: 0,         
            map_length: 0,
            map_entry_size: 0,
            x_origin: 0,
            y_origin: 0,
            image_width: width as u16,
            image_height: height as u16,
            pixel_depth: pixel_depth,
            image_desc: image_desc,
            raw_data: raw_data,
            image_id: [usage, alpha, format, mipmap_levels]
        };
    }

    pub fn open(path: PathBuf) -> Result<TgaFile, Box<dyn Error>> {
        let file = File::open(&path).expect("Файл не найден");
        let mut reader = BufReader::new(file);  

        let mut id_length = [0u8; 1];
        reader.read_exact (&mut id_length)?;
        let id_length: u8 = u8::from_le_bytes(id_length);

        let mut map_type = [0u8; 1];
        reader.read_exact (&mut map_type)?;
        let map_type: u8 = u8::from_le_bytes(map_type);

        let mut image_type = [0u8; 1];
        reader.read_exact (&mut image_type)?;
        let image_type: u8 = u8::from_le_bytes(image_type);

        let mut map_origin = [0u8; 2];
        reader.read_exact (&mut map_origin)?;
        let map_origin: u16 = u16::from_le_bytes(map_origin);

        let mut map_length = [0u8; 2];
        reader.read_exact (&mut map_length)?;
        let map_length: u16 = u16::from_le_bytes(map_length);

        let mut map_entry_size = [0u8; 1];
        reader.read_exact (&mut map_entry_size)?;
        let map_entry_size: u8 = u8::from_le_bytes(map_entry_size);

        let mut x_origin = [0u8; 2];
        reader.read_exact (&mut x_origin)?;
        let x_origin: u16 = u16::from_le_bytes(x_origin);

        let mut y_origin = [0u8; 2];
        reader.read_exact (&mut y_origin)?;
        let y_origin: u16 = u16::from_le_bytes(y_origin);

        let mut image_width = [0u8; 2];
        reader.read_exact (&mut image_width)?;
        let image_width: u16 = u16::from_le_bytes(image_width);

        let mut image_height = [0u8; 2];
        reader.read_exact (&mut image_height)?;
        let image_height: u16 = u16::from_le_bytes(image_height);

        let mut pixel_depth = [0u8; 1];
        reader.read_exact (&mut pixel_depth)?;
        let pixel_depth: u8 = u8::from_le_bytes(pixel_depth);

        let mut image_desc = [0u8; 1];
        reader.read_exact (&mut image_desc)?;
        let image_desc: u8 = u8::from_le_bytes(image_desc);

        let mut raw_data: Vec<u8> = Vec::new();
        reader.read_to_end(&mut raw_data)?;     



        let file_name = &path.file_name().unwrap().to_str().unwrap().to_owned();
        let splitted_name: Vec<&str> = file_name.split(".").collect();
        if splitted_name.len() != 3 {
            panic!("что то пошло не так");
        }
        let splitted_params: Vec<&str> = splitted_name[1].split(|c| c == ',' || c == '(' || c == ')').collect();
        if splitted_params.len() != 6 {
            panic!("что то пошло не так");
        }    
        
        let usage: u8 = splitted_params[1].parse()?;
        let alpha: u8 = splitted_params[2].parse()?;
        let format: u8 = splitted_params[3].parse()?;
        let mipmap_levels: u8 = splitted_params[4].parse()?;

        return Ok(TgaFile {
            id_length: id_length, 
            map_type: map_type, 
            image_type: image_type, 
            map_origin: map_origin,         
            map_length: map_length,
            map_entry_size: map_entry_size,
            x_origin: x_origin,
            y_origin: y_origin,
            image_width: image_width,
            image_height: image_height,
            pixel_depth: pixel_depth,
            image_desc: image_desc,
            raw_data: raw_data,
            image_id: [usage, alpha, format, mipmap_levels]
        });        
        
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();

        bytes.extend_from_slice(&self.id_length.to_le_bytes());
        bytes.extend_from_slice(&self.map_type.to_le_bytes());
        bytes.extend_from_slice(&self.image_type.to_le_bytes());
        bytes.extend_from_slice(&self.map_origin.to_le_bytes());
        bytes.extend_from_slice(&self.map_length.to_le_bytes());
        bytes.extend_from_slice(&self.map_entry_size.to_le_bytes());
        bytes.extend_from_slice(&self.x_origin.to_le_bytes());
        bytes.extend_from_slice(&self.y_origin.to_le_bytes());
        bytes.extend_from_slice(&self.image_width.to_le_bytes());
        bytes.extend_from_slice(&self.image_height.to_le_bytes());
        bytes.extend_from_slice(&self.pixel_depth.to_le_bytes());
        bytes.extend_from_slice(&self.image_desc.to_le_bytes());
        bytes.extend_from_slice(&self.raw_data);
        return bytes;
    }

    pub fn save(&self, mut path: PathBuf) -> Result<(), Box<dyn Error>> {
        path.set_extension("");
        let file_name = path.file_name().unwrap().to_str().unwrap().to_owned();
        path.set_file_name(file_name + ".(" + &self.image_id[0].to_string() + "," + &self.image_id[1].to_string() + "," + &self.image_id[2].to_string() + "," + &self.image_id[3].to_string() + ").tga");
        let mut writer = BufWriter::new(File::create(&path)?);
        writer.write(&self.to_bytes())?;
        return Ok(());
    }

}




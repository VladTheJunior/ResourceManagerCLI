use crate::ddt::DdtFile;

use crate::consts::BINARY_SIGNATURE_DDT;
use crate::consts::BINARY_BAR_MAGIC;
use crate::consts::BAR_VERSION_AOE3DE;
use crate::consts::BAR_VERSION_AOE3;
use crate::consts::BINARY_SIGNATURE_ALZ4;
use crate::consts::BINARY_SIGNATURE_L33T;
use crate::consts::BINARY_SIGNATURE_WAV_DECODED;
use crate::consts::BINARY_SIGNATURE_WAV_ENCODED;
use crate::consts::BINARY_SIGNATURE_BAR;

use crate::loc::ERR_NOT_MATCHED_ENTRY_COUNT;
use crate::loc::ERR_NOT_VALID_BAR_MAGIC;
use crate::loc::ERR_NOT_SUPPORTED_BAR_VERSION;
use crate::loc::ERR_NOT_VALID_BAR_SIGNATURE;
use crate::loc::ERR_BAR_NOT_FOUND;
use crate::loc::ERR_NOT_VALID_DECODED_WAV_SIGNATURE;
use std::process::Command;
use std::{
    env,
    thread,
    error::Error,
    path::{PathBuf},
    time::{Instant,SystemTime},
    convert::TryInto,
    fs::{self, File},
    io::{self, Read, Write, BufReader, BufWriter, SeekFrom, Seek},
};



pub struct BarFile {
    pub bar_path: PathBuf, // path to opened bar file []
    signature: u32,    // signature [de, legacy]
    pub version: u32, // version of bar file [de, legacy]
    magic: u32, // magic number 1 [de, legacy]
    unk1: [u8; 66 * 4], // 264 zero bytes [de, legacy]
    unk2: u32, // ? [de, legacy]
    pub file_count: u32, // count of files in bar file [de, legacy]
    unk3: u32, // ? [de]
    pub files_table_offset: u64, // offset. u64 for DE and u32 for legacy [de, legacy]
    unk4: u32, // ? [de, legacy]
    unk5: u32, // ? [de]
    root_path_length: u32, // length of path name [de, legacy]
    pub root_path: Vec<u8>, // root path [de, legacy]
    root_file_count: u32, // count of files in root. should be same as file_count [de, legacy]
    pub entries: Vec<BarEntry>, // bar entries [de, legacy]
}

#[derive(Clone)]
pub struct BarEntry {
    //pub bar_path: PathBuf, // path to opened bar file []
    //pub root_path: Vec<u8>, // path to root directory of entry []
    pub offset: u64, // offset. u64 for DE and u32 for legacy [de, legacy]
    file_size1: u32, // uncompressed size [de, legacy]
    pub file_size2: u32, // real size of binary [de, legacy]
    file_size3: u32, // some dublicate size [de]
    year: u16, // last write time [legacy]
    month: u16,
    day_of_week: u16,
    day: u16,
    hour: u16,
    minute: u16,
    second: u16,
    msecond: u16,
    file_name_length: u32, // entry file name length [de, legacy]
    pub file_name: Vec<u8>, // entry file name [de, legacy]
    pub is_encoded: u32, // type of encoding [de]
}

impl BarEntry {
    pub fn get_path(&self, managed_path: &PathBuf, root_path: &[u8]) -> Result<PathBuf, Box<dyn Error>> {
        return Ok(managed_path.join(&BarFile::vec_u8_to_string_u16(root_path)?).join(BarFile::vec_u8_to_string_u16(&self.file_name)?));
    }
}

struct RawBarEntry{
    size: u64,
    path: PathBuf,
    modified_datetime: SystemTime,
}

fn get_raw_bar_entries_in_directory(dir: &PathBuf) -> Result<Vec<RawBarEntry>, Box<dyn Error>> {
    let items = fs::read_dir(dir)?
    .map(|res| res.map(|e| e.path()))
    .collect::<Result<Vec<_>, io::Error>>()?;

    let mut raw_entries: Vec<RawBarEntry> = Vec::new();
    for item in items{
        if item.is_dir(){
            raw_entries.append(&mut get_raw_bar_entries_in_directory(&item)?);
        }
        else{
            raw_entries.push(RawBarEntry {
                size: item.metadata()?.len(), 
                modified_datetime: item.metadata()?.modified()?, 
                path: item 
            });
        }
    }
    return Ok(raw_entries);
}

pub fn get_file_signature(source: &[u8], size: usize) -> u32 {
    let mut data = [0u8; 4];
    data.copy_from_slice(&source[0..size]);
    return u32::from_le_bytes(data);
}


impl BarFile{
    // convert Vec<u8> to utf-16-le string
    fn vec_u8_to_string_u16(source: &[u8]) -> Result<String, Box<dyn Error>> {
        let mut dest = vec![0u16; 0];
        for i in 0..source.len() / 2 {
            let n: u16 = ((source[2 * i + 1] as u16) << 8) | source[2 * i] as u16;
            dest.push(n);
        }

        Ok(String::from_utf16(&dest)?)
    }

    // decode encrypted sound files
    pub fn decode_sound(source: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {

        let qword8: u64 = 0x23966BA95E28C33F;
        let qword10: u64 = 0x39BAE3441DB35873;
        let mut qword18: u64 = 0x2AF92545ADDE0B65;

        let mut decoded_data:Vec<u8> = Vec::new();
        let non_padded_size = source.len();
        let padding_length = (8 - non_padded_size % 8) % 8;
        let padded_data = &mut source.to_vec();

        if padding_length > 0 {
            padded_data.resize(non_padded_size + padding_length, 0);
        }
        let mut padded_data = &padded_data[..];

        while padded_data.len() >= 8 {
            let mut buf = padded_data[..8].try_into()?;         
            let buf_value: u64 = u64::from_le_bytes(buf);    
            
            qword18 = (qword10.wrapping_mul(qword18.wrapping_add(qword8))).rotate_left(32);
            buf = (buf_value ^ qword18).to_le_bytes();

            decoded_data.extend_from_slice(&buf);
            padded_data = &padded_data[8..];   
        }

        decoded_data.resize(non_padded_size, 0);
        let signature = decoded_data[..4].try_into()?;
        let signature:u32 = u32::from_le_bytes(signature); 
        assert_eq!(signature, BINARY_SIGNATURE_WAV_DECODED, "{}", ERR_NOT_VALID_DECODED_WAV_SIGNATURE);
        return Ok(decoded_data);
    }

    /*pub fn multiextract(&self) -> Result<(), Box<dyn Error>> {
        // multithreading extraction
        let mut threads = vec![];  
        let entries_chunked: Vec<Vec<BarEntry>> = self.entries.chunks(self.file_count as usize / 512).map(|x| x.to_vec()).collect();
        
        for chunk in entries_chunked{
            threads.push(thread::spawn(move || {
                for entry in chunk {    
                    let file = File::open(PathBuf::from(&entry.bar_path)).expect(ERR_BAR_NOT_FOUND);
                    let mut reader = BufReader::new(file);               

                    reader.seek(SeekFrom::Start(entry.offset)).unwrap();
                    let mut fdata = vec![0u8; entry.file_size2 as usize];
                    reader.read_exact (&mut fdata).unwrap();
                    let mut fpath = entry.get_path().unwrap();
                    let prefix = fpath.parent().unwrap();
                    fs::create_dir_all(prefix).unwrap();
                    let signature = get_file_signature(&fdata, 4);
                    // better to check signature, not is_encoded
                    match entry.is_encoded {
                        0 => {
                            let mut writer = BufWriter::new(File::create(&fpath).unwrap());
                            writer.write(&fdata).unwrap();
                        },
                        1 => {

                        },
                        2 => {
                            
                            if signature == BINARY_SIGNATURE_WAV_ENCODED {
                                let decoded_data: Vec<u8> = BarFile::decode_sound(&fdata).unwrap();
                                let mut writer = BufWriter::new(File::create(&fpath).unwrap());
                                writer.write(&decoded_data).unwrap();
                            }
                        },
                        _ => (),
                    }   
                    
                    if signature == BINARY_SIGNATURE_DDT {
                        fpath.set_extension("tga");
                        let ddt_file = DdtFile::new(&fdata).unwrap();
                        let tga_file = ddt_file.to_tga().unwrap();
                        tga_file.save(fpath).unwrap();
                    }
                }
            }));
        }
        // run threads
        for thread in threads{
            thread.join();
        }
        return Ok(());
    }*/

    pub fn extract(&self) -> Result<(), Box<dyn Error>> {
        let file = File::open(PathBuf::from(&self.bar_path)).expect(ERR_BAR_NOT_FOUND);
        let mut reader = BufReader::new(file);    

        let managed_path = env::current_dir()?.join("managed").join("timing");
        let extracted_path = managed_path.join("extracted");
        let converted_path = managed_path.join("converted");

        self.to_csv(&extracted_path)?;

        for entry in &self.entries {    
           

            reader.seek(SeekFrom::Start(entry.offset))?;
            let mut data = vec![0u8; entry.file_size2 as usize];
            reader.read_exact (&mut data)?;
            let extracted_entry_path = entry.get_path(&extracted_path, &self.root_path)?;
            let prefix = extracted_entry_path.parent().unwrap();
            fs::create_dir_all(prefix)?;
            let signature = get_file_signature(&data, 4);

            let mut writer = BufWriter::new(File::create(&extracted_entry_path)?);
            writer.write(&data)?;
            // better to check signature, not is_encoded
            match entry.is_encoded {
                1 => {

                },
                2 => {
                    if signature == BINARY_SIGNATURE_WAV_ENCODED {
                        let converted_entry_path = entry.get_path(&converted_path, &self.root_path)?;
                        let prefix = converted_entry_path.parent().unwrap();
                        fs::create_dir_all(prefix)?;
                        let decoded_data: Vec<u8> = BarFile::decode_sound(&data)?;
                        let mut writer = BufWriter::new(File::create(&converted_entry_path)?);
                        writer.write(&decoded_data)?;
                    }
                },
                _ => (),
            }   
            
            if signature == BINARY_SIGNATURE_DDT {
                let mut converted_entry_path = entry.get_path(&converted_path, &self.root_path)?;
                let prefix = converted_entry_path.parent().unwrap();
                fs::create_dir_all(prefix)?;
                converted_entry_path.set_extension("tga");
                let ddt_file = DdtFile::read(&data)?;
                let tga_file = ddt_file.to_tga()?;
                tga_file.save(converted_entry_path)?;
            }
        }
        Command::new("explorer")
            .arg(managed_path)
            .spawn()
            .unwrap();
        return Ok(());
    }

    fn to_csv(&self, dest: &PathBuf) -> Result<(), Box<dyn Error>>{
        fs::create_dir_all(dest)?;
        let path = PathBuf::from(dest).join("__entries.csv");
        let mut file = BufWriter::new(File::create(path)?);
        let csv_delimiter = "\t";
        file.write_all(b"#")?;
        file.write_all(csv_delimiter.as_bytes())?;
        file.write_all(b"\"file_name\"")?;
        file.write_all(csv_delimiter.as_bytes())?;
        file.write_all(b"\"file_size\"")?;

        for (i, entry) in self.entries.iter().enumerate() {
            file.write_all((i + 1).to_string().as_bytes())?;
            file.write_all(csv_delimiter.as_bytes())?;
            file.write_all(BarFile::vec_u8_to_string_u16(&entry.file_name)?.as_bytes())?;
            file.write_all(csv_delimiter.as_bytes())?;
            file.write_all(&entry.file_size2.to_string().as_bytes())?;
            if i != self.entries.len() - 1 {
                file.write_all(csv_delimiter.as_bytes())?;
                file.write_all(b"\n")?;
            }
        }
        return Ok(());
    }

   
    pub fn open(path: PathBuf) -> Result<BarFile, Box<dyn Error>> {
        let from_dump: bool = false;
        let file = File::open(&path).expect(&ERR_BAR_NOT_FOUND);
        let mut reader = BufReader::new(file);

        let mut signature = [0u8; 4];
        reader.read_exact (&mut signature)?;
        let signature: u32 = u32::from_le_bytes(signature);
        assert_eq!(signature, BINARY_SIGNATURE_BAR, "{}", ERR_NOT_VALID_BAR_SIGNATURE);

        let mut version = [0u8; 4];
        reader.read_exact (&mut version)?;
        let version: u32 = u32::from_le_bytes(version);

        if version != BAR_VERSION_AOE3DE && version != BAR_VERSION_AOE3 {
            panic!("{}", ERR_NOT_SUPPORTED_BAR_VERSION);
        }

        let mut magic = [0u8; 4];
        reader.read_exact (&mut magic)?;
        let magic: u32 = u32::from_le_bytes(magic);
        assert_eq!(magic, BINARY_BAR_MAGIC, "{}", ERR_NOT_VALID_BAR_MAGIC);

        let mut unk1 = [0u8; 264];
        reader.read_exact (&mut unk1)?;


        let mut unk2 = [0u8; 4];
        reader.read_exact (&mut unk2)?;
        let unk2: u32 = u32::from_le_bytes(unk2);

        let mut file_count = [0u8; 4];
        reader.read_exact (&mut file_count)?;
        let file_count: u32 = u32::from_le_bytes(file_count);

        let files_table_offset:u64;
        let mut unk3: u32 = 0;
        if version == BAR_VERSION_AOE3 {
            let mut _files_table_offset = [0u8; 4];
            reader.read_exact (&mut _files_table_offset)?;
            files_table_offset = u32::from_le_bytes(_files_table_offset) as u64;
        }
        else{
            let mut _unk3 = [0u8; 4];
            reader.read_exact (&mut _unk3)?;
            unk3 = u32::from_le_bytes(_unk3);

            let mut _files_table_offset = [0u8; 8];
            reader.read_exact (&mut _files_table_offset)?;
            files_table_offset = u64::from_le_bytes(_files_table_offset);           
        }
        let mut unk4 = [0u8; 4];
        reader.read_exact (&mut unk4)?;
        let unk4:u32 = u32::from_le_bytes(unk4);
        let mut unk5: u32 = 0;
        if version == BAR_VERSION_AOE3DE {
            let mut _unk5 = [0u8; 4];
            reader.read_exact (&mut _unk5)?;
            unk5 = u32::from_le_bytes(_unk5);                
        }

        if from_dump == false{
            reader.seek(SeekFrom::Start(files_table_offset))?;
        }
        let mut root_path_length = [0u8; 4];
        reader.read_exact (&mut root_path_length)?;
        let root_path_length: u32 = u32::from_le_bytes(root_path_length);
        let mut root_path = vec![0u8; root_path_length as usize * 2];
        reader.read_exact (&mut root_path)?;
        let mut root_file_count = [0u8; 4];
        reader.read_exact (&mut root_file_count)?;
        let root_file_count: u32 = u32::from_le_bytes(root_file_count);

        assert_eq!(file_count, root_file_count, "{}", ERR_NOT_MATCHED_ENTRY_COUNT);

        let mut entries: Vec<BarEntry> = Vec::new();
        for _ in 0..root_file_count {

            let offset: u64;
            if version == BAR_VERSION_AOE3 {
                let mut _offset = [0u8; 4];
                reader.read_exact (&mut _offset)?;
                offset = u32::from_le_bytes(_offset) as u64;
            }
            else {
                let mut _offset = [0u8; 8];
                reader.read_exact (&mut _offset)?;
                offset = u64::from_le_bytes(_offset);
            }
            let mut fsize1 = [0u8; 4];
            reader.read_exact (&mut fsize1)?;
            let fsize1: u32 = u32::from_le_bytes(fsize1);

            let mut fsize2 = [0u8; 4];
            reader.read_exact (&mut fsize2)?;
            let fsize2: u32 = u32::from_le_bytes(fsize2);
            
            let mut fsize3: u32 = 0;

            let mut year: u16 = 0;
            let mut month: u16 = 0;  
            let mut day_of_week: u16 = 0;
            let mut day: u16 = 0; 
            let mut hour: u16 = 0;
            let mut minute: u16 = 0;
            let mut second: u16 = 0;  
            let mut msecond: u16 = 0;

            if version == BAR_VERSION_AOE3DE {
                let mut _fsize3 = [0u8; 4];
                reader.read_exact (&mut _fsize3)?;
                fsize3 = u32::from_le_bytes(_fsize3); 
            }
            else {
                let mut _year = [0u8; 2];
                reader.read_exact (&mut _year)?;
                year = u16::from_le_bytes(_year); 
                
                let mut _month = [0u8; 2];
                reader.read_exact (&mut _month)?;
                month = u16::from_le_bytes(_month); 

                let mut _day_of_week = [0u8; 2];
                reader.read_exact (&mut _day_of_week)?;
                day_of_week = u16::from_le_bytes(_day_of_week); 

                let mut _day = [0u8; 2];
                reader.read_exact (&mut _day)?;
                day = u16::from_le_bytes(_day); 

                let mut _hour = [0u8; 2];
                reader.read_exact (&mut _hour)?;
                hour = u16::from_le_bytes(_hour); 

                let mut _minute = [0u8; 2];
                reader.read_exact (&mut _minute)?;
                minute = u16::from_le_bytes(_minute); 

                let mut _second = [0u8; 2];
                reader.read_exact (&mut _second)?;
                second = u16::from_le_bytes(_second); 

                let mut _msecond = [0u8; 2];
                reader.read_exact (&mut _msecond)?;
                msecond = u16::from_le_bytes(_msecond); 
            }
            
            let mut flength = [0u8; 4];
            reader.read_exact (&mut flength)?;
            let flength: u32 = u32::from_le_bytes(flength); 
            
            let mut fname = vec![0u8; (flength * 2).try_into()?];
            reader.read_exact (&mut fname)?;
            let mut is_encoded: u32 = 0;
            if version == BAR_VERSION_AOE3DE {
                let mut _is_encoded = [0u8; 4];
                reader.read_exact (&mut _is_encoded)?;
                is_encoded = u32::from_le_bytes(_is_encoded);   
            }
            let entry = BarEntry {
                //bar_path: path, 
                //: root_path.clone(),
                offset: offset, 
                file_size1: fsize1, 
                file_size2: fsize2, 
                file_size3: fsize3, 
                year: year, 
                month: month, 
                day_of_week: day_of_week, 
                day: day, 
                hour: hour, 
                minute: minute, 
                second: second, 
                msecond: msecond,
                file_name_length: flength, 
                file_name: fname, 
                is_encoded: is_encoded
            };
            
            entries.push(entry);
        }

        return Ok(BarFile { 
            bar_path: path,         
            signature: signature,
            version: version,
            magic: magic,
            unk1: unk1,
            unk2: unk2,
            file_count: file_count,
            unk3: unk3,
            files_table_offset: files_table_offset,
            unk4: unk4,
            unk5: unk5,
            root_path_length: root_path_length,
            root_path: root_path,
            root_file_count: root_file_count,
            entries: entries,   
        });
    }

    pub fn create(dir: PathBuf, version: u32) -> Result<BarFile, Box<dyn Error>> {

        let managed_path = env::current_dir()?.join("managed").join("timing");
        let created_path = managed_path.join("created");
        fs::create_dir_all(&created_path)?;
        let mut bar_path = created_path.join(dir.file_name().unwrap());
        bar_path.set_extension("bar");

        let files = get_raw_bar_entries_in_directory(&dir)?;
        let files_count: u32 = files.len() as u32;

        let mut size: u64 = 0;
        for file in &files {
            size += file.size;
        }

        let file = File::create(&bar_path)?;
        let mut writer = BufWriter::new(file);

        writer.write_all(&BINARY_SIGNATURE_BAR.to_le_bytes())?;
        writer.write_all(&version.to_le_bytes())?;
        writer.write_all(&BINARY_BAR_MAGIC.to_le_bytes())?;
        writer.write_all(&[0u8; 264])?;
        writer.write_all(&[0u8; 4])?;
        writer.write_all(&files_count.to_le_bytes())?;
        let files_table_offset: u64;
        if version == BAR_VERSION_AOE3DE {
            writer.write_all(&[0u8; 4])?;
            files_table_offset = size + 304;
            writer.write_all(&files_table_offset.to_le_bytes())?;
        }
        else {
            files_table_offset = size + 292;
            writer.write_all(&(files_table_offset as u32).to_le_bytes())?;
        }
        writer.write_all(&[0u8; 4])?;

        if version == BAR_VERSION_AOE3DE {
            writer.write_all(&[0u8; 4])?;
        }

        let start_offset: u64 = writer.seek(SeekFrom::Current(0))?;
        let mut is_encoded_vec: Vec<u32> = Vec::new();
        for f in &files {
            
            let file = File::open(&f.path)?;
            let mut reader = BufReader::new(file);
            
            let mut data: Vec<u8> = Vec::new();
            reader.read_to_end(&mut data)?;

            let is_encoded = data[..4].try_into()?;
            let is_encoded: u32 = u32::from_le_bytes(is_encoded); 
            match is_encoded {
                BINARY_SIGNATURE_ALZ4 => {
                    is_encoded_vec.push(1);
                },
                BINARY_SIGNATURE_L33T => {
                    is_encoded_vec.push(1);
                },
                BINARY_SIGNATURE_WAV_ENCODED => {
                    is_encoded_vec.push(2);
                },
                _ => {
                    is_encoded_vec.push(0);
                },
            }
            
            writer.write_all(&data)?;
        }
        let root_path = dir.file_name().unwrap().to_str().unwrap().to_owned() + "\\";



        let root_path_len: u32 = root_path.len() as u32;
        
        writer.write_all(&root_path_len.to_le_bytes())?;
        let root_path_vec_16: Vec<u16> = root_path.encode_utf16().collect();
        let mut root_path_vec_8: Vec<u8> = Vec::new();
        for u16_byte in root_path_vec_16 {
            root_path_vec_8.append(&mut u16_byte.to_le_bytes().to_vec());         
        }
        writer.write_all(&root_path_vec_8)?;

        writer.write_all(&files_count.to_le_bytes())?;

        let mut offset: u64 = start_offset;

        let mut entries: Vec<BarEntry> = Vec::new();
        for (i, file) in files.iter().enumerate() {
            let file_size = file.size as u32;
            if version == BAR_VERSION_AOE3 {
                writer.write_all(&(offset as u32).to_le_bytes())?;
                writer.write_all(&file_size.to_le_bytes())?;
                writer.write_all(&file_size.to_le_bytes())?;
                

                // NEED TO KNOW DATA TIME
                writer.write_all(&[0u8; 2])?;
                writer.write_all(&[0u8; 2])?;
                writer.write_all(&[0u8; 2])?;
                writer.write_all(&[0u8; 2])?;
                writer.write_all(&[0u8; 2])?;
                writer.write_all(&[0u8; 2])?;
                writer.write_all(&[0u8; 2])?;
                writer.write_all(&[0u8; 2])?;
            }
            else{
                writer.write_all(&offset.to_le_bytes())?;
                //check if alz4 or l33t (and maybe sound?) to determine uncomprerssed file size
                if is_encoded_vec[i] == 1 {

                }
                else{

                }
    
                writer.write_all(&[0u8; 4])?;

                writer.write_all(&file_size.to_le_bytes())?;
                writer.write_all(&file_size.to_le_bytes())?;
            }
            let relative_file_path: String = file.path.strip_prefix(&dir)?.display().to_string();
            let relative_file_path_len: u32 = relative_file_path.len() as u32;
            writer.write_all(&relative_file_path_len.to_le_bytes())?;
            let relative_file_path_vec_16: Vec<u16> = relative_file_path.encode_utf16().collect();
            let mut relative_file_path_vec_8: Vec<u8> = Vec::new();
            for u16_byte in relative_file_path_vec_16 {
                relative_file_path_vec_8.append(&mut u16_byte.to_le_bytes().to_vec());             
            }
            writer.write_all(&relative_file_path_vec_8)?;

            if version == BAR_VERSION_AOE3DE {
                writer.write_all(&is_encoded_vec[i].to_le_bytes())?;  
            }

            // replace zero to values
            let entry = BarEntry {
                //bar_path: bar_path,   
                //root_path: root_path_vec_8.clone(),
                offset: offset, 
                file_size1: 0, 
                file_size2: file_size, 
                file_size3: file_size, 
                year: 0, 
                month: 0, 
                day_of_week: 0, 
                day: 0, 
                hour: 0, 
                minute: 0, 
                second: 0, 
                msecond: 0,
                file_name_length: relative_file_path_len, 
                file_name: relative_file_path_vec_8, 
                is_encoded: is_encoded_vec[i]
            };
            
            entries.push(entry);

            offset += file_size as u64;

        }

        let bar = BarFile { 
            bar_path: bar_path,         
            signature: BINARY_SIGNATURE_BAR,
            version: version,
            magic: BINARY_BAR_MAGIC,
            unk1: [0u8; 264],
            unk2: 0,
            file_count: files_count,
            unk3: 0,
            files_table_offset: files_table_offset,
            unk4: 0,
            unk5: 0,
            root_path_length: root_path_len,
            root_path: root_path_vec_8,
            root_file_count: files_count,
            entries: entries,   
        };

        bar.to_csv(&created_path)?;

        return Ok(bar);
    }
}


const TEST_BAR_DE_PATH: &str = "C:\\Users\\NOKOMPL\\Desktop\\TestFiles\\art4.bar";
const TEST_BAR_LEGACY_PATH: &str = "C:\\Users\\NOKOMPL\\Desktop\\ResourceManager\\test.bar";
const TEST_ENCODED_SOUND_PATH: &str = r"C:\Users\NOKOMPL\Desktop\TestFiles\alainmagnanattack1_de.wav";

const TEST_DIR_DE_PATH: &str = "C:\\Users\\NOKOMPL\\Desktop\\ResourceManager\\test_de";
const TEST_DIR_LEGACY_PATH: &str = "C:\\Users\\NOKOMPL\\Desktop\\ResourceManager\\test_legacy";


#[test]
#[ignore]
fn create_de_bar_file(){
    BarFile::create(PathBuf::from(TEST_DIR_DE_PATH), BAR_VERSION_AOE3DE).unwrap();          
}

#[test]
#[ignore]
fn create_legacy_bar_file(){
    BarFile::create(PathBuf::from(TEST_DIR_LEGACY_PATH), BAR_VERSION_AOE3).unwrap();          
}

#[test]
#[ignore]
fn open_de_bar_file(){
    BarFile::open(PathBuf::from(TEST_BAR_DE_PATH)).unwrap();          
}

#[test]
#[ignore]
fn open_legacy_bar_file(){
    BarFile::open(PathBuf::from(TEST_BAR_LEGACY_PATH)).unwrap();          
}

#[test]
#[ignore]
fn decode_sound_file(){
    let file = File::open(TEST_ENCODED_SOUND_PATH).unwrap();
    let mut reader = BufReader::new(file);
    let mut data: Vec<u8> = Vec::new();
    reader.read_to_end (&mut data).unwrap();
    BarFile::decode_sound(&data).unwrap();   
}

#[test]
#[ignore]
fn extract_de_bar_file(){
    let bar = BarFile::open(PathBuf::from(TEST_BAR_DE_PATH)).unwrap();  
    bar.extract().unwrap();  
}

#[test]
#[ignore]
fn extract_legacy_bar_file(){
    let bar = BarFile::open(PathBuf::from(TEST_BAR_LEGACY_PATH)).unwrap(); 
    bar.extract().unwrap();         
}
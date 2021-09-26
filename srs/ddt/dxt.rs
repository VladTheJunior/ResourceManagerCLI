use crate::consts::DDT_FORMAT_DXT5;
use crate::consts::DDT_USAGE_BUMP;
use crate::consts::DDT_FORMAT_DXT1;
use crate::consts::DDT_FORMAT_DXT1DE;
use crate::consts::DDT_FORMAT_DXT3;

use std::io::Cursor;
use std::io::BufReader;
use std::io::Read;
use std::error::Error;
use std::mem::swap;

pub struct DxtImage {
    pub width: u32,
    pub height: u32,
    pub offset: u32,
    pub length: u32,
    pub raw_data: Vec<u8>,
}

type Rgb = [u8; 3];

impl DxtImage {
    // Convert rgb 5,6,5 bytes to rgb 8,8,8 bytes
    fn rgb565_to_rgb888(color: u16, r: &mut u8, g: &mut u8, b: &mut u8){
        let mut temp = (color >> 11) * 255 + 16;
        *r = ((temp / 32 + temp) / 32) as u8;
        temp = ((color & 0x07E0) >> 5) * 255 + 32;
        *g = ((temp / 64 + temp) / 64) as u8;
        temp = (color & 0x001F) * 255 + 16;
        *b = ((temp / 32 + temp) / 32) as u8;
    }
    
    // decompress DXT data
    pub fn decompress(&self, format: u8, usage:u8) -> Result<Vec<u8>, Box<dyn Error>> {
        let block_count_x = (self.width + 3 ) / 4;
        let block_count_y = (self.height + 3) / 4;
        let mut image_data = vec![0u8; (self.width * self.height * 4) as usize];
        let mut reader = BufReader::new(Cursor::new(&self.raw_data));
        for y in 0..block_count_y {
            for x in 0..block_count_x {
                let mut alpha = [0u8; 8];
                if format == DDT_FORMAT_DXT3 || format == DDT_FORMAT_DXT5 {                     
                    reader.read_exact (&mut alpha)?;
                }
                let mut c0 = [0u8; 2];
                reader.read_exact (&mut c0)?;
                let c0:u16 = u16::from_le_bytes(c0);
                
                let mut c1 = [0u8; 2];
                reader.read_exact (&mut c1)?;
                let c1:u16 = u16::from_le_bytes(c1);
    
                let mut lookup_table = [0u8; 4];
                reader.read_exact (&mut lookup_table)?;
                let lookup_table:u32 = u32::from_le_bytes(lookup_table);
                DxtImage::decomprerss_dxt_block(format, usage, &alpha, c0, c1, lookup_table, x, y, &mut image_data, self.width, self.height);
            }
        }
            return Ok(image_data);
    }
    
    fn decomprerss_dxt_block(format: u8, usage: u8, alpha: &[u8], c0: u16, c1: u16, lookup_table: u32, x: u32, y: u32, data: &mut Vec<u8>, width: u32, height: u32) {
        
        let mut r0: u8 = 0;
        let mut r1: u8 = 0;
    
        let mut g0: u8 = 0;
        let mut g1: u8 = 0;
    
        let mut b0: u8 = 0;
        let mut b1: u8 = 0;
    
        DxtImage::rgb565_to_rgb888(c0,  &mut r0, &mut g0, &mut b0);
        DxtImage::rgb565_to_rgb888(c1,  &mut r1, &mut g1, &mut b1);
    
        let mut alpha_index: u32 = 0;
        let mut alpha_mask: u64 = 0;
        if format == DDT_FORMAT_DXT5{
            alpha_mask = alpha[2] as u64;
            alpha_mask += (alpha[3] as u64) << 8;
            alpha_mask += (alpha[4] as u64) << 16;
            alpha_mask += (alpha[5] as u64) << 24;
            alpha_mask += (alpha[6] as u64) << 32;
            alpha_mask += (alpha[7] as u64) << 40;
        }

        for block_y in 0..4 {
            for block_x in 0..4 {
                let mut r: u8 = 0;
                let mut g: u8 = 0;
                let mut b: u8 = 0;
                let mut a: u8 = 0;

                if format == DDT_FORMAT_DXT1 || format == DDT_FORMAT_DXT1DE{
                    a = 255;
                }
    
                let index = (lookup_table >> (2 * (4 * block_y+block_x))) & 0x03;
                if format == DDT_FORMAT_DXT3{
                    match alpha_index{
                        0 => {
                            a = (alpha[0] & 0x0F) | ((alpha[0] & 0x0F) << 4);
                        },
                        1 => {
                            a = (alpha[0] & 0xF0) | ((alpha[0] & 0xF0) >> 4);
                        },
                        2 => {
                            a = (alpha[1] & 0x0F) | ((alpha[1] & 0x0F) << 4);
                        },
                        3 => {
                            a = (alpha[1] & 0xF0) | ((alpha[1] & 0xF0) >> 4);
                        },
                        4 => {
                            a = (alpha[2] & 0x0F) | ((alpha[2] & 0x0F) << 4);
                        },
                        5 => {
                            a = (alpha[2] & 0xF0) | ((alpha[2] & 0xF0) >> 4);
                        },
                        6 => {
                            a = (alpha[3] & 0x0F) | ((alpha[3] & 0x0F) << 4);
                        },
                        7 => {
                            a = (alpha[3] & 0xF0) | ((alpha[3] & 0xF0) >> 4);
                        },
                        8 => {
                            a = (alpha[4] & 0x0F) | ((alpha[4] & 0x0F) << 4);
                        },
                        9 => {
                            a = (alpha[4] & 0xF0) | ((alpha[4] & 0xF0) >> 4);
                        },
                        10 => {
                            a = (alpha[5] & 0x0F) | ((alpha[5] & 0x0F) << 4);
                        },
                        11 => {
                            a = (alpha[5] & 0xF0) | ((alpha[5] & 0xF0) >> 4);
                        },
                        12 => {
                            a = (alpha[6] & 0x0F) | ((alpha[6] & 0x0F) << 4);
                        },
                        13 => {
                            a = (alpha[6] & 0xF0) | ((alpha[6] & 0xF0) >> 4);
                        },
                        14 => {
                            a = (alpha[7] & 0x0F) | ((alpha[7] & 0x0F) << 4);
                        },
                        15 => {
                            a = (alpha[7] & 0xF0) | ((alpha[7] & 0xF0) >> 4);
                        },
                        _ =>{}
                    }
                    alpha_index += 1;
                }
    
                if format == DDT_FORMAT_DXT5 {
                    alpha_index = ((alpha_mask >> (3 * (4 * block_y + block_x))) & 0x07) as u32;
                    match alpha_index{
                        0 => {
                            a = alpha[0];
                        },
                        1 => {
                            a = alpha[1];
                        },
                        _ =>{
                            if alpha[0] > alpha[1]{
                                a = (((8 - alpha_index) * alpha[0] as u32 + (alpha_index - 1) * alpha[1] as u32) / 7) as u8;
                            }
                            else {
                                match alpha_index{
                                    6 =>{
                                        a = 0;
                                    },
                                    7 =>{
                                        a = 0xFF;
                                    },
                                    _ =>{
                                        a = (((6 - alpha_index) * alpha[0] as u32 + (alpha_index - 1) * alpha[1] as u32) / 5) as u8;
                                    }
                                }
                            }
                        }
                    }
                }
                match index{
                    0 => {
                        r = r0;
                        g = g0;
                        b = b0;                       
                    },
                    1 => {
                        r = r1;
                        g = g1;
                        b = b1;               
                    },  
                    2 => {
                        if format == DDT_FORMAT_DXT1 || format == DDT_FORMAT_DXT1DE {
                            if c0 > c1 {
                                r = ((2 * r0 as u16 + r1 as u16) / 3) as u8;
                                g = ((2 * g0 as u16 + g1 as u16) / 3) as u8;
                                b = ((2 * b0 as u16 + b1 as u16) / 3) as u8;                               
                            }
                            else {
                                r = ((r0 as u16 + r1 as u16) / 2) as u8;
                                g = ((g0 as u16 + g1 as u16) / 2) as u8;
                                b = ((b0 as u16 + b1 as u16) / 2) as u8;        
                            }
                        }
                        else {
                            r = ((2 * r0 as u16 + r1 as u16) / 3) as u8;
                            g = ((2 * g0 as u16 + g1 as u16) / 3) as u8;
                            b = ((2 * b0 as u16 + b1 as u16) / 3) as u8;
                        }
                    },       
                    3 => {
                        if format == DDT_FORMAT_DXT1 || format == DDT_FORMAT_DXT1DE {
                            if c0 > c1 {
                                r = ((r0 as u16 + (2 * r1 as u16)) / 3) as u8;
                                g = ((g0 as u16 + (2 * g1 as u16)) / 3) as u8;
                                b = ((b0 as u16 + (2 * b1 as u16)) / 3) as u8;                          
                            }
                            else {
                                r = 0;
                                g = 0;
                                b = 0;
                                a = 0; 
                            }
                        }
                        else {
                            r = ((r0 as u16 + (2 * r1 as u16)) / 3) as u8;
                            g = ((g0 as u16 + (2 * g1 as u16)) / 3) as u8;
                            b = ((b0 as u16 + (2 * b1 as u16)) / 3) as u8;
                        }
                    }, 
                    _=>{
    
                    }                    
                }
                        
    
                let px = (x << 2) + block_x;
                let py = (y << 2) + block_y;
                if px >= width || py >= height {
                    continue;
                }
                let offset = (py * width + px) << 2;
    
                data[offset as usize] = b;
                
                data[offset as usize + 1] = g;
                if usage & DDT_USAGE_BUMP == DDT_USAGE_BUMP && format == DDT_FORMAT_DXT5 {
                    data[offset as usize + 2] = a;
                    data[offset as usize + 3] = r;
                }
                else{
                    data[offset as usize + 2] = r;
                    data[offset as usize + 3] = a;
                }
            }
        }
    }

    fn prepare_to_encoding(data: &[u8], format: u8, usage: u8) -> Vec<u8> {

        let mut res: Vec<u8> = Vec::new();
    
        for chunk in data.chunks(4) {
            let b = chunk[0];
            let g = chunk[1];
            let r: u8;
            let a: u8;
            if usage & DDT_USAGE_BUMP == DDT_USAGE_BUMP && format == DDT_FORMAT_DXT5 {
                r = chunk[3];
                a = chunk[2];
            }
            else{
                r = chunk[2];
                a = chunk[3];           
            }
    
            
    
            if format == DDT_FORMAT_DXT1 || format == DDT_FORMAT_DXT1DE {
                res.push(r);
                res.push(g);
                res.push(b);
            }
            else{
                res.push(r); 
                res.push(g);
                res.push(b);
                res.push(a);
            }
        }
    
    res
    
    }
    
    pub fn compress(data: &[u8], format: u8, usage: u8, width: u32, height: u32) -> Vec<u8>{
    
        let data = DxtImage::prepare_to_encoding(data, format, usage);
    
        let width_blocks = width / 4;
        let height_blocks = height / 4;
        let stride = DxtImage::decoded_bytes_per_block(format);
        
        let mut res: Vec<u8> = Vec::new();
        for chunk in data.chunks(width_blocks as usize * stride) {
            let mut buf;
            if format == DDT_FORMAT_DXT1 || format == DDT_FORMAT_DXT1DE {
                buf = DxtImage::encode_dxt1_row(chunk);
            }
            else if format == DDT_FORMAT_DXT3 {
                buf = DxtImage::encode_dxt3_row(chunk);
            }
            else {
                buf = DxtImage::encode_dxt5_row(chunk);
            }
            res.append(&mut buf);
        }
        res
    }
    
    fn decoded_bytes_per_block(format: u8) -> usize {
        if format == DDT_FORMAT_DXT1 || format == DDT_FORMAT_DXT1DE {
           48
        }
        else {
           64
        }
    }
    
    fn encoded_bytes_per_block(format: u8) -> usize {
        if format == DDT_FORMAT_DXT1 || format == DDT_FORMAT_DXT1DE {
           8
        }
        else {
           16
        }
    }

    fn scanline_bytes(width_blocks: u32, format: u8) -> u64 {
        DxtImage::decoded_bytes_per_block(format) as u64 * u64::from(width_blocks)
    }
    
    
    
    fn enc565_decode(value: u16) -> Rgb {
        let red =  (value >> 11) & 0x1F;
        let green =  (value >> 5) & 0x3F;
        let blue =  (value) & 0x1F;
        [
            (red * 0xFF / 0x1F) as u8,
            (green * 0xFF / 0x3F) as u8,
            (blue * 0xFF / 0x1F) as u8,
        ]
    }
    
    fn enc565_encode(rgb:Rgb) -> u16{
        let red = (u16::from(rgb[0]) * 0x1F + 0x7E) / 0xFF;
        let green = (u16::from(rgb[1]) * 0x3F + 0x7E) / 0xFF;
        let blue = (u16::from(rgb[2]) * 0x1F + 0x7E) / 0xFF;
    
        (red << 11) | (green << 5) | blue
    }
    
    
    fn square(a: i32) -> i32 {
        a * a
    }
    
    fn diff(a: Rgb, b: Rgb) -> i32{
        DxtImage::square (i32::from(a[0]) - i32::from(b[0])) + DxtImage::square(i32::from(a[1]) - i32::from(b[1]))
        + DxtImage::square (i32::from(a[2]) - i32::from(b[2]))
    }
    
    fn encode_dxt_colors(source: &[u8], dest: &mut [u8]) {
        assert!((source.len() == 64 || source.len() == 48) && dest.len() == 8);
        let stride = source.len() / 16;
        let mut colors = [[0u8; 3]; 4];
    
        let mut targets = [[0u8; 3]; 16];
    
        for (s, d) in source.chunks(stride).rev().zip(&mut targets) {
            *d = [s[0], s[1], s[2]];
        }
    
        let mut colorspace = targets.to_vec();
    
        for rgb in & mut colorspace {
            *rgb = DxtImage::enc565_decode(DxtImage::enc565_encode(*rgb));
        }
    
        colorspace.dedup();
    
        if colorspace.len() == 1 {
            let ref_rgb = colorspace[0];
            let mut rgb = targets.iter()
            .cloned()
            .max_by_key(|rgb| DxtImage::diff(*rgb, ref_rgb))
            .unwrap();
    
            for i in 0..3 {
                rgb[i] = ((i16::from(rgb[i]) - i16::from(ref_rgb[i])) * 5 / 2 + i16::from(ref_rgb[i])) as u8;
            }
    
            let encoded = DxtImage::enc565_encode(rgb);
            let rgb = DxtImage::enc565_decode(encoded);
    
            if rgb == ref_rgb {
                dest[0] = encoded as u8;
                dest[1] = (encoded >> 8) as u8;
                for d in dest.iter_mut().take(8).skip(2) {
                    *d = 0;
                }
                return;
            }
            colorspace.push(rgb);
        }
    
        let mut chosen_colors = [[0; 3]; 4];
        let mut chosen_use_0 = false;
        let mut chosen_error = 0xFFFF_FFFFu32;
        'search: for (i, &c1) in colorspace.iter().enumerate() {
            colors[0] = c1;
    
            for &c2 in &colorspace[0..i] {
                colors[1] = c2;
    
                for use_0 in 0..2 {
    
                    if use_0 != 0 {
                        for i in 0..3 {
                            colors[2][i] = ((u16::from(colors[0][i]) + u16::from(colors[1][i]) + 1) / 2) as u8;
                        }
    
                        colors[3] = [0, 0, 0];
                    }
                    else{
                        for i in 0..3 {
                            colors[2][i] = ((u16::from(colors[0][i]) * 2 + u16::from(colors[1][i]) + 1) / 3) as u8;
                            colors[3][i] = ((u16::from(colors[0][i]) + u16::from(colors[1][i]) * 2 + 1) / 3) as u8;
                        }
                    }
    
                    let total_error = targets.iter()
                        .map(|t| colors.iter().map(|c| DxtImage::diff(*c, *t) as u32).min().unwrap())
                        .sum();
                    if total_error < chosen_error {
                        chosen_colors = colors;
                        chosen_use_0 = use_0 != 0;
                        chosen_error = total_error;
    
                        if total_error < 4 {
                            break 'search;
                        }
                    }
                }
            }
        }
    
    
        let mut chosen_indices = 0u32;
        for t in &targets {
            let (idx, _) = chosen_colors
                .iter()
                .enumerate()
                .min_by_key(|&(_,c) | DxtImage::diff (*c, *t))
                .unwrap();
    
            chosen_indices = (chosen_indices << 2) | idx as u32;
        }
    
        let mut color0 = DxtImage::enc565_encode(chosen_colors[0]);
        let mut color1 = DxtImage::enc565_encode(chosen_colors[1]);
    
        if color0> color1 {
            if chosen_use_0 {
                swap(&mut color0, &mut color1);
    
                let filter = (chosen_indices & 0xAAAA_AAAA) >> 1;
                chosen_indices ^= filter ^ 0x5555_5555;
    
            }
        }
        else if !chosen_use_0 {
            swap(&mut color0, &mut color1);
            chosen_indices ^= 0x5555_5555;
        }
    
        dest[0] = color0 as u8;
        dest[1] = (color0 >> 8) as u8;
        dest[2] = color1 as u8;
        dest[3] = (color1 >> 8) as u8;
    
        for i in 0..4 {
            dest[i + 4] = (chosen_indices >> (i * 8)) as u8;
        }
    }
    
    fn encode_dxt1_block(source: &[u8], dest: &mut [u8]) {
        assert!(source.len() == 48 && dest.len() == 8);
        DxtImage::encode_dxt_colors(source, dest);
    }
    
    fn encode_dxt1_row(source: &[u8]) -> Vec<u8> {
        assert!(source.len() % 48 == 0);
        let block_count = source.len() / 48;
        let mut dest = vec![0u8; block_count * 8];
        let mut decoded_block = [0u8; 48];
    
        for (x, encoded_block) in dest.chunks_mut(8).enumerate() {
            for line in 0..4 {
                let offset = (block_count * line + x) * 12;
                decoded_block[line * 12..(line + 1) * 12].copy_from_slice(&source[offset..offset + 12]);
            }
    
            DxtImage::encode_dxt1_block(&decoded_block, encoded_block);
        }
        return dest;
    }
    
    
    fn encode_dxt5_alpha(alpha0: u8, alpha1: u8, alphas: &[u8;16]) -> (i32, u64){
        let table = DxtImage::alpha_table_dxt5(alpha0, alpha1);
    
        let mut indices = 0u64;
        let mut total_error = 0i32;
    
        for (i, &a) in alphas.iter().enumerate(){
            let (index, error) = table  
            .iter()
            .enumerate()
            .map(|(i, &e)| (i, DxtImage::square(i32::from(e) - i32::from(a))))
            .min_by_key(|&(_, e)| e)
            .unwrap();
            total_error += error;
            indices |= (index as u64) << (i * 3);
        }
    
        (total_error, indices)
    }
    
    fn encode_dxt5_block(source: &[u8], dest: &mut [u8]){
        assert!(source.len() == 64 && dest.len() == 16);
    
        DxtImage::encode_dxt_colors(source, &mut dest[8..16]);
        let mut alphas = [0; 16];
        for i in 0..16{
            alphas[i] = source[i * 4 + 3];
        }
    
        let alpha07 = alphas.iter().cloned().min().unwrap();
        let alpha17 = alphas.iter().cloned().max().unwrap();
        let (error7, indices7) = DxtImage::encode_dxt5_alpha(alpha07, alpha17, &alphas);
    
        let alpha05 = alphas
            .iter()
            .cloned()
            .filter(|&i| i != 255)
            .max()
            .unwrap_or(255);
        let alpha15 = alphas
            .iter()
            .cloned()
            .filter(|&i| i != 0)
            .min()
            .unwrap_or(0);
    
        let (error5, indices5) = DxtImage::encode_dxt5_alpha(alpha05, alpha15, &alphas);
    
        let mut alpha_table = if error5 < error7{
            dest[0] = alpha05;
            dest[1] = alpha15;
            indices5
        }
        else{
            dest[0] = alpha07;
            dest[1] = alpha17;
            indices7        
        };
    
        for byte in dest[2..8].iter_mut(){
            *byte = alpha_table as u8;
            alpha_table >>= 8;
    
        }
    }
    
    fn encode_dxt3_block(source:&[u8], dest: &mut[u8]){
        assert!(source.len() == 64 && dest.len() == 16);
    
        DxtImage::encode_dxt_colors(source, &mut dest[8..16]);
    
        let mut alpha_table = 0u64;
        for i in 0..16{
            let alpha = u64::from(source[i * 4 + 3]);
            let alpha = (alpha + 0x8) / 0x11;
            alpha_table |= alpha << (i * 4);
        }
    
        for byte in &mut dest[0..9]{
            *byte = alpha_table as u8;
            alpha_table >>= 8;
        }
    }
    
    
    fn encode_dxt3_row(source: &[u8]) -> Vec<u8>{
        assert!(source.len() % 64 == 0);
        let block_count = source.len() / 64;
    
        let mut dest = vec![0u8; block_count * 16];
        let mut decoded_block = [0u8; 64];
    
        for (x, encoded_block) in dest.chunks_mut(16).enumerate(){
            for line in 0..4{
                let offset = (block_count * line + x) * 16;
                decoded_block[line * 16..(line + 1) * 16].copy_from_slice(&source[offset..offset+ 16]);
            }
    
            DxtImage::encode_dxt3_block(&decoded_block, encoded_block);
    
        }
    
        dest
    }
    
    fn encode_dxt5_row(source: &[u8]) -> Vec<u8>{
        assert!(source.len() % 64 == 0);
        let block_count = source.len() / 64;
        let mut dest = vec![0u8; block_count * 16];
        let mut decoded_block = [0u8; 64];
    
        for (x, encoded_block) in dest.chunks_mut(16).enumerate(){
            for line in 0..4{
                let offset = (block_count * line + x) * 16;
                decoded_block[line * 16..(line+1) * 16].copy_from_slice(&source[offset..offset + 16]);
            }
    
            DxtImage::encode_dxt5_block(&decoded_block, encoded_block);
        }
        dest
    }
    
    fn alpha_table_dxt5(alpha0: u8, alpha1: u8) -> [u8; 8]{
        let mut table = [alpha0, alpha1, 0, 0, 0, 0, 0, 0xFF];
        if alpha0 > alpha1{
            for i in 2..8u16{
                table[i as usize] = (((8 - i) * u16::from(alpha0) + (i - 1) * u16::from(alpha1)) / 7) as u8;
            }
        }
        else{
            for i in 2..6u16{
                table[i as usize] = (((6 - i) * u16::from(alpha0) + (i - 1) * u16::from(alpha1)) / 5) as u8;            
            }
        }
    
        table
    }
    
}







/*
pub fn decode_dxt(data: &[u8], format: u8, usage:u8, width: u32, height: u32)-> Vec<u8>{

    if width % 4 != 0 || height % 4 != 0{
        panic!("Ошибка декодирования: неверное разрешение DXT текстуры");
    }

    

    let width_blocks = width / 4;
    let height_blocks = height / 4;

    let mut buf:Vec<u8> = Vec::new();
    let mut reader = BufReader::new(Cursor::new(&data));
    for chunk in data.chunks(scanline_bytes(width_blocks, format) as usize){
        let mut src = vec![0u8; encoded_bytes_per_block(format) * width_blocks as usize];
        let mut dec_src = chunk.to_vec();
        reader.read_exact(&mut src).unwrap();
        if format == DDT_FORMAT_DXT1 || format == DDT_FORMAT_DXT1DE{
           decode_dxt1_row(&src, &mut dec_src);
        }
        else if format == DDT_FORMAT_DXT3{
           decode_dxt3_row(&src, &mut dec_src);
        }
        else{
          decode_dxt5_row(&src, &mut dec_src);
        }
        buf.append(&mut dec_src);
    }
    assert_eq!(buf.len()as usize, (width*height*4) as usize);
    buf
    

}
*/

/*
fn decode_dxt_colors(source: &[u8], dest: &mut [u8]){
    assert!(source.len() == 8 && (dest.len() == 48 || dest.len() == 64));
    let pitch = dest.len() / 16;
    let color0 = u16::from(source[0]) | (u16::from(source[1]) << 8);
    let color1 = u16::from(source[2]) | (u16::from(source[3]) << 8);
    let color_table = u32::from(source[4]) | (u32::from(source[5]) << 8)
    | (u32::from(source[6]) << 16) | (u32::from(source[7]) << 24);

    let mut colors = [[0;3]; 4];
    colors[0] = enc565_decode(color0);
    colors[1] = enc565_decode(color1);

    if color0 > color1{
        for i in 0..3{
            colors[2][i] = ((u16::from(colors[0][i]) * 2 + u16::from(colors[1][i]) + 1) / 3) as u8;
            colors[2][i] = ((u16::from(colors[0][i]) + u16::from(colors[1][i]) * 2 + 1) / 3) as u8;
        }
    }
    else{
        for i in 0..3{
            colors[2][i] = ((u16::from(colors[0][i]) + u16::from(colors[1][i]) + 1) / 2) as u8;
        }
    }


    for i in 0..16{
        dest[i * pitch..i*pitch + 3].copy_from_slice(&colors[(color_table >> (i * 2)) as usize & 3]);
    }
}





fn decode_dxt1_block(source: &[u8], dest: &mut [u8]){
    assert!(source.len() == 8 && dest.len() == 48);
    decode_dxt_colors(&source, dest);
}

fn decode_dxt3_block(source: &[u8], dest: &mut [u8]){
    assert!(source.len() == 16 && dest.len() == 64);

    let alpha_table = source[0..8]
        .iter()
        .rev()
        .fold(0, |t, &b| (t << 8) | u64::from(b));


    for i in 0..16{
        dest[i * 4 + 3] = ((alpha_table >> (i * 4)) as u8 & 0xF) * 0x11;
    }

    decode_dxt_colors(&source[8..16], dest);
}

fn decode_dxt5_block(source: &[u8], dest: &mut [u8]){
    assert!(source.len() == 16 && dest.len() == 64);

    let alpha_table = source[2..8]
        .iter()
        .rev()
        .fold(0, |t, &b| (t << 8) | u64::from(b));

    let alphas = alpha_table_dxt5(source[0], source[1]);

    for i in 0..16{
        dest[i * 4 + 3] = alphas[(alpha_table >> (i * 3)) as usize & 7];
    }

    decode_dxt_colors(&source[8..16], dest);
}


fn decode_dxt1_row(source: &[u8], dest: &mut [u8]){
    assert!(source.len() % 8 == 0);
    let block_count = source.len() / 8;
    assert!(dest.len() >= block_count * 48);

    let mut decoded_block = [0u8; 48];
    for (x, encoded_block) in source.chunks(8).enumerate(){
        decode_dxt1_block(encoded_block, &mut decoded_block);
        for line in 0..4{
            let offset = (block_count * line +x) * 12;
            dest[offset..offset + 12].copy_from_slice(&decoded_block[line*12..(line+1)*12]);
        }
    }
}

fn decode_dxt3_row(source: &[u8], dest: &mut [u8]){
    assert!(source.len() % 16 == 0);
    let block_count = source.len() / 16;
    assert!(dest.len() >= block_count * 64);

    let mut decoded_block = [0u8; 64];
    for (x, encoded_block) in source.chunks(16).enumerate(){
        decode_dxt3_block(encoded_block, &mut decoded_block);
        for line in 0..4{
            let offset = (block_count * line +x) * 16;
            dest[offset..offset + 16].copy_from_slice(&decoded_block[line*16..(line+1)*16]);
        }
    }
}

fn decode_dxt5_row(source: &[u8], dest: &mut [u8]){
    assert!(source.len() % 16 == 0);
    let block_count = source.len() / 16;
    assert!(dest.len() >= block_count * 64);

    let mut decoded_block = [0u8; 64];
    for (x, encoded_block) in source.chunks(16).enumerate(){
        decode_dxt5_block(encoded_block, &mut decoded_block);
        for line in 0..4{
            let offset = (block_count * line +x) * 16;
            dest[offset..offset + 16].copy_from_slice(&decoded_block[line*16..(line+1)*16]);
        }
    }
}*/
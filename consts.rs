

pub const TGA_ALPHA_BITS_MASK: u8 = 0b1111;
pub const TGA_SCREEN_ORIGIN_BIT_MASK: u8 = 0b10_0000;
pub const TGA_UNCOMPRESSED_TRUE_COLOR: u8 = 2;

pub const DDT_ALPHA_NONE: u8 = 0;
pub const DDT_ALPHA_PLAYER: u8 = 1;
pub const DDT_ALPHA_TRANS: u8 = 4;
pub const DDT_ALPHA_BLEND: u8 = 8;

pub const DDT_USAGE_STANDARD: u8 = 0;
pub const DDT_USAGE_ALPHATEST: u8 = 1;
pub const DDT_USAGE_LOWDETAIL: u8 = 2;
pub const DDT_USAGE_BUMP: u8 = 4;
pub const DDT_USAGE_CUBE: u8 = 8;

pub const DDT_FORMAT_BGRA: u8 = 1;
pub const DDT_FORMAT_DXT1: u8 = 4;
pub const DDT_FORMAT_DXT1DE: u8 = 5;
pub const DDT_FORMAT_GREY: u8 = 7;
pub const DDT_FORMAT_DXT3: u8 = 8;
pub const DDT_FORMAT_DXT5: u8 = 9;


pub const BAR_VERSION_AOE3: u32 = 2; // Legacy
pub const BAR_VERSION_AOE3DE: u32 = 6; // DE

pub const ENCODE_TYPE_NONE: u32 = 0; // raw data
pub const ENCODE_TYPE_ALZ4_L33T: u32 = 1; // alz4 or l33t encoding
pub const ENCODE_TYPE_SND: u32 = 2; // sound file encoding


pub const BINARY_SIGNATURE_ALZ4: u32 = 0x347A6C61;
pub const BINARY_SIGNATURE_L33T: u32 = 0x6C333374;
pub const BINARY_SIGNATURE_WAV_DECODED: u32 = 0x46464952;
pub const BINARY_SIGNATURE_WAV_ENCODED: u32 = 0xB4428C6D;
pub const BINARY_SIGNATURE_MP3: u32 = 0x334449;
pub const BINARY_SIGNATURE_BAR: u32 = 0x4E505345;
pub const BINARY_SIGNATURE_DDT: u32 = 0x33535452;

pub const BINARY_BAR_MAGIC: u32 = 0x44332211;
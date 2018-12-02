use std::fmt;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::str;

struct Color {
    red: u8,
    green: u8,
    blue: u8,
}

impl Color {
    fn hex_code(&self) -> u32 {
        return ((self.red as u32 & 0xff) << 16) + ((self.green as u32 & 0xff) << 8)
            + (self.blue as u32 & 0xff);
    }
}

impl fmt::Debug for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Color {{ {:X} }}", self.hex_code())
    }
}

#[derive(Debug)]
struct Gif {
    version: GifVersion,
    lsd: LogicalScreenDescriptor,
    global_color_table: Option<Vec<Color>>,
}

#[derive(Debug)]
struct LogicalScreenDescriptor {
    width: u16,
    height: u16,
    has_global_color_table: bool,
    color_resolution: u8,
    is_global_color_table_sorted: bool,
    background_color_index: Option<u8>,
    global_color_table_size: u8,
    pixel_aspect_ratio: u8,
}

#[derive(Debug)]
enum GifError {
    Io(io::Error),
    InvalidGifFile,
    UnsupportedVersion(String),
}

#[derive(Debug)]
enum GifVersion {
    V87a,
    V89a,
}

impl Gif {
    fn from_file(f: &mut File) -> Result<Gif, GifError> {
        //read header
        let mut buffer = [0; 6];
        try!(f.read(&mut buffer).map_err(|e| GifError::Io(e)));
        let version = try!(Gif::parse_version(&buffer));

        //read logical screen descriptor
        let mut buffer = [0; 7];
        try!(f.read(&mut buffer).map_err(|e| GifError::Io(e)));
        let lsd = try!(Gif::parse_logical_screen_descriptor(&buffer));

        //read global color table, if present.
        let global_color_table = match lsd.has_global_color_table {
            true => {
                let mut buffer = vec![0; lsd.global_color_table_size as usize];
                try!(f.read(&mut buffer).map_err(|e| GifError::Io(e)));
                Some(Gif::parse_global_color_table(&buffer))
            }
            _ => None,
        };

        //TODO: remove
        let mut bytes = vec![];
        try!(f.read_to_end(&mut bytes).map_err(|e| GifError::Io(e)));

        return Ok(Gif {
            version: version,
            lsd: lsd,
            global_color_table: global_color_table,
        });
    }

    fn parse_version(bytes: &[u8; 6]) -> Result<GifVersion, GifError> {
        if str::from_utf8(&bytes[0..3]).unwrap() != "GIF" {
            return Err(GifError::InvalidGifFile);
        }

        let version = match str::from_utf8(&bytes[3..6]).unwrap() {
            "87a" => GifVersion::V87a,
            "89a" => GifVersion::V89a,
            unsupported => return Err(GifError::UnsupportedVersion(unsupported.to_owned())),
        };
        Ok(version)
    }

    fn parse_logical_screen_descriptor(
        bytes: &[u8; 7],
    ) -> Result<LogicalScreenDescriptor, GifError> {
        let width = ((bytes[1] as u16) * 1u16 << 8u16) + (bytes[0] as u16);
        let height = ((bytes[3] as u16) * 1u16 << 8u16) + (bytes[2] as u16);

        let packed_fields = bytes[4];
        let has_global_color_table = (packed_fields & 0b10000000) == 0b10000000;
        let is_global_color_table_sorted = (packed_fields & 0b00001000) == 0b00001000;

        let color_resolution = (bytes[4] & 0b01110000) + 1u8;
        let global_color_table_size = 3 * ((bytes[4] & 0b00000111) + 1u8).pow(2);

        let background_color_index = match has_global_color_table {
            true => Some(bytes[5]),
            _ => None,
        };

        let pixel_aspect_ratio = bytes[6];

        Ok(LogicalScreenDescriptor {
            width: width,
            height: height,
            has_global_color_table: has_global_color_table,
            color_resolution: color_resolution,
            is_global_color_table_sorted: is_global_color_table_sorted,
            background_color_index: background_color_index,
            global_color_table_size: global_color_table_size,
            pixel_aspect_ratio: pixel_aspect_ratio,
        })
    }

    fn parse_global_color_table(table: &Vec<u8>) -> Vec<Color> {
        let mut colors = Vec::with_capacity(table.len() / 3);
        let mut i = 0;
        while i < table.len() {
            colors.push(Color {
                red: table[i],
                green: table[i + 1],
                blue: table[i + 2],
            });
            i += 3;
        }
        colors
    }
}

fn main() {
    let file_name = "earth.gif";
    let mut f = File::open(file_name).expect("file not found");

    let gif = Gif::from_file(&mut f).unwrap();

    println!("-> {:?}", gif);
}

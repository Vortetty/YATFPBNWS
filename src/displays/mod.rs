mod edid_video_input_params;
mod preferred_timing;
mod descriptor_parse;

use std::{fs::File, path::Path};
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use edid_video_input_params::{EDIDAnalogVideoInputParams, EDIDBitDepth, EDIDDigitalVideoInputParams, EDIDInputParams, EDIDVideoInterface};

static EDID_HEADER_PATTERN: u64 = 0x00_FF_FF_FF_FF_FF_FF_00;

#[derive(Debug, Clone)]
enum EDIDParseErrorType {
    MissingFile,
    InvalidHeader,
    EarlyEof,
    UnsupportedEDIDVer
}

#[derive(Debug, Clone)]
struct EDIDParseError {
    err_type: EDIDParseErrorType,
    message: Option<String>
}

struct EDID {
    good_header: bool,
    manufacturer_id: String,
    product_code: u16,
    serial_number: u32,
    edid_ver: f32,
    input_params: EDIDInputParams,
    debug_number: i64
}

macro_rules! EDIDError {
    ($t:expr) => {
        Err(EDIDParseError { err_type: $t, message: None })
    };
    ($t:expr, $msg:expr) => {
        Err(EDIDParseError { err_type: $t, message: Some($msg.to_string()) })
    };
}

impl EDID {
    pub fn parse_edid(filename: String) -> Result<EDID, EDIDParseError> {
        let mut out = EDID {
            good_header: false,
            manufacturer_id: "".to_string(),
            product_code: 0,
            serial_number: 0,
            edid_ver: 0.0,
            input_params: EDIDInputParams {
                digi: None,
                anal: None,
                is_anal: false
            },
            debug_number: 0,
            descriptors: MonitorDescriptors
        };

        let mut f;
        if Path::new(&filename).is_file() {
            f = File::open(filename).unwrap();
        } else {
            return EDIDError!(EDIDParseErrorType::MissingFile);
        }

        let header = f.read_u64::<BigEndian>();
        if header.is_ok_and(|hdr| hdr == EDID_HEADER_PATTERN) {
            out.good_header = true;
        } else {
            return EDIDError!(EDIDParseErrorType::InvalidHeader);
        }

        let manufacturer_id = f.read_u16::<BigEndian>();
        if manufacturer_id.is_ok() {
            let id = manufacturer_id.unwrap();
            let mut tmp: Vec<char> = vec![];
            tmp.push((((id & 0b0111110000000000) >> 10) as u8 + 0x40) as char);
            tmp.push((((id & 0b0000001111100000) >> 5) as u8 + 0x40) as char);
            tmp.push(((id & 0b0000000000011111) as u8 + 0x40) as char);
            out.manufacturer_id = String::from_iter(tmp);
        } else {
            return EDIDError!(EDIDParseErrorType::EarlyEof, "Manufacturer id missing")
        }

        let prod_code = f.read_u16::<LittleEndian>();
        if prod_code.is_ok() {
            out.product_code = prod_code.unwrap();
        } else {
            return EDIDError!(EDIDParseErrorType::EarlyEof, "Product code missing")
        }

        let serial = f.read_u32::<LittleEndian>();
        if serial.is_ok() {
            out.serial_number = serial.unwrap();
        } else {
            return EDIDError!(EDIDParseErrorType::EarlyEof, "Serial missing")
        }

        let _dateinfo = f.read_u16::<BigEndian>(); // IDGAF about this, none of it is *really* standard aside from year

        let edid_ver = f.read_u8();
        let edid_rev = f.read_u8();
        if edid_ver.is_ok() && edid_rev.is_ok() {
            if edid_ver.unwrap() == 0x01 || edid_rev.unwrap() == 0x03 { // EDID Ver 1.4 only
                out.edid_ver = 1.4;
            } else {
                return EDIDError!(EDIDParseErrorType::UnsupportedEDIDVer, "EDID Version Unsupported")
            }
        } else {
            return EDIDError!(EDIDParseErrorType::EarlyEof, "EDID Version Missing")
        }

        // *
        // *
        // * Basic display info
        // *
        // *
        let input_params = f.read_u8();
        if  input_params.is_ok() {
            if input_params.as_ref().unwrap() & 0b10000000 > 0 { // Digi
                out.input_params.is_anal = false;
                out.input_params.digi = Some(EDIDDigitalVideoInputParams::from(input_params.unwrap()));
            } else { // Anal
                out.input_params.is_anal = true;
                out.input_params.anal = Some(EDIDAnalogVideoInputParams::from(input_params.unwrap()));
            }
        } else {
            return EDIDError!(EDIDParseErrorType::EarlyEof, "Display info missing");
        }

        // Display size (why do i need this)
        let _horiz_size = f.read_u8();
        let _vert_size = f.read_u8();
        let _gamma = f.read_u8();

        // feature bitmap (useless for me)
        let _features = f.read_u8();


        // *
        // *
        // * Chromaticity
        // *
        // *
        for _ in 0..10 { // Feck u i don't need u
            let _features = f.read_u8();
        }

        // *
        // *
        // * Established timins
        // *
        // *
        for _ in 0..3 { // Feck u i don't need u, the max resolution will be read from extended display info
            let _features = f.read_u8();
        }

        // *
        // *
        // * Standard timing info
        // *
        // *
        for _ in 0..16 { // Feck u i don't need u, the max resolution will be read from extended display info
            let _features = f.read_u8();
        }

        // *
        // *
        // * Display/Detailed descriptors
        // *
        // *

        return Ok(out);
    }
}

pub fn get_displays() -> String {
    let tmp = EDID::parse_edid("/sys/class/drm/card1-DP-1/edid".to_string()).unwrap();
    return format!(
        "{}, {}, {:04x}, {}, {}, {}@{}, {}",
        tmp.good_header, tmp.manufacturer_id, tmp.product_code, tmp.serial_number, tmp.edid_ver, match tmp.input_params.digi.as_ref().unwrap().interface {
            EDIDVideoInterface::Undef => "Undefined Protocol",
            EDIDVideoInterface::DVI => "DVI",
            EDIDVideoInterface::HDMIA => "HDMI Type A",
            EDIDVideoInterface::HDMIB => "HDMI Type B",
            EDIDVideoInterface::MDDI => "MDDI",
            EDIDVideoInterface::DP => "DP",
        }, match tmp.input_params.digi.as_ref().unwrap().bit_depth {
            EDIDBitDepth::Undef => "?b",
            EDIDBitDepth::B6 => "6b",
            EDIDBitDepth::B8 => "8b",
            EDIDBitDepth::B10 => "10b",
            EDIDBitDepth::B12 => "12b",
            EDIDBitDepth::B14 => "14b",
            EDIDBitDepth::Bpp16 => "16bpp",
            EDIDBitDepth::Reserved => "Reserved",
        },
        tmp.debug_number
    );
}

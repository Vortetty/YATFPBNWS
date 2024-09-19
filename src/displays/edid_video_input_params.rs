

pub enum EDIDBitDepth {
    Undef,
    B6,
    B8,
    B10,
    B12,
    B14,
    Bpp16,
    Reserved,
}
pub enum EDIDVideoInterface {
    Undef,
    DVI,
    HDMIA,
    HDMIB,
    MDDI,
    DP,
}

#[allow(non_camel_case_types)]
pub enum EDIDWhiteSyncLevels {
    V0_7_0_3,     // +0.7/−0.3 V
    V0_714_0_286, // +0.714/−0.286 V
    V1_0_0_4,     // +1.0/−0.4 V
    V0_7_0_0_EVC, // +0.7/0 V
    CodeFailure,
}

#[allow(dead_code)]
pub struct EDIDDigitalVideoInputParams {
    pub bit_depth: EDIDBitDepth,
    pub interface: EDIDVideoInterface,
}
#[allow(dead_code)]
pub struct EDIDAnalogVideoInputParams {
    pub white_sync_levels: EDIDWhiteSyncLevels,
    pub blank_to_black: bool,
    pub separate_sync_support: bool,
    pub composite_sync: bool,
    pub sync_on_green: bool,
    pub green_sync_vsync_serration: bool,
}

impl From<u8> for EDIDDigitalVideoInputParams {
    fn from(v: u8) -> Self {
        // the easy one
        EDIDDigitalVideoInputParams {
            bit_depth: match (v >> 4) & 0b00000111 {
                0b000 => EDIDBitDepth::Undef,
                0b001 => EDIDBitDepth::B6,
                0b010 => EDIDBitDepth::B8,
                0b011 => EDIDBitDepth::B10,
                0b100 => EDIDBitDepth::B12,
                0b101 => EDIDBitDepth::B14,
                0b110 => EDIDBitDepth::Bpp16,
                0b111 => EDIDBitDepth::Reserved,
                _ => EDIDBitDepth::Undef,
            },
            interface: match v & 0b00001111 {
                0b0000 => EDIDVideoInterface::Undef,
                0b0001 => EDIDVideoInterface::DVI,
                0b0010 => EDIDVideoInterface::HDMIA,
                0b0011 => EDIDVideoInterface::HDMIB,
                0b0100 => EDIDVideoInterface::MDDI,
                0b0101 => EDIDVideoInterface::DP,
                _ => EDIDVideoInterface::Undef,
            },
        }
    }
}

impl From<u8> for EDIDAnalogVideoInputParams {
    fn from(v: u8) -> Self {
        EDIDAnalogVideoInputParams {
            white_sync_levels: match (v & 0b01100000) >> 5 {
                0b00 => EDIDWhiteSyncLevels::V0_7_0_3,
                0b01 => EDIDWhiteSyncLevels::V0_714_0_286,
                0b10 => EDIDWhiteSyncLevels::V1_0_0_4,
                0b11 => EDIDWhiteSyncLevels::V0_7_0_0_EVC,
                _ => EDIDWhiteSyncLevels::CodeFailure, // I SWEAR TO GOD IF THIS HAPPENS I WILL... idk probably go buy a farm like the neofetch dev
            },
            blank_to_black: (v & 0b00010000) > 0,
            separate_sync_support: (v & 0b00001000) > 0,
            composite_sync: (v & 0b00000100) > 0,
            sync_on_green: (v & 0b00000010) > 0,
            green_sync_vsync_serration: (v & 0b00000001) > 0
        }
    }
}

pub struct EDIDInputParams {
    pub digi: Option<EDIDDigitalVideoInputParams>,
    pub anal: Option<EDIDAnalogVideoInputParams>,
    pub is_anal: bool // False = digital, True = analog
}
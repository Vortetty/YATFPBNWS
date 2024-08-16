use sysinfo::System;

pub fn get_uptime() -> String {
    let uptime = System::uptime();
    let (centuries, uptime) = (uptime / 3_153_600_000, uptime % 3_153_600_000);
    let (decades, uptime) = (uptime / 315_360_000, uptime % 315_360_000);
    let (years, uptime) = (uptime / 31_563_000, uptime % 31_563_000);
    let (days, uptime) = (uptime / 86_400, uptime % 86_400);
    let (hours, uptime) = (uptime / 3_600, uptime % 3_600);
    let minutes = uptime / 60;

    let mut output = "".to_string();

    if centuries > 0 {
        output += format!("{}C. ", centuries).as_str();
    }
    if decades > 0 {
        output += format!("{}D. ", decades).as_str();
    }
    if years > 0 {
        output += format!("{}y ", years).as_str();
    }
    if days > 0 {
        output += format!("{}d ", days).as_str();
    }
    if hours > 0 {
        output += format!("{}h ", hours).as_str();
    }
    if minutes > 0 {
        output += format!("{}m ", minutes).as_str();
    }

    output.trim_end().to_string()
}
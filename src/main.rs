#![feature(int_roundings)]

mod conf;
mod cpu;
mod desktop;
mod displays;
mod gpus;
mod model;
mod packages;
mod shell;
mod terminal;
mod uptime;
mod utils;
mod viuer;

use clap::{arg, command, ArgAction};
use cpu::get_cpus;
use crossterm::{cursor, execute};
use csscolorparser::Color;
use desktop::get_de;
use displays::get_displays;
use gpus::get_gpus;
use image::{imageops, DynamicImage, ImageBuffer, Rgba};
use model::get_model;
use owo_colors::OwoColorize;
use packages::get_packages;
use shell::get_shell;
use std::fmt::Display;
use std::{env, io};
use sysinfo::{MemoryRefreshKind, Pid, ProcessRefreshKind, RefreshKind, System, Users};
use terminal::get_term;
use text_splitter::TextSplitter;
use uptime::get_uptime;

macro_rules! clearScreen {
    ($T:expr) => {
        //clearscreen::clear().unwrap_or(());
        for _ in 0..$T {
            println!();
        }
    };
}

#[derive(Clone)]
struct TermLine {
    label: Option<String>,
    text: String,
    newline_left_pad: usize,
}
impl Display for TermLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.label.is_some() {
            return write!(
                f,
                "{}: {}",
                self.label.as_ref().unwrap().bright_magenta(),
                self.text
            );
        } else {
            return write!(f, "{}", self.text);
        }
    }
}

macro_rules! addLine {
    ($lines:expr, $label:expr, $line:expr) => {
        $lines.push(TermLine {
            label: $label,
            text: $line,
            newline_left_pad: 0,
        })
    };
    ($lines:expr, $label:expr, $line:expr, $leftpad:expr) => {
        $lines.push(TermLine {
            label: $label,
            text: $line,
            newline_left_pad: $leftpad,
        })
    };
}

macro_rules! moveCursor {
    ($x:expr, $y:expr) => {
        execute!(io::stdout(), cursor::MoveTo($x, $y)).expect("Failed to print line");
    };
}
macro_rules! moveCursorX {
    ($x:expr) => {
        execute!(io::stdout(), cursor::MoveToColumn($x)).expect("Failed to print line");
    };
}

struct RGB {
    r: u8,
    g: u8,
    b: u8
}

fn calc_truecolor(w: u16, h: u16, x: u16, y: u16) -> RGB {
    let xf = x as f32 / w as f32;
    let yf = y as f32 / h as f32;

    let mut out = RGB {
        r: 0,
        g: 0,
        b: 0
    };

    out.r = 63 + (192.0 * (1.0-xf)).round() as u8;
    out.g = 63 + (192.0 * xf).round() as u8;
    out.b = 63 + (192.0 * yf).round() as u8;

    return out;
}

fn main() {
    // Get bare minimum system info
    let mut sys = System::new_with_specifics(
        RefreshKind::new().with_processes(ProcessRefreshKind::everything()).with_memory(MemoryRefreshKind::everything()),
    );
    let users = Users::new_with_refreshed_list();
    let current_user =
        if let Some(p) = sys.process(Pid::from_u32(std::os::unix::process::parent_id())) {
            users.get_user_by_id(p.user_id().unwrap())
        } else {
            users.first()
        }
        .unwrap();

    let (term_size_x, term_size_y) = viuer::terminal_size();
    let name_string = format!(
        "{}@{}",
        current_user.name(),
        System::host_name().unwrap_or_else(|| -> String { String::from("?") })
    );

    let args = command!()
        .version(env!("CARGO_PKG_VERSION"))
        .arg(arg!(-i --im <FILE> "Image to display, defaults to none").required(false))
        .arg(arg!(-b --bgc <COLOR> "Any valid css color").required(false))
        .arg(arg!(-c --conf <PATH> "Load a config file").required(false))
        .arg(arg!(-w --colorwidth <WIDTH> "Width of the color blocks (default: 3, 0 to disable)").required(false).value_parser(clap::value_parser!(u16)))
        .arg(arg!(-t --truecolor "Enable truecolor block (will be a minimum of [colorwidth/2 * colorwidth/2])").action(ArgAction::SetTrue))
        .arg(arg!(-u --cpuusage "Enable cpu usage (requires an extra delay, may be slow)").action(ArgAction::SetTrue))
        .get_matches();

    let mut im_w =
        ((term_size_x as f32 / 2.0).floor() - (name_string.len() as f32 / 2.0).floor()) as u32;
    if im_w / 2 > (term_size_y - 2) as u32 {
        im_w = ((term_size_y - 2) * 2) as u32
    }
    let mut im_h = 0 as u32;
    let im_path = args.get_one::<String>("im");
    let bg_color = args.get_one::<String>("bgc");
    let has_im = im_path.is_some();
    let mut lines: Vec<TermLine> = vec![];

    clearScreen!(term_size_y);

    //lines.push(name_string.clone());
    //lines.push(format!("╶{:─<1$}╴", "", name_string.len() - 2));
    //lines.push(format!("{} {}", whoami::distro(), whoami::arch()));

    // Name
    addLine!(
        lines,
        None,
        format!(
            "{}@{}",
            current_user.name().bright_magenta(),
            System::host_name()
                .unwrap_or_else(|| -> String { String::from("?") })
                .bright_magenta()
        )
    );

    // Spacer
    addLine!(lines, None, format!("╶{:─<1$}╴", "", name_string.len() - 2));

    // OS Info
    addLine!(
        lines,
        Some("OS".to_string()),
        format!(
            "{} {} ({})",
            System::distribution_id(),
            System::cpu_arch().unwrap(),
            System::kernel_version().unwrap_or_else(|| { "Unknown kernel".to_string() })
        )
    );

    // Model
    addLine!(lines, Some("Model".to_string()), format!("{}", get_model()));

    // Uptime
    addLine!(lines, Some("Uptime".to_string()), get_uptime());

    // Packages
    // https://github.com/dylanaraps/neofetch/blob/ccd5d9f52609bbdcd5d8fa78c4fdb0f12954125f/neofetch#L1509
    addLine!(lines, Some("Packages".to_string()), get_packages());

    // Terminal
    addLine!(lines, Some("Terminal".to_string()), get_term(&sys));

    // Shell
    addLine!(lines, Some("Shell".to_string()), get_shell(&sys));

    // Desktop env
    addLine!(lines, Some("DE".to_string()), get_de());

    // Displays
    let disps = get_displays();
    let tmp = disps.split("\n");
    addLine!(lines, Some("Displays".to_string()), "".to_string());
    for i in tmp {
        addLine!(lines, None, i.to_string(), 1);
    }

    // CPUs
    let cpus = get_cpus(&mut sys, args.get_one::<bool>("cpuusage").unwrap().clone());
    let tmp: Vec<&str> = cpus.split("\n").collect();
    addLine!(
        lines,
        if tmp.len() > 1 {
            Some(format!("CPUs"))
        } else {
            Some(format!("CPU"))
        },
        "".to_string()
    );
    for i in tmp {
        addLine!(lines, None, i.to_string(), 1);
    }

    // GPUs
    let gpus = get_gpus();
    let tmp: Vec<&str> = gpus.split("\n").collect();
    addLine!(
        lines,
        if tmp.len() > 1 {
            Some(format!("GPUs"))
        } else {
            Some(format!("GPU"))
        },
        "".to_string()
    );
    for i in tmp {
        addLine!(lines, None, i.to_string(), 1);
    }

    // RAM
    addLine!(
        lines,
        Some("RAM".to_string()),
        format!("{:.2}/{:.2}GiB ({:.2}%)", sys.used_memory() as f32/1073741824.0, sys.total_memory() as f32/1073741824.0, sys.used_memory() as f32/sys.total_memory() as f32 * 100.0)
    );
    if sys.total_swap() > 0 {
       addLine!(
           lines,
           Some("SWAP".to_string()),
           format!("{:.2}/{:.2}GiB ({:.2}%)", sys.used_swap() as f32/1073741824.0, sys.total_swap() as f32/1073741824.0, sys.used_swap() as f32/sys.total_swap() as f32 * 100.0)
       );
    }

    // Image
    if has_im {
        moveCursor!(0, 0);
        print!("Loading image...");
        moveCursor!(0, 0);
        print!("                ");
        let im = image::open(im_path.unwrap())
            .expect("Image load failed")
            .to_rgba8();

        let conf = viuer::Config {
            // set offset
            x: 0,
            y: 0,
            // set dimensions
            width: Some(im_w),
            transparent: true,
            ..Default::default()
        };
        let mut bgim = ImageBuffer::from_pixel(
            u32::max(im.width(), im.height()),
            u32::max(im.width(), im.height()),
            Rgba::<u8>([0, 0, 0, 0]),
        );
        if bg_color.is_some() {
            let clr = bg_color.unwrap().parse::<Color>().expect("BG Color error");
            bgim = ImageBuffer::from_pixel(
                u32::max(im.width(), im.height()),
                u32::max(im.width(), im.height()),
                Rgba::<u8>([
                    (clr.r * 255.0) as u8,
                    (clr.g * 255.0) as u8,
                    (clr.b * 255.0) as u8,
                    (clr.a * 255.0) as u8,
                ]),
            );
        }
        if im.width() < im.height() {
            imageops::overlay(&mut bgim, &im, (im.height() / 2 - im.width() / 2) as i64, 0);
        } else if im.width() < im.height() {
            imageops::overlay(&mut bgim, &im, 0, (im.width() / 2 - im.height() / 2) as i64);
        } else {
            imageops::overlay(&mut bgim, &im, 0, 0);
        }

        println!();
        (im_w, im_h) = viuer::print(&DynamicImage::ImageRgba8(bgim), &conf).expect("Shit");
    } else {
        im_w = 0;
    }

    let max_line_size = term_size_x as usize - im_w as usize;
    let mut to_print: Vec<String> = vec![];
    for line in lines {
        let tmp = line.to_string();
        let strs = TextSplitter::new(max_line_size - line.newline_left_pad - 3)
            .chunks(tmp.as_str())
            .collect::<Vec<&str>>();

        if strs.len() == 1 {
            to_print.push(format!("{}{}", " ".repeat(line.newline_left_pad), tmp));
        } else if strs.len() > 1 {
            let strs_enumerated = strs.iter().clone().enumerate();
            let cnt = strs_enumerated.len();
            for (i, splt) in strs_enumerated {
                if i == 0 {
                    to_print.push(format!("{}{}", " ".repeat(line.newline_left_pad), splt));
                } else if i == cnt-1 {
                    to_print.push(format!("{} ╰ {}", " ".repeat(line.newline_left_pad), splt));
                } else {
                    to_print.push(format!("{} │ {}", " ".repeat(line.newline_left_pad), splt));
                }
            }
        }
    }


    // Color block calc
    let colorblockwidth_opt = args.get_one::<u16>("colorwidth");
    let colorblockwidth = if colorblockwidth_opt.is_some() {
        colorblockwidth_opt.unwrap().clone()
    } else {
        3
    } as u32;
    let colorblock_str = " ".repeat(colorblockwidth as usize);
    let has_truecolor = args.get_one::<bool>("truecolor").unwrap().clone();

    // text out
    moveCursor!(0, 0);
    for p in to_print.clone().iter() {
        moveCursorX!((im_w + (1 * ((im_w != 0) as u32))) as u16);
        println!("{}", p)
    }

    // Truecolor out
    if has_truecolor {
        let truecolor_height = u32::max(i32::max(im_h as i32 - to_print.len() as i32 - 5, 0) as u32, colorblockwidth.div_ceil(2)) as u16;
        moveCursor!(
            0,
            (to_print.len() + 1) as u16
        );
        for y in (0..truecolor_height*2).step_by(2) {
            let mut line = "".to_string();
            for x in 0..truecolor_height*2 {
                let bg = calc_truecolor(truecolor_height*2, truecolor_height*2, x, y);
                let fg = calc_truecolor(truecolor_height*2, truecolor_height*2, x, y+1);

                line += "▄".on_truecolor(bg.r, bg.g, bg.b).truecolor(fg.r, fg.g, fg.b).to_string().as_str();
            }
            moveCursorX!((im_w + (1 * ((im_w != 0) as u32))) as u16);
            println!("{}", line);
        }
    } else { // Fill in space not taken by truecolor code
        for _ in 0..i32::max(im_h as i32 - to_print.len() as i32 - 4, 0) {
            println!();
        }
    }

    // Standard colorblock out
    if colorblockwidth > 0 {
        println!();
        moveCursorX!((im_w + (1 * ((im_w != 0) as u32))) as u16);
        println!("{}{}{}{}{}{}{}{}", colorblock_str.on_black(), colorblock_str.on_red(), colorblock_str.on_green(), colorblock_str.on_yellow(), colorblock_str.on_blue(), colorblock_str.on_purple(), colorblock_str.on_cyan(), colorblock_str.on_white());
        moveCursorX!((im_w + (1 * ((im_w != 0) as u32))) as u16);
        println!("{}{}{}{}{}{}{}{}", colorblock_str.on_bright_black(), colorblock_str.on_bright_red(), colorblock_str.on_bright_green(), colorblock_str.on_bright_yellow(), colorblock_str.on_bright_blue(), colorblock_str.on_bright_purple(), colorblock_str.on_bright_cyan(), colorblock_str.on_bright_white());
    }
    // Newline (duhhhh)
    println!();
}

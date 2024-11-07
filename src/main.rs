mod conf;
mod model;
mod uptime;
mod viuer;
mod shell;
mod packages;
mod utils;
mod displays;
mod terminal;
mod desktop;
mod cpu;
mod gpus;

use clap::{arg, command};
use cpu::get_cpus;
use desktop::get_de;
use displays::get_displays;
use packages::get_packages;
use shell::get_shell;
use terminal::get_term;
use core::str;
use crossterm::{cursor, execute};
use csscolorparser::Color;
use image::{imageops, DynamicImage, ImageBuffer, Rgba};
use model::get_model;
use owo_colors::OwoColorize;
use std::{env, io};
use std::fmt::Display;
use sysinfo::{CpuRefreshKind, Pid, ProcessRefreshKind, RefreshKind, System, Users};
use text_splitter::TextSplitter;
use uptime::get_uptime;
use gpus::get_gpus;

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

fn main() {
    // Get bare minimum system info
    let mut sys = System::new_with_specifics(
        RefreshKind::new()
            .with_processes(ProcessRefreshKind::everything())
    );
    let users = Users::new_with_refreshed_list();
    let current_user = if let Some(p) = sys.process(Pid::from_u32(std::os::unix::process::parent_id())) {
        users.get_user_by_id(p.user_id().unwrap())
    } else {
        users.first()
    }.unwrap();

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
        .arg(arg!(-g --genconf <PATH> "Create a default config file").required(false))
        .arg(
            arg!(-p --lowerpad <INT> "Padding amount for lower ")
                .required(false)
                .default_value("3"),
        )
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
    addLine!(
        lines,
        Some("Terminal".to_string()),
        get_term(&sys)
    );

    // Shell
    addLine!(
        lines,
        Some("Shell".to_string()),
        get_shell(&sys)
    );

    // Desktop env
    addLine!(
        lines,
        Some("DE".to_string()),
        get_de()
    );

    // Displays
    let disps = get_displays();
    let tmp = disps.split("\n");
    addLine!(
        lines,
        Some("Displays".to_string()),
        "".to_string()
    );
    for i in tmp {
        addLine!(lines, None, i.to_string(), 1);
    }

    // CPUs
    let cpus = get_cpus(&mut sys);
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

    let max_line_size = term_size_x as usize - im_w as usize - 3 as usize;
    let mut totallines = 0;
    for (i, line) in lines.clone().iter().enumerate() {
        let linestr = format!("{}", line).replace("\n", " ");
        let strs = TextSplitter::new(max_line_size)
            .chunks(linestr.as_str())
            .collect::<Vec<&str>>();
        totallines += strs.len() as u32;
        for (j, s) in strs.iter().enumerate() {
            if j > 0 && j < strs.len() - 1 {
                moveCursor!(
                    (im_w + (1 * ((im_w != 0) as u32))) as u16,
                    i as u16 + j as u16
                );
                println!("{}│ {}", " ".repeat(line.newline_left_pad), s);
            } else if j == 0 {
                moveCursor!((im_w + (1 * ((im_w != 0) as u32))) as u16, i as u16);
                println!("{}{}", " ".repeat(line.newline_left_pad), s);
            } else {
                moveCursor!(
                    (im_w + (1 * ((im_w != 0) as u32))) as u16,
                    i as u16 + j as u16
                );
                println!("{}╰ {}", " ".repeat(line.newline_left_pad), s);
            }
        }
    }

    moveCursor!(
        (im_w - (1 * ((im_w > 0) as u32))) as u16,
        (u32::max(im_h, totallines - 1) - (1 * ((im_w > 0) as u32))) as u16
    );
    println!();

    get_displays();
}

use std::collections::HashMap;

use average::MeanWithError;
use regex::Regex;
use sysinfo::System;

struct CpuCnt {
    count: i64,
    name: String,
    usages: Vec<f64>,      // Percent
    frequencies: Vec<f64>, // GHz
}

pub fn get_cpus(sys: &mut System, show_usage: bool) -> String {
    let mut cpus: Vec<String> = vec![];
    let mut cpu_counter: HashMap<String, CpuCnt> = HashMap::new();

    // Refresh cpu usage
    sys.refresh_cpu_all();
    if show_usage.clone() {
        std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL); // TODO: Add config for how long, default is 200ms but half that works consistently and accurately on my machine ¯\_(ツ)_/¯
        sys.refresh_cpu_all();
    }

    for cpu in sys.cpus() {
        if cpu_counter.contains_key(&cpu.brand().to_string()) {
            cpu_counter.get_mut(&cpu.brand().to_string()).unwrap().count += 1;
            cpu_counter
                .get_mut(&cpu.brand().to_string())
                .unwrap()
                .usages
                .push(cpu.cpu_usage() as f64);
            cpu_counter
                .get_mut(&cpu.brand().to_string())
                .unwrap()
                .frequencies
                .push(cpu.frequency() as f64 / 1000.0);
        } else {
            cpu_counter.insert(
                cpu.brand().to_string(),
                CpuCnt {
                    count: 1,
                    name: cpu.brand().to_string(),
                    usages: vec![cpu.cpu_usage() as f64],
                    frequencies: vec![cpu.frequency() as f64 / 1000.0]
                },
            );
        }
    }

    for (_, cpu) in cpu_counter.iter() {
        let cpu_unneeded_removal = Regex::new(r"(?i)(\{tm\}|\{r\}|CPU|Processor|Dual-Core|Quad-Core|Six-Core|Eight-Core|\d+-core)").unwrap();
        let trimmedname = cpu_unneeded_removal.replace_all(cpu.name.as_str(), "").trim().to_string();

        let freq: MeanWithError = cpu.frequencies.iter().collect();
        let usage: MeanWithError = cpu.usages.iter().collect();

        if show_usage.clone() {
            cpus.push(
                format!("{} ({}) @ {:.1}GHz ({:.1}±{:.1}%)", trimmedname, cpu.count, freq.mean(), usage.mean(), usage.error())
            );
        } else {
            cpus.push(
                format!("{} ({}) @ {:.1}GHz", trimmedname, cpu.count, freq.mean())
            );
        }
    }

    if cpus.len() == 0 {
        return "None... found?".to_string();
    } else if cpus.len() == 1 {
        return "╰ ".to_string() + cpus[0].as_str();
    } else {
        let mut out = "".to_string();
        for (i, mon) in cpus.iter().enumerate() {
            if i == cpus.len() - 1 {
                out.push_str("\n╰ ");
            } else {
                out.push_str("\n│ ");
            }
            out.push_str(mon.as_str());
        }
        return out.trim().to_string();
    }
}

use clap::{crate_name, AppSettings, Arg, Command};
use hashbrown::HashSet;
use std::{
    env,
    fs::{self, OpenOptions},
    io::{stdin, BufWriter, Read, Write},
    path::Path,
    process,
};

const FILESIZE: usize = 102400;
const HHIGNORE: &str = "HHIGNORE";
const HISTIGNORE: &str = "HISTIGNORE";
const HISTTIMEFORMAT: &str = "HISTTIMEFORMAT";
const VERSION: &str = include_str!(concat!(env!("OUT_DIR"), "/VERSION"));

fn parse_hist_ignore() -> HashSet<String> {
    let mut ret = HashSet::new();
    if let Ok(var) = env::var(HISTIGNORE) {
        for var in var.split(':') {
            ret.insert(var.to_owned());
        }
    }

    if let Ok(var) = env::var(HHIGNORE) {
        for var in var.split(':') {
            ret.insert(var.to_owned());
        }
    } 
    ret
}

fn parse_hist_format() -> usize {
    let var =
        env::var(HISTTIMEFORMAT).expect("Failed read the HISTTIMEFORMAT environment variable!");
    let vars: Vec<&str> = var.split_ascii_whitespace().collect();
    if (vars[0] == "%F" || vars[0].contains("%d"))
        && vars[1] == "%T"
        && var.ends_with(char::is_whitespace)
    {
        return vars.len();
    }
    panic!("Failed parse HISTTIMEFORMAT: {:?}", var);
}

fn get_user() -> String {
    if let Ok(var) = env::var("USER") {
        var
    } else {
        process::Command::new("whoami").output().map_or_else(
            |_e| "unknown".into(),
            |v| String::from_utf8(v.stdout).unwrap_or_else(|_e| "unknown".into()),
        )
    }
}

fn ignore(cmd: &[&str], skip: &HashSet<String>) -> bool {
    if cmd[0].starts_with('#') || cmd[0] == crate_name!() {
        true
    } else if cmd[0] == "nohup" || cmd.iter().any(|x| x.contains(&['>', '|'])) {
        false
    } else {
        skip.contains(cmd[0])
    }
}

fn get_last_cmdindex(buf: &[u8]) -> usize {
    let mut index = buf.len() - 1;
    while index > 1 {
        if buf[index - 1] == b'#' && buf[index] == b'>' {
            return index - 1;
        }
        index -= 1;
    }
    0
}

fn read_last_cmd<P: AsRef<Path>>(path: P) -> (Vec<u8>, Vec<usize>) {
    let buf = fs::read(path).expect("Failed read output file!");
    let mut indexs = Vec::new();
    let mut index = buf.len() - 1;
    while index > 1 {
        if buf[index - 1] == b'#' && buf[index] == b'>' {
            break;
        } else if buf[index - 1] == b'\n' {
            indexs.push(index);
        }
        index -= 1;
    }
    (buf, indexs)
}

fn is_dup_cmd(cmd1: &[u8], cmd2: &[&str]) -> bool {
    let mut i = 0;
    for var in cmd2 {
        if *var == "&" || *var == "nohup" {
            continue;
        }
        for v in var.as_bytes() {
            if i < cmd1.len() && *v == cmd1[i] {
                i += 1;
            } else {
                return false;
            }
        }
        while char::is_whitespace(cmd1[i] as char) {
            i += 1;
        }
    }
    i >= cmd1.len() || cmd1[i] == b'#'
}

fn out_info(w: &mut dyn Write) {
    writeln!(
        w,
        "#!/bin/bash\n# This file is generated by {}, see www. for details.",
        crate_name!()
    )
    .expect("Failed write to output file!");
}

fn main() {
    let args = Command::new(crate_name!())
		.version(VERSION)
		.about("Record bash command history in current directory")
		.global_setting(AppSettings::DeriveDisplayOrder)
		.arg(
			Arg::new("INT")
			.default_value("1")
			.help("only record INT valid commands.")
			.takes_value(true)
		)
		.arg(
			Arg::new("no_smart")
			.short('s')
			.help("disable smart mode, commands from the HISTIGNORE environment variable are ignored in smart mode.")
		)
		.arg(
			Arg::new("reset")
			.short('r')
			.help("reset the last insert operation, or delete the last inserted content.")
		)
		.arg(
			Arg::new("index")
			.short('i')
			.value_name("INT")
			.requires("no_smart")
			.help("only record the INT record from last.")
			.takes_value(true)
		)
		.arg(
			Arg::new("output")
			.short('o')
			.value_name("FILE")
			.default_value(&(crate_name!().to_owned() + ".sh"))
			.help("output file name.")
			.takes_value(true)
		)
		.get_matches();

    let valid_n: usize = args.value_of_t("INT").unwrap();
    let no_smart = args.is_present("no_smart");
    let valid_i: usize = args.value_of_t("index").unwrap_or(0);
    let outfile: String = args.value_of_t("output").unwrap();
    let outpath = Path::new(&outfile);
    let is_exists = outpath.exists();

    if is_exists && args.is_present("reset") {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(outpath)
            .expect("Failed open output file!");
        let mut buffer = Vec::with_capacity(FILESIZE);
        file.read_to_end(&mut buffer)
            .expect("Failed read output file!");
        let last_cmdindex = get_last_cmdindex(&buffer);
        if last_cmdindex > 0 {
            file.set_len(last_cmdindex as u64)
                .expect("Failed truncate output file!");
        }
    }else if !atty::is(atty::Stream::Stdin) {
        let last_cmds = if is_exists {
            read_last_cmd(outpath)
        } else {
            (Vec::new(), Vec::new())
        };
        let user = get_user();
        let skip_cmds = parse_hist_ignore();
        let cmd_index = parse_hist_format() + 1;

        let out = OpenOptions::new()
            .append(true)
            .create(true)
            .open(outpath)
            .expect("Failed open output file!");
        let mut w = BufWriter::with_capacity(FILESIZE, out);
        if !is_exists {
            out_info(&mut w);
        }

        let mut input_cmds = String::with_capacity(FILESIZE);
        stdin()
            .read_to_string(&mut input_cmds)
            .expect("Failed read from stdin!");

        let mut n = 0;
        let mut out_sep = false;
        let mut log = String::with_capacity(FILESIZE);
        for cmd in input_cmds.split('\n').rev().skip(1) {
            //skip last line break
            if valid_i > 0 && n != valid_i - 1 {
                continue;
            }
            let cmds: Vec<&str> = cmd.split_ascii_whitespace().collect();
            if cmds.len() <= cmd_index {
                continue;
            }
            let (date, time) = (cmds[1], cmds[2]);
            let cmds = &cmds[cmd_index..];
            if no_smart || !ignore(cmds, &skip_cmds) {
                if !last_cmds
                    .1
                    .iter()
                    .any(|x| is_dup_cmd(&last_cmds.0[*x..], cmds))
                {
                    if !out_sep {
                        writeln!(w, "#> {}", user).expect("Failed write to output file!");
                        out_sep = true;
                    }
                    for opt in cmds {
                        if *opt != "&" && *opt != "nohup" {
                            write!(w, "{} ", opt).expect("Failed write to output file!");
                            log += opt;
                            log.push(' ');
                        }
                    }
                    writeln!(w, " # {} {}", date, time).expect("Failed write to output file!");
                    log.push('\n');
                }
                n += 1;
                if n >= valid_n {
                    break;
                }
            }
        }
        if !log.is_empty() {
            eprintln!("\x1b[1;35m{}\x1b[0m", log);
        }
    }
}

use crate::helper::trim_quotes;
use colored::Colorize;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::process::exit;

#[derive(Debug)]
pub struct Stop<'a> {
    stop_id: &'a str,
    stop_code: &'a str,
    stop_name: &'a str,
    stop_desc: &'a str,
    stop_lat: &'a str,
    stop_long: &'a str,
    zone_id: &'a str,
    plc_url: &'a str,
    location_type: &'a str,
    parent_station: &'a str,
}

impl<'a> Display for Stop<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} at {}, {}",
            self.stop_name.blue(), self.stop_lat.yellow(), self.stop_long.yellow()
        )
    }
}

impl<'a> Stop<'a> {
    pub fn new(
        stop_id: &'a str,
        stop_code: &'a str,
        stop_name: &'a str,
        stop_desc: &'a str,
        stop_lat: &'a str,
        stop_long: &'a str,
        zone_id: &'a str,
        plc_url: &'a str,
        location_type: &'a str,
        parent_station: &'a str,
    ) -> Self {
        Self {
            stop_id: trim_quotes(stop_id),
            stop_code: trim_quotes(stop_code),
            stop_name: trim_quotes(stop_name),
            stop_desc: trim_quotes(stop_desc),
            stop_lat: trim_quotes(stop_lat),
            stop_long: trim_quotes(stop_long),
            zone_id: trim_quotes(zone_id),
            plc_url: trim_quotes(plc_url),
            location_type: trim_quotes(location_type),
            parent_station: trim_quotes(parent_station),
        }
    }

    pub fn parse_stops_file(file_path: &str) -> Vec<Stop<'static>> {
        let mut stops = Vec::new();
        let mut contents = String::new();

        // Read stops file and split at lines
        match File::open(file_path) {
            Ok(mut file) => {
                file.read_to_string(&mut contents)
                    .expect("Can't read file. Does it exist?");
                for line in contents.lines() {
                    let parts: Vec<&str> = line.split(",").collect();
                    stops.push(Stop::new(
                        Box::leak(parts[0].to_string().into_boxed_str()),
                        Box::leak(parts[1].to_string().into_boxed_str()),
                        Box::leak(parts[2].to_string().into_boxed_str()),
                        Box::leak(parts[3].to_string().into_boxed_str()),
                        Box::leak(parts[4].to_string().into_boxed_str()),
                        Box::leak(parts[5].to_string().into_boxed_str()),
                        Box::leak(parts[6].to_string().into_boxed_str()),
                        Box::leak(parts[7].to_string().into_boxed_str()),
                        Box::leak(parts[8].to_string().into_boxed_str()),
                        Box::leak(parts[9].to_string().into_boxed_str()),
                    ));
                }
            }
            Err(_) => {
                println!("{} {} {}", "File".red(), file_path, "not found 🤯".red());
                exit(1)
            }
        }
        stops
    }
}

pub fn old_parse_stops_file(file_path: &str) {
    let mut stops_vec: Vec<String> = Vec::new();

    println!("Reading file at: {}", file_path);

    match File::open(file_path) {
        Ok(file) => {
            for line in BufReader::new(file).lines().map_while(Result::ok) {
                stops_vec.push(line);
            }
        }
        Err(_) => {}
    }
    let mut i = 0;
    for stop in stops_vec {
        println!("Stop {}: {:?}\n", i, stop);
        i += 1;
    }
}

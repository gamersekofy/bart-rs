use std::path::Path;

mod attributes;
mod helper;

use attributes::stops;

fn main() {
    /*let bart_feed = gtfs_structures::Gtfs::new("data.zip");
    let calendar = bart_feed.unwrap().calendar;
    println!("{:?}", calendar);*/
    let mut path = "data/stops.txt";
    let lol: Vec<stops::Stop> = stops::Stop::parse_stops_file(path);

    println!("First stop: {}", lol[1]);
    println!("Last stop: {}", lol[15]);
}

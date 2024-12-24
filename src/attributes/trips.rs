pub struct Trip<'a> {
    route_id: &'a str,
    service_id: &'a str,
    trip_id: &'a str,
    trip_headsign: &'a str,
    direction_id: &'a str,
    block_id: &'a str,
    shape_id: &'a str,
    trip_load_information: &'a str,
    wheelchair_accessible: &'a str,
    bikes_allowed: &'a str
}
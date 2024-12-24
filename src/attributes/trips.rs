use crate::helper::trim_quotes;

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
    bikes_allowed: &'a str,
}

impl<'a> Trip<'a> {
    pub fn new(
        route_id: &'a str,
        service_id: &'a str,
        trip_id: &'a str,
        trip_headsign: &'a str,
        direction_id: &'a str,
        block_id: &'a str,
        shape_id: &'a str,
        trip_load_information: &'a str,
        wheelchair_accessible: &'a str,
        bikes_allowed: &'a str,
    ) -> Self {
        Self {
            route_id,
            service_id,
            trip_id,
            trip_headsign,
            direction_id,
            block_id,
            shape_id,
            trip_load_information,
            wheelchair_accessible,
            bikes_allowed,
        }
    }
}

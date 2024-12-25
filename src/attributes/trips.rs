use crate::helper::trim_quotes;

#[derive(Debug)]
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
            route_id: trim_quotes(route_id),
            service_id: trim_quotes(service_id),
            trip_id: trim_quotes(trip_id),
            trip_headsign: trim_quotes(trip_headsign),
            direction_id: trim_quotes(direction_id),
            block_id: trim_quotes(block_id),
            shape_id: trim_quotes(shape_id),
            trip_load_information: trim_quotes(trip_load_information),
            wheelchair_accessible: trim_quotes(wheelchair_accessible),
            bikes_allowed: trim_quotes(bikes_allowed),
        }
    }
}

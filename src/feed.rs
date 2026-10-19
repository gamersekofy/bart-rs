//! The in-memory representation of a whole GTFS feed.
//!
//! [`Feed`] owns all eighteen tables of a BART feed and the indexes built
//! over them, and answers the cross-file questions the CLI asks: which lines
//! serve a station, when the next train is due, what a trip costs to take.

use std::collections::{BTreeSet, HashMap};
use std::path::Path;

use chrono::NaiveDate;
use serde::Serialize;

use crate::attributes::agency::Agency;
use crate::attributes::calendar::Calendar;
use crate::attributes::calendar_attributes::CalendarAttribute;
use crate::attributes::calendar_dates::CalendarDate;
use crate::attributes::directions::Direction;
use crate::attributes::fare_attributes::FareAttribute;
use crate::attributes::fare_rider_categories::FareRiderCategory;
use crate::attributes::fare_rules::FareRule;
use crate::attributes::feed_info::FeedInfo;
use crate::attributes::realtime_routes::RealtimeRoute;
use crate::attributes::rider_categories::RiderCategory;
use crate::attributes::route_attributes::RouteAttribute;
use crate::attributes::routes::Route;
use crate::attributes::shapes::Shape;
use crate::attributes::stop_times::StopTime;
use crate::attributes::stops::Stop;
use crate::attributes::transfers::Transfer;
use crate::attributes::trips::Trip;
use crate::error::{Error, Result};
use crate::schedule;

/// A train arriving at a stop.
#[derive(Debug, Clone, Serialize)]
pub struct Arrival {
    /// Seconds since midnight of the service day.
    pub time: u32,
    /// The trip that is arriving.
    pub trip_id: String,
    /// The route the trip runs on.
    pub route_id: String,
    /// The route's short name, e.g. `Red-S`.
    pub route_short_name: Option<String>,
    /// Where the train is headed, e.g. `Richmond`.
    pub headsign: Option<String>,
    /// The platform the train arrives at.
    pub platform_id: Option<String>,
    /// The service pattern the trip belongs to.
    pub service_id: String,
    /// The human-readable name of that service pattern, e.g. `Weekday`.
    pub service_description: Option<String>,
}

/// A train departing from a stop.
///
/// Departures carry exactly the same information as arrivals; only the
/// meaning of `time` differs (departure rather than arrival).
pub type Departure = Arrival;

/// The price of a fare for one rider category.
#[derive(Debug, Clone, Serialize)]
pub struct RiderPrice {
    /// The rider category id.
    pub category_id: String,
    /// The human-readable category name, e.g. `Youth Clipper`.
    pub description: String,
    /// The price, in cents.
    pub price_cents: i64,
}

/// A fare and the prices that apply to it.
#[derive(Debug, Clone, Serialize)]
pub struct Fare {
    /// The fare id.
    pub fare_id: String,
    /// The base price, in cents.
    pub base_price_cents: i64,
    /// The currency, e.g. `USD`.
    pub currency: Option<String>,
    /// The per-rider-category prices.
    pub prices: Vec<RiderPrice>,
}

impl Fare {
    /// The base price formatted as a currency string.
    pub fn base_price_display(&self) -> String {
        format!(
            "${}.{:02}",
            self.base_price_cents / 100,
            (self.base_price_cents % 100).abs()
        )
    }
}

/// A whole GTFS feed.
pub struct Feed {
    /// The agency operating the network.
    pub agency: Vec<Agency>,
    /// The recurring service patterns.
    pub calendar: Vec<Calendar>,
    /// Human-readable names for the service patterns.
    pub calendar_attributes: Vec<CalendarAttribute>,
    /// Single-date exceptions to the service patterns.
    pub calendar_dates: Vec<CalendarDate>,
    /// Compass directions per route and direction.
    pub directions: Vec<Direction>,
    /// The price of each fare.
    pub fare_attributes: Vec<FareAttribute>,
    /// Per-rider-category prices.
    pub fare_rider_categories: Vec<FareRiderCategory>,
    /// Which fare applies between which places.
    pub fare_rules: Vec<FareRule>,
    /// Metadata about the feed.
    pub feed_info: Vec<FeedInfo>,
    /// Which routes carry live updates.
    pub realtime_routes: Vec<RealtimeRoute>,
    /// The rider categories BART recognises.
    pub rider_categories: Vec<RiderCategory>,
    /// BART's internal route classification.
    pub route_attributes: Vec<RouteAttribute>,
    /// The commercial lines.
    pub routes: Vec<Route>,
    /// The geographic shapes.
    pub shapes: Vec<Shape>,
    /// The stations, platforms and entrances.
    pub stops: Vec<Stop>,
    /// When each trip stops where.
    pub stop_times: Vec<StopTime>,
    /// Transfer rules between stops.
    pub transfers: Vec<Transfer>,
    /// The individual runs of a route.
    pub trips: Vec<Trip>,

    stops_by_id: HashMap<String, usize>,
    routes_by_id: HashMap<String, usize>,
    trips_by_id: HashMap<String, usize>,
    fare_attributes_by_id: HashMap<String, usize>,
    stop_times_by_trip: HashMap<String, Vec<usize>>,
    stop_times_by_stop: HashMap<String, Vec<usize>>,
    platforms_by_station: HashMap<String, Vec<usize>>,
    entrances_by_station: HashMap<String, Vec<usize>>,
    trips_by_route: HashMap<String, Vec<usize>>,
    service_descriptions: HashMap<String, String>,
    rider_category_names: HashMap<String, String>,
    directions_by_route: HashMap<(String, u8), usize>,
}

impl Feed {
    /// Parse a feed from a directory of `.txt` files.
    ///
    /// The directory must contain the 18 files that make up a BART feed.
    pub fn open(dir: &Path) -> Result<Feed> {
        if !dir.join("feed_info.txt").exists() {
            return Err(Error::FeedNotFound(dir.to_path_buf()));
        }

        let file = |name: &str| dir.join(format!("{name}.txt"));

        let feed = Feed {
            agency: Agency::parse_agency_file(&file("agency"))?,
            calendar: Calendar::parse_calendar_file(&file("calendar"))?,
            calendar_attributes: CalendarAttribute::parse_calendar_attributes_file(&file(
                "calendar_attributes",
            ))?,
            calendar_dates: CalendarDate::parse_calendar_dates_file(&file("calendar_dates"))?,
            directions: Direction::parse_directions_file(&file("directions"))?,
            fare_attributes: FareAttribute::parse_fare_attributes_file(&file("fare_attributes"))?,
            fare_rider_categories: FareRiderCategory::parse_fare_rider_categories_file(&file(
                "fare_rider_categories",
            ))?,
            fare_rules: FareRule::parse_fare_rules_file(&file("fare_rules"))?,
            feed_info: FeedInfo::parse_feed_info_file(&file("feed_info"))?,
            realtime_routes: RealtimeRoute::parse_realtime_routes_file(&file("realtime_routes"))?,
            rider_categories: RiderCategory::parse_rider_categories_file(&file(
                "rider_categories",
            ))?,
            route_attributes: RouteAttribute::parse_route_attributes_file(&file(
                "route_attributes",
            ))?,
            routes: Route::parse_routes_file(&file("routes"))?,
            shapes: Shape::parse_shapes_file(&file("shapes"))?,
            stops: Stop::parse_stops_file(&file("stops"))?,
            stop_times: StopTime::parse_stop_times_file(&file("stop_times"))?,
            transfers: Transfer::parse_transfers_file(&file("transfers"))?,
            trips: Trip::parse_trips_file(&file("trips"))?,

            stops_by_id: HashMap::new(),
            routes_by_id: HashMap::new(),
            trips_by_id: HashMap::new(),
            fare_attributes_by_id: HashMap::new(),
            stop_times_by_trip: HashMap::new(),
            stop_times_by_stop: HashMap::new(),
            platforms_by_station: HashMap::new(),
            entrances_by_station: HashMap::new(),
            trips_by_route: HashMap::new(),
            service_descriptions: HashMap::new(),
            rider_category_names: HashMap::new(),
            directions_by_route: HashMap::new(),
        };

        Ok(feed.build_indexes())
    }

    /// Parse the feed installed in the user's data directory.
    pub fn from_data_dir() -> Result<Feed> {
        Feed::open(&crate::paths::current_feed_dir()?)
    }

    fn build_indexes(mut self) -> Feed {
        self.stops_by_id = self
            .stops
            .iter()
            .enumerate()
            .map(|(i, s)| (s.id.clone(), i))
            .collect();
        self.routes_by_id = self
            .routes
            .iter()
            .enumerate()
            .map(|(i, r)| (r.id.clone(), i))
            .collect();
        self.trips_by_id = self
            .trips
            .iter()
            .enumerate()
            .map(|(i, t)| (t.trip_id.clone(), i))
            .collect();
        self.fare_attributes_by_id = self
            .fare_attributes
            .iter()
            .enumerate()
            .map(|(i, f)| (f.fare_id.clone(), i))
            .collect();

        for (i, stop) in self.stops.iter().enumerate() {
            match stop.location_type {
                crate::attributes::stops::LocationType::StopOrPlatform => {
                    if let Some(parent) = &stop.parent_station {
                        self.platforms_by_station
                            .entry(parent.clone())
                            .or_default()
                            .push(i);
                    }
                }
                crate::attributes::stops::LocationType::EntranceOrExit => {
                    if let Some(parent) = &stop.parent_station {
                        self.entrances_by_station
                            .entry(parent.clone())
                            .or_default()
                            .push(i);
                    }
                }
                _ => {}
            }
        }

        for (i, st) in self.stop_times.iter().enumerate() {
            self.stop_times_by_trip
                .entry(st.trip_id.clone())
                .or_default()
                .push(i);
            self.stop_times_by_stop
                .entry(st.stop_id.clone())
                .or_default()
                .push(i);
        }

        for (i, trip) in self.trips.iter().enumerate() {
            self.trips_by_route
                .entry(trip.route_id.clone())
                .or_default()
                .push(i);
        }

        for attr in &self.calendar_attributes {
            self.service_descriptions
                .insert(attr.service_id.clone(), attr.service_description.clone());
        }
        for cat in &self.rider_categories {
            self.rider_category_names
                .insert(cat.rider_category_id.clone(), cat.description.clone());
        }
        for (i, dir) in self.directions.iter().enumerate() {
            self.directions_by_route
                .insert((dir.route_id.clone(), dir.direction_id), i);
        }

        self
    }

    // --- metadata ---------------------------------------------------------

    /// The feed's own metadata, if it has any.
    pub fn feed_info(&self) -> Option<&FeedInfo> {
        self.feed_info.first()
    }

    /// The agency operating the network, if there is one.
    pub fn publisher(&self) -> Option<&Agency> {
        self.agency.first()
    }

    /// The feed version, if known.
    pub fn version(&self) -> Option<&str> {
        self.feed_info().and_then(|f| f.version.as_deref())
    }

    /// The description of a service pattern, e.g. `Weekday`.
    pub fn service_description(&self, service_id: &str) -> Option<&str> {
        self.service_descriptions
            .get(service_id)
            .map(String::as_str)
    }

    // --- lookups ----------------------------------------------------------

    /// Look up a route by id.
    pub fn route(&self, id: &str) -> Option<&Route> {
        self.routes_by_id.get(id).map(|i| &self.routes[*i])
    }

    /// Look up a route by id or by short name.
    pub fn find_route(&self, query: &str) -> Option<&Route> {
        if let Some(route) = self.route(query) {
            return Some(route);
        }
        let lowered = query.to_ascii_lowercase();
        self.routes.iter().find(|r| {
            r.short_name
                .as_deref()
                .is_some_and(|n| n.eq_ignore_ascii_case(query))
                || r.long_name
                    .as_deref()
                    .is_some_and(|n| n.to_ascii_lowercase() == lowered)
        })
    }

    /// Look up a trip by id.
    pub fn trip(&self, id: &str) -> Option<&Trip> {
        self.trips_by_id.get(id).map(|i| &self.trips[*i])
    }

    /// Look up a stop by id.
    pub fn stop(&self, id: &str) -> Option<&Stop> {
        self.stops_by_id.get(id).map(|i| &self.stops[*i])
    }

    /// Look up a stop by id, stop code, or name.
    pub fn find_stop(&self, query: &str) -> Option<&Stop> {
        if let Some(stop) = self.stop(query) {
            return Some(stop);
        }

        let lowered = query.to_ascii_lowercase();
        self.stops
            .iter()
            .find(|s| {
                s.code
                    .as_deref()
                    .is_some_and(|c| c.eq_ignore_ascii_case(query))
                    || s.name
                        .as_deref()
                        .is_some_and(|n| n.to_ascii_lowercase() == lowered)
            })
            .or_else(|| {
                self.stops.iter().find(|s| {
                    s.name
                        .as_deref()
                        .is_some_and(|n| n.to_ascii_lowercase().contains(&lowered))
                })
            })
    }

    /// Search for stops whose name, id or code contains `query`.
    pub fn find_stops(&self, query: &str) -> Vec<&Stop> {
        let lowered = query.to_ascii_lowercase();
        self.stops
            .iter()
            .filter(|s| {
                s.id.to_ascii_lowercase().contains(&lowered)
                    || s.code
                        .as_deref()
                        .is_some_and(|c| c.to_ascii_lowercase().contains(&lowered))
                    || s.name
                        .as_deref()
                        .is_some_and(|n| n.to_ascii_lowercase().contains(&lowered))
            })
            .collect()
    }

    // --- stop hierarchy ---------------------------------------------------

    /// Every station (`location_type = 1`).
    pub fn stations(&self) -> Vec<&Stop> {
        self.stops.iter().filter(|s| s.is_station()).collect()
    }

    /// Every platform (`location_type = 0`).
    pub fn platforms(&self) -> Vec<&Stop> {
        self.stops.iter().filter(|s| s.is_platform()).collect()
    }

    /// Every entrance or exit (`location_type = 2`).
    pub fn entrances(&self) -> Vec<&Stop> {
        self.stops.iter().filter(|s| s.is_entrance()).collect()
    }

    /// The station a stop belongs to.
    ///
    /// A station is its own station; a platform or an entrance resolves
    /// through its `parent_station`.
    pub fn station_of(&self, stop_id: &str) -> Option<&Stop> {
        let stop = self.stop(stop_id)?;
        if stop.is_station() {
            return Some(stop);
        }
        stop.parent_station.as_deref().and_then(|p| self.stop(p))
    }

    /// The platforms of a station.
    pub fn platforms_of(&self, station_id: &str) -> Vec<&Stop> {
        self.platforms_by_station
            .get(station_id)
            .map(|idx| idx.iter().map(|i| &self.stops[*i]).collect())
            .unwrap_or_default()
    }

    /// The entrances of a station.
    pub fn entrances_of(&self, station_id: &str) -> Vec<&Stop> {
        self.entrances_by_station
            .get(station_id)
            .map(|idx| idx.iter().map(|i| &self.stops[*i]).collect())
            .unwrap_or_default()
    }

    /// The platform ids a stop query should be expanded to.
    ///
    /// A station expands to its platforms, a platform to itself, and an
    /// entrance to the platforms of its station. This is what lets a user
    /// type any of the three and get the right answer.
    pub fn platform_ids_for(&self, stop_id: &str) -> Vec<String> {
        let Some(stop) = self.stop(stop_id) else {
            return Vec::new();
        };

        if stop.is_platform() {
            return vec![stop.id.clone()];
        }
        if stop.is_station() {
            return self
                .platforms_of(&stop.id)
                .into_iter()
                .map(|p| p.id.clone())
                .collect();
        }
        // An entrance: use the platforms of the station it belongs to.
        stop.parent_station
            .as_deref()
            .map(|parent| {
                self.platforms_of(parent)
                    .into_iter()
                    .map(|p| p.id.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    // --- cross-file queries -----------------------------------------------

    /// The routes that serve a stop.
    pub fn routes_serving(&self, stop_id: &str) -> Vec<&Route> {
        let platform_ids = self.platform_ids_for(stop_id);
        let mut route_ids: BTreeSet<&str> = BTreeSet::new();

        for platform in &platform_ids {
            let Some(indices) = self.stop_times_by_stop.get(platform) else {
                continue;
            };
            for index in indices {
                let Some(trip) = self.trips_by_id.get(&self.stop_times[*index].trip_id) else {
                    continue;
                };
                route_ids.insert(&self.trips[*trip].route_id);
            }
        }

        route_ids
            .into_iter()
            .filter_map(|id| self.route(id))
            .collect()
    }

    /// The trips that run a route.
    pub fn trips_on_route(&self, route_id: &str) -> Vec<&Trip> {
        self.trips_by_route
            .get(route_id)
            .map(|idx| idx.iter().map(|i| &self.trips[*i]).collect())
            .unwrap_or_default()
    }

    /// The stop times of a trip, in order.
    pub fn stop_times_for_trip(&self, trip_id: &str) -> Vec<&StopTime> {
        let Some(indices) = self.stop_times_by_trip.get(trip_id) else {
            return Vec::new();
        };
        let mut times: Vec<&StopTime> = indices.iter().map(|i| &self.stop_times[*i]).collect();
        times.sort_by_key(|st| st.stop_sequence);
        times
    }

    /// The stations a route serves, in the order the route visits them.
    ///
    /// BART's routes are directional pairs, so each route has its own stop
    /// order. The sequence is taken from the route's longest trip, which is
    /// the most complete one.
    pub fn stations_on_route(&self, route_id: &str) -> Vec<&Stop> {
        let trips = self.trips_on_route(route_id);
        let best = trips
            .iter()
            .map(|t| self.stop_times_for_trip(&t.trip_id))
            .max_by_key(|times| times.len());

        let Some(times) = best else {
            return Vec::new();
        };

        let mut seen = BTreeSet::new();
        let mut stations = Vec::new();
        for time in times {
            if let Some(stop) = self.stop(&time.stop_id) {
                if let Some(station) = self.station_of(&stop.id) {
                    if seen.insert(station.id.clone()) {
                        stations.push(station);
                    }
                }
            }
        }
        stations
    }

    /// The compass direction of a route direction.
    pub fn direction_of(&self, route_id: &str, direction_id: u8) -> Option<&Direction> {
        self.directions_by_route
            .get(&(route_id.to_string(), direction_id))
            .map(|i| &self.directions[*i])
    }

    /// The BART-internal attributes of a route.
    pub fn route_attributes_of(&self, route_id: &str) -> Option<&RouteAttribute> {
        self.route_attributes
            .iter()
            .find(|a| a.route_id == route_id)
    }

    /// Whether realtime updates are published for a route.
    pub fn realtime_enabled(&self, route_id: &str) -> bool {
        self.realtime_routes
            .iter()
            .any(|r| r.route_id == route_id && r.realtime_enabled)
    }

    // --- schedule ---------------------------------------------------------

    /// The service ids in effect on `date`.
    pub fn service_ids_for(&self, date: NaiveDate) -> BTreeSet<String> {
        schedule::service_ids_for(self, date)
    }

    /// The trains arriving at a stop on a service date, at or after `from`.
    pub fn arrivals_at(&self, stop_id: &str, date: NaiveDate, from: u32) -> Vec<Arrival> {
        self.arrivals_or_departures(stop_id, date, from, false)
    }

    /// The trains departing from a stop on a service date, at or after `from`.
    pub fn departures_from(&self, stop_id: &str, date: NaiveDate, from: u32) -> Vec<Departure> {
        self.arrivals_or_departures(stop_id, date, from, true)
    }

    fn arrivals_or_departures(
        &self,
        stop_id: &str,
        date: NaiveDate,
        from: u32,
        departing: bool,
    ) -> Vec<Arrival> {
        let services = self.service_ids_for(date);
        let platform_ids = self.platform_ids_for(stop_id);
        let mut results: Vec<Arrival> = Vec::new();

        for platform in &platform_ids {
            let Some(indices) = self.stop_times_by_stop.get(platform) else {
                continue;
            };

            for index in indices {
                let stop_time = &self.stop_times[*index];
                let Some(trip_index) = self.trips_by_id.get(&stop_time.trip_id) else {
                    continue;
                };
                let trip = &self.trips[*trip_index];
                if !services.contains(&trip.service_id) {
                    continue;
                }

                let time = if departing {
                    stop_time.departure_time.or(stop_time.arrival_time)
                } else {
                    stop_time.arrival_time.or(stop_time.departure_time)
                };
                let Some(time) = time else { continue };
                if time < from {
                    continue;
                }

                let route = self.route(&trip.route_id);
                results.push(Arrival {
                    time,
                    trip_id: trip.trip_id.clone(),
                    route_id: trip.route_id.clone(),
                    route_short_name: route.and_then(|r| r.short_name.clone()),
                    headsign: trip.trip_headsign.clone(),
                    platform_id: Some(stop_time.stop_id.clone()),
                    service_id: trip.service_id.clone(),
                    service_description: self
                        .service_description(&trip.service_id)
                        .map(str::to_owned),
                });
            }
        }

        results.sort_by(|a, b| {
            a.time
                .cmp(&b.time)
                .then_with(|| a.trip_id.cmp(&b.trip_id))
                .then_with(|| a.platform_id.cmp(&b.platform_id))
        });
        results
    }

    // --- fares ------------------------------------------------------------

    /// The fare between two places.
    ///
    /// Both endpoints may be given as a station id, a platform id, or a name;
    /// they are resolved to station ids, because `fare_rules.txt` is keyed on
    /// stations.
    pub fn fare(&self, from: &str, to: &str) -> Option<Fare> {
        let from_station = self.resolve_station_id(from)?;
        let to_station = self.resolve_station_id(to)?;

        let rule = self
            .fare_rules
            .iter()
            .find(|r| {
                r.origin_id.as_deref() == Some(&from_station)
                    && r.destination_id.as_deref() == Some(&to_station)
            })
            .or_else(|| {
                self.fare_rules.iter().find(|r| {
                    r.origin_id.as_deref() == Some(&to_station)
                        && r.destination_id.as_deref() == Some(&from_station)
                })
            })
            .or_else(|| {
                self.fare_rules.iter().find(|r| {
                    r.origin_id.as_deref() == Some(&from_station)
                        && r.destination_id.as_deref() == Some(&from_station)
                })
            })?;

        let attribute = self
            .fare_attributes_by_id
            .get(&rule.fare_id)
            .map(|i| &self.fare_attributes[*i])?;

        let prices = self
            .fare_rider_categories
            .iter()
            .filter(|c| c.fare_id == rule.fare_id)
            .map(|c| RiderPrice {
                category_id: c.rider_category_id.clone(),
                description: self
                    .rider_category_names
                    .get(&c.rider_category_id)
                    .cloned()
                    .unwrap_or_else(|| c.rider_category_id.clone()),
                price_cents: c.price_cents,
            })
            .collect();

        Some(Fare {
            fare_id: rule.fare_id.clone(),
            base_price_cents: attribute.price_cents,
            currency: attribute.currency_type.clone(),
            prices,
        })
    }

    /// Resolve a user-supplied place to a station id.
    pub fn resolve_station_id(&self, query: &str) -> Option<String> {
        let stop = self.find_stop(query)?;
        self.station_of(&stop.id).map(|s| s.id.clone())
    }
}

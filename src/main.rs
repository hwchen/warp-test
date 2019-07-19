use warp::{path, Filter};
use std::str::FromStr;

fn main() {
    // declare routes

    // GET /
    let root = warp::path::end()
        .map(|| "root");

    // GET /cubes
    let cubes = path!("cubes")
        .and(warp::path::end())
        .map(|| "metadata for all cubes");

    // GET /cubes/<cube_name>
    let cube = path!("cubes" / String)
        .and(warp::path::end())
        .map(|cube_name| {
            format!("metadata for {}", cube_name)
        });

    // GET /cubes/<cube_name>/aggregate<.fmt>
    let aggregate = path!("cubes" / String / AggregateRoute)
        .and(warp::path::end())
        .map(|cube_name, format| {
            format!("aggregate for {}, {:?}", cube_name, format)
        });

    let routes = root
        .or(cubes)
        .or(cube)
        .or(aggregate);

    warp::serve(routes).run(([127,0,0,1], 3030));
}

#[derive(Debug)]
pub struct AggregateRoute {
    format_type: FormatType,
}

impl FromStr for AggregateRoute {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, ()> {
        let mut agg_and_fmt = s.split('.');

        if let Some(agg) = agg_and_fmt.next() {
            if agg != "aggregate" {
                return Err(());
            }
        }

        let fmt = if let Some(fmt) = agg_and_fmt.next() {
            fmt.parse()?
        } else {
            FormatType::default()
        };

        Ok(Self {
            format_type: fmt,
        })
    }
}

#[derive(Debug)]
pub enum FormatType {
    Csv,
    JsonRecords
}

impl FromStr for FormatType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, ()> {
        match s {
            "csv" => Ok(FormatType::Csv),
            "jsonrecords" => Ok(FormatType::JsonRecords),
            _ => Err(())
        }
    }
}

impl Default for FormatType {
    fn default() -> Self {
        FormatType::Csv
    }
}

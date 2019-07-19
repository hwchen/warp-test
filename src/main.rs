use warp::{path, Filter};
use std::str::FromStr;
use std::sync::{Arc, RwLock};

fn main() {
    // set up state
    // use a Filter to be able to easily combine with others.
    let schema = Arc::new(RwLock::new(Schema::new()));
    let schema = warp::any().map(move || schema.clone());

    // ====================================================
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
        .and(schema.clone())
        .map(|cube_name, schema: Arc<RwLock<Schema>>| {
            let schema = schema.read().unwrap();

            format!("metadata for {}, schema {:?}",
                    cube_name,
                    schema,
                    )
        });

    // GET /cubes/<cube_name>/aggregate<.fmt>
    let aggregate = path!("cubes" / String / AggregateRoute)
        .and(warp::path::end())
        .and(schema.clone())
        .map(|cube_name, format, schema: Arc<RwLock<Schema>>| {
            let schema = schema.read().unwrap();

            format!("aggregate for {}, {:?}; schema {:?}",
                cube_name,
                format,
                schema,
                )
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

#[derive(Debug)]
pub struct Schema(String);

impl Schema {
    pub fn new() -> Self{
        Schema("A Schema".into())
    }
}


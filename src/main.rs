use futures_util::future;
use log::{debug, error};
use pretty_env_logger;
use std::error::Error as StdError;
use std::fmt::{self, Display};
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::time::delay_for;
use warp::{
    http::{
        status,
        Response,
    },
    path,
    Filter,
    Rejection,
};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

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
    let aggregate = warp::path("cubes")
        .and(warp::path::param::<String>())
        .and(warp::path::param::<AggregateRoute>()) // want _param_with_err, but it's in PR right now
        .and(warp::path::end())
        .and(schema.clone())
        .and_then(handle_aggregate)
        .recover(|err: Rejection| {
            let err = {
                if let Some(e) = err.find::<Error>() {
                    error!("Error: {}", e);
                    Ok(Response::builder()
                    .status(status::StatusCode::from_u16(404).unwrap())
                    .body(e.to_string())
                    )
                } else {
                    Err(err)
                }
            };
            future::ready(err)
        });

    let routes = root
        .or(cubes)
        .or(cube)
        .or(aggregate)
        .with(warp::log("warp-test"));

    warp::serve(routes).run(([127,0,0,1], 3030)).await;
}

async fn handle_aggregate(cube_name: String, agg: AggregateRoute, schema: Arc<RwLock<Schema>>) -> Result<impl warp::Reply, warp::Rejection> {
    delay_for(Duration::from_secs(2))
        .await;

    let schema = schema.read().unwrap();

    let res = format!("agg for {}, {:?}; schema {:?}",
        cube_name,
        agg,
        schema,
    );
    debug!("{}", res);

    Ok(res)
}

#[derive(Debug)]
pub struct AggregateRoute {
    format_type: FormatType,
}

impl FromStr for AggregateRoute {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut agg_and_fmt = s.split('.');

        if let Some(agg) = agg_and_fmt.next() {
            if agg != "aggregate" {
                return Err(Error { msg: "".into() });
            }
        }

        let fmt = if let Some(fmt) = agg_and_fmt.next() {
            fmt.parse()
                .map_err(|e| Error {
                    msg: format!("{}", e),
                })?
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
    type Err = String;

    fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "csv" => Ok(FormatType::Csv),
            "jsonrecords" => Ok(FormatType::JsonRecords),
            _ => Err(format!("format {:?} not supported", s))
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

#[derive(Debug)]
pub struct Error {
    msg: String,
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.msg)
    }
}

impl StdError for Error {}
impl warp::reject::Reject for Error {}


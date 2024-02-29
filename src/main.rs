mod parse_manage;
mod data_structs;
mod async_inner_functions;
mod base_and_timer;

use std::sync::{Arc};
use mysql::{PooledConn};
use tokio::sync::Mutex;
use warp::{Filter};
use async_inner_functions::refuse_connection;
use crate::base_and_timer::establish_connection;
use crate::data_structs::CityWithEvent;

#[tokio::main]
async fn main() {

    let cityWithEvents : Arc<Mutex<Vec<CityWithEvent>>> = Arc::new(Mutex::new(Vec::new()));
    let database_keys : Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let transfer : Arc<Mutex<Vec<data_structs::GroupEvent>>> = Arc::new(Mutex::new(Vec::new()));
    let pool : Arc<Mutex<PooledConn>> = Arc::new(Mutex::new(establish_connection()));

    base_and_timer::timer_updatable(Arc::clone(&transfer), Arc::clone(&cityWithEvents)).await;
    base_and_timer::update_keygen(Arc::clone(&database_keys), Arc::clone(&pool)).await;
    base_and_timer::refresh_connection(Arc::clone(&pool)).await;

    let total_events = warp::path!("api" / "total")
        .and(warp::get())
        .and(warp::header::<String>("Keygen"))
        .and(warp::header::<String>("User-Agent"))
        .and(warp::header::<String>("Unique-ID")) // Added this header, so I could track the ID of the device
        .and(async_inner_functions::with_params(Arc::clone(&transfer)))
        .and(async_inner_functions::with_base(Arc::clone(&database_keys)))
        .and(async_inner_functions::with_pool(Arc::clone(&pool)))
        .and_then(async_inner_functions::return_collected);

    let filtered_events = warp::path!("api" / "filter")
        .and(warp::post())
        .and(warp::body::json())
        .and(warp::header::<String>("Keygen"))
        .and(warp::header::<String>("User-Agent"))
        .and(warp::header::<String>("Unique-ID"))
        .and(async_inner_functions::with_params(Arc::clone(&transfer)))
        .and(async_inner_functions::with_base(Arc::clone(&database_keys)))
        .and(async_inner_functions::with_pool(Arc::clone(&pool)))
        .and_then(async_inner_functions::return_filtered);

    let available_places = warp::path!("api" / "available_places")
        .and(warp::get())
        .and(warp::header::<String>("Keygen"))
        .and(warp::header::<String>("User-Agent"))
        .and(warp::header::<String>("Unique-ID"))
        .and(async_inner_functions::with_base(Arc::clone(&database_keys)))
        .and(async_inner_functions::with_pool(Arc::clone(&pool)))
        .and(async_inner_functions::with_crossed(Arc::clone(&cityWithEvents)))
        .and_then(async_inner_functions::return_available_cities);

    let refuse_connection = warp::any().and(warp::method()).and_then(refuse_connection); // Refuse connections that don't match other routes.

    println!("Server is initialized.\nDeployment address : http://localhost:8000/");

    let routes = total_events.or(filtered_events).or(available_places).or(refuse_connection);

    warp::serve(routes).run(([0, 0, 0, 0], 8000)).await;
}
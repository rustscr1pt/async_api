mod parse_manage;
mod data_structs;
mod async_inner_functions;
mod base_and_timer;

use std::sync::{Arc};
use mysql::PooledConn;
use tokio::sync::Mutex;
use warp::{Filter};
use crate::base_and_timer::establish_connection;

#[tokio::main]
async fn main() {
    let database_keys : Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    let update_base = Arc::clone(&database_keys);
    let return_check = Arc::clone(&database_keys);
    let filter_check = Arc::clone(&database_keys);
    let available_check = Arc::clone(&database_keys);

    let transfer : Arc<Mutex<Vec<data_structs::GroupEvent>>> = Arc::new(Mutex::new(Vec::new()));

    let updatable_link = Arc::clone(&transfer);
    let return_link = Arc::clone(&transfer);
    let filter_link = Arc::clone(&transfer);
    let available_link = Arc::clone(&transfer);

    let pool : Arc<Mutex<PooledConn>> = Arc::new(Mutex::new(establish_connection()));

    let updatable_pool : Arc<Mutex<PooledConn>> = Arc::clone(&pool); // For refreshing the connection
    let return_pool : Arc<Mutex<PooledConn>> = Arc::clone(&pool); // For WARP
    let filter_pool : Arc<Mutex<PooledConn>> = Arc::clone(&pool); // For WARP
    let available_pool : Arc<Mutex<PooledConn>> = Arc::clone(&pool); // For WARP
    let keygen_pool : Arc<Mutex<PooledConn>> = Arc::clone(&pool); // For keygen

    base_and_timer::timer_updatable(updatable_link).await;

    base_and_timer::update_keygen(update_base, keygen_pool).await;

    base_and_timer::refresh_connection(updatable_pool).await;

    let total_events = warp::path!("api" / "total")
        .and(warp::get())
        .and(warp::header::<String>("Keygen"))
        .and(warp::header::<String>("User-Agent"))
        .and(async_inner_functions::with_params(return_link)) // test before using for stability
        .and(async_inner_functions::with_base(return_check))
        .and(async_inner_functions::with_pool(return_pool))
        .and_then(async_inner_functions::return_collected);

    let filtered_events = warp::path!("api" / "filter")
        .and(warp::post())
        .and(warp::body::json())
        .and(warp::header::<String>("Keygen"))
        .and(warp::header::<String>("User-Agent"))
        .and(async_inner_functions::with_params(filter_link))
        .and(async_inner_functions::with_base(filter_check))
        .and(async_inner_functions::with_pool(filter_pool))
        .and_then(async_inner_functions::return_filtered);

    let available_places = warp::path!("api" / "available_places")
        .and(warp::get())
        .and(warp::header::<String>("Keygen"))
        .and(warp::header::<String>("User-Agent"))
        .and(async_inner_functions::with_params(available_link))
        .and(async_inner_functions::with_base(available_check))
        .and(async_inner_functions::with_pool(available_pool))
        .and_then(async_inner_functions::return_available_cities);

    println!("Server is initialized.");

    let routes = total_events.or(filtered_events).or(available_places);

    warp::serve(routes).run(([0, 0, 0, 0], 8000)).await;
}
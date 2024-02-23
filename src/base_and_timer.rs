use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::sleep;
use std::time::Duration;
use chrono::Local;
use tokio::task::{JoinHandle};
use mysql::*;
use mysql::prelude::Queryable;
use crate::{data_structs, parse_manage};
use crate::data_structs::{CityWithEvent, GroupEvent, KeyObjected};
use itertools::Itertools;


pub async fn timer_updatable(updatable_link : Arc<Mutex<Vec<GroupEvent>>>, cityevent_link : Arc<Mutex<Vec<CityWithEvent>>>) -> () {
    tokio::spawn(async move {
        let mut timer : u8 = 60;
        loop {
            if timer == 0 {
                let cloned_updatable = Arc::clone(&updatable_link);
                let cloned_events = Arc::clone(&cityevent_link);
                tokio::spawn(async move {
                    let mut processed : Vec<JoinHandle<()>> = Vec::new();
                    println!("Started loading the data at {}\n", Local::now().format("[%Y-%m-%d][%H:%M:%S]"));
                    let mut answer = parse_manage::na_collected().await; // Get main page with a check for connection. Retry till it's fine.
                    let copied = answer.clone();
                    let to_fill : Arc<Mutex<Vec<data_structs::MapsToMatch>>> = Arc::new(Mutex::new(Vec::new()));
                    for links in copied {
                    let fill_copy = Arc::clone(&to_fill);
                    let spawned_thread = tokio::spawn(async move {
                        println!("Spawned a thread for : {}", links.link);
                        let retrieved = parse_manage::retrieve_maps(links.link).await;
                        let mut unlocked = fill_copy.lock().await;
                        println!("{:#?}", retrieved);
                        unlocked.push(retrieved);
                    });
                    processed.push(spawned_thread)
                    }
                    futures::future::join_all(processed).await;
                    for elements in to_fill.lock().await.iter() {
                    for control in answer.iter_mut() {
                        if elements.link.to_string() == control.link.to_string() {
                            control.yandex_maps = elements.map.to_string();
                            control.subway_colored = elements.subway.clone();
                        }
                    }
                }
                    let mut opened_value = cloned_updatable.lock().await;
                    *opened_value = answer;
                    println!("{:#?}", *opened_value);
                    let cloned_group = opened_value.clone();
                    drop(opened_value);
                    format_the_final(cloned_group, cloned_events).await;
                });
                timer = 60;
            }
            else {
                println!("Time estimated till update : {}", timer);
                sleep(Duration::from_secs(1)).await;
                timer -= 1;
            }
        }
    });
}

pub async fn update_keygen(database : Arc<Mutex<Vec<String>>>, stable_connection : Arc<Mutex<PooledConn>>) -> () {
    tokio::spawn(async move {
        let mut timer : u16 = 0;
        loop {
            if timer == 0 {
                let mut connection = stable_connection.lock().await;
                let arrived : Vec<String> = connection.query_map("SELECT keypass FROM keygens",
                                                                      |key| {
                                                                          KeyObjected {
                                                                              key
                                                                          }
                                                                      }
                ).unwrap().iter().map(|element| element.key.to_string()).collect::<Vec<String>>();
                println!("{:#?}", arrived);
                println!("Database is refreshed at {}", Local::now().format("[%Y-%m-%d][%H:%M:%S]"));
                let mut unlocked = database.lock().await;
                *unlocked = arrived;
                drop(connection);
                drop(unlocked);
                timer = 900
            }
            else {
                sleep(Duration::from_secs(1)).await;
                timer -= 1;
            }
        }
    });
}

pub async fn refresh_connection(to_refresh : Arc<Mutex<PooledConn>>) -> () {
    tokio::spawn(async move {
        let mut timer : u8 = 60;
        loop {
            if timer == 0 {
                let url = r#"mysql://gen_user:U\3+)5,,bGwcsM@94.241.169.12/default_db"#;
                let pool = Pool::new(url).expect("Couldn't connect to a base").get_conn().unwrap();
                let mut unlocked = to_refresh.lock().await;
                *unlocked = pool;
                drop(unlocked);
                println!("Pooled connection was refreshed at {}", Local::now().format("[%Y-%m-%d][%H:%M:%S]"));
                timer = 60
            }
            else {
                sleep(Duration::from_secs(1)).await;
                timer -= 1;
            }
        }
    });
}

pub fn establish_connection() -> PooledConn {
    let url = r#"mysql://gen_user:U\3+)5,,bGwcsM@94.241.169.12/default_db"#;
    let pool = Pool::new(url).expect("Couldn't connect to a base");
    println!("Connection with MySQL pool is established!");
    return pool.get_conn().unwrap();
}


pub async fn format_the_final(events_vec : Vec<GroupEvent>, cityEventsVec : Arc<Mutex<Vec<CityWithEvent>>>) -> () {

    clear_stocks(Arc::clone(&cityEventsVec)).await; // Clear the events vec before adding new data.

    let cities_vec = events_vec.iter().map(|element | element.city.to_string()).collect::<Vec<String>>().into_iter().unique().collect::<Vec<String>>();
    let mut threads_holder : Vec<JoinHandle<()>> = Vec::new();
    for city in cities_vec {
        let cloned_events = events_vec.clone();
        let cloned_result = Arc::clone(&cityEventsVec);
        let thread = tokio::spawn(async move {
            let filtered_by_city = cloned_events.into_iter().filter(|stack| stack.city == city).collect::<Vec<GroupEvent>>();
            let total_scheduled = filtered_by_city.len();
            let next_event = filtered_by_city.get(0).expect("Couldn't get a first element").clone();
            let mut unlocked = cloned_result.lock().await;
            unlocked.push(CityWithEvent{ cityname: city, total_count : total_scheduled as u16, firstevent: next_event});
            drop(unlocked)
        });
        threads_holder.push(thread);
    }
    futures::future::join_all(threads_holder).await;
}

pub async fn clear_stocks(stock : Arc<Mutex<Vec<CityWithEvent>>>) -> () {
    println!("Clearing the CityEvents vector");
    let mut unlocked = stock.lock().await;
    unlocked.clear()
}
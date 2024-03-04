use std::sync::Arc;
use mysql::{PooledConn, params};
use mysql::prelude::Queryable;
use tokio::sync::Mutex;
use crate::{data_structs};
use crate::data_structs::{AVAILABLE_REQUEST, Cities, CityWithEvent, FILTER_REQUEST, GET_REQUEST, GroupEvent, Rejected, Themes, VisitorData};
use warp::{reply::json, Filter, Rejection, Reply};
use warp::http::Method;

type WebResult<T> = Result<T, Rejection>;

pub async fn return_collected(key : String, agent : String, id : String, request : Arc<Mutex<Vec<GroupEvent>>>, base : Arc<Mutex<Vec<String>>>, pool : Arc<Mutex<PooledConn>>) -> WebResult<impl Reply> {
    match check_key(key, base).await {
        (true, t) => {
            let result = request.lock().await;
            if result.len() == 0 {
                let mut returned_value : Vec<GroupEvent> = Vec::new();
                returned_value.push(GroupEvent {
                    group_name : "Null".to_string(),
                    place : "Null".to_string(),
                    time : "Null".to_string(),
                    schedule : "Null".to_string(),
                    link : "Null".to_string(),
                    city : "Null".to_string(),
                    thematics : Themes {
                        theme : Vec::new()
                    },
                    yandex_maps : "Null".to_string(),
                    subway_colored : Vec::new()
                    });
                    write_stats("GET", pool, GET_REQUEST, true, t, agent, id).await;
                    return Ok(json(&returned_value))
            }
            write_stats("GET", pool, GET_REQUEST, true, t, agent, id).await;
            return Ok(json(&*result)) // If more than 1 element return VEC of GroupEvent
            }
        (false, t) => {
            write_stats("GET", pool, GET_REQUEST, false, t, agent, id).await;
            println!("Wrong keygen. Please use another one!")
        }
    }
    return Ok(json(&Rejected {
        reply : "Wrong keygen. Please use another one!".to_string()
    }))
}

pub async fn return_filtered(body : data_structs::FilterRequest, key : String, agent : String, id : String, request : Arc<Mutex<Vec<GroupEvent>>>, base : Arc<Mutex<Vec<String>>>, pool : Arc<Mutex<PooledConn>>) -> WebResult<impl Reply> {
    match check_key(key, base).await {
        (true, t) => {
           let response = request.lock().await;
           println!("Answered for a filter POST request, Key == True");
           let result = response.clone().into_iter().filter(|element| element.city.to_string() == body.filter_query.to_string()).collect::<Vec<GroupEvent>>();
            drop(response); // Release the lock
           write_stats("POST", pool, FILTER_REQUEST, true, t, agent, id).await;
           return Ok(json(&result)) },
        (false, t) => {
            write_stats("POST", pool, FILTER_REQUEST, false, t, agent, id).await;
            println!("Wrong keygen. Please use another one!");
        }
    }
    return Ok(json(&Rejected {
        reply : "Wrong keygen. Please use another one!".to_string()
    }))
}

pub async fn return_available_cities(key : String, agent : String, id : String, base : Arc<Mutex<Vec<String>>>, pool : Arc<Mutex<PooledConn>>, cross_link : Arc<Mutex<Vec<CityWithEvent>>>) -> WebResult<impl Reply> {
    match check_key(key, base).await {
        (true, t) => {
            let reply = cross_link.lock().await;
            let copy_reply = reply.clone();
            drop(reply);

            write_stats("GET", pool, AVAILABLE_REQUEST, true, t, agent, id).await;
            return Ok(json(&Cities {
                cities : copy_reply
            }))
        }
        (false, t) => {
            write_stats("GET", pool, AVAILABLE_REQUEST, false, t, agent, id).await;
            println!("Wrong key")
        }
    }
    return Ok(json(&Rejected {
        reply : "Wrong keygen. Please use another one!".to_string()
    }))
}

pub async fn refuse_connection(_ : Method) -> WebResult<impl Reply> {
    return Ok(json(&Rejected {
        reply: "Access denied. The connection is dropped.".to_string(),
    }))
}

async fn check_key(checked : String, object : Arc<Mutex<Vec<String>>>) -> (bool, String) {
    let unlocked = object.lock().await;
    if unlocked.iter().any(|object| *object == checked) {
        drop(unlocked);
        return (true, checked)
    }
    else {
        drop(unlocked);
        return (false, "REJECTED".to_string())
    }
}

async fn write_stats(http_method : &str, pool : Arc<Mutex<PooledConn>>, request : &str, is_success : bool, used_key : String, agent : String, id : String) -> () {
    let http_method : &str = http_method;

    let key_approved : &str = match is_success {
        true => "TRUE",
        false => "FALSE"
    };

    let object_vec = vec![VisitorData {
        http_method: http_method,
        request_type: request,
        user_agent: agent.as_str(),
        device_id: id.as_str(),
        key_approved: key_approved,
        used_key: used_key.as_str(),
    }];

    let mut connection = pool.lock().await;

    match connection.exec_batch(r"INSERT INTO requests (http_method, request_type, user_agent, device_id, key_approved, used_key, time_happened) values (:http_method, :request_type, :user_agent, :device_id, :key_approved, :used_key, now())",
object_vec.iter().map(|object| params! {
    "http_method" => object.http_method,
    "request_type" => object.request_type,
    "user_agent" => object.user_agent,
    "device_id" => object.device_id, // Added here but not active in the database
    "key_approved" => object.key_approved,
    "used_key" => object.used_key
})
) {
        Ok(_) => println!("Data has been written to the database."),
        Err(e) => println!("Error : {}", e)
    }
}
use serde::{Deserialize, Serialize};

pub const GET_REQUEST : &'static str = "GET EVENTS";
pub const FILTER_REQUEST : &'static str = "FILTERED EVENTS";
pub const AVAILABLE_REQUEST : &'static str = "AVAILABLE CITIES";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GroupEvent {
    pub group_name : String,
    pub place : String,
    pub time : String,
    pub schedule : String,
    pub link : String,
    pub city : String,
    pub thematics : Themes,
    pub yandex_maps : String,
    pub subway_colored : Vec<SubwayColors>
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Cities {
    pub cities : Vec<CityWithEvent>
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CityWithEvent {
    pub cityname : String,
    pub total_count : u16,
    pub firstevent : GroupEvent
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Themes {
    pub theme : Vec<String>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Start {
    pub array : Vec<GroupEvent>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FilterRequest {
    pub filter_query : String
}

#[derive(Debug)]
pub struct MapsToMatch {
    pub link : String,
    pub map : String,
    pub subway : Vec<SubwayColors>
}

#[derive(Debug, Serialize)]
pub struct Rejected {
    pub reply : String
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SubwayColors {
    pub subway : String,
    pub color : String
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct KeyObjected {
    pub key : String
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisitorData<'a> {
    pub http_method : &'a str,
    pub request_type : &'a str,
    pub user_agent : &'a str,
    pub device_id : &'a str,
    pub key_approved : &'a str,
    pub used_key : &'a str,
}

pub fn with_params(request : Arc<Mutex<Vec<GroupEvent>>>) -> impl Filter<Extract = (Arc<Mutex<Vec<GroupEvent>>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || request.clone())
}

pub fn with_base(base : Arc<Mutex<Vec<String>>>) -> impl Filter<Extract = (Arc<Mutex<Vec<String>>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || base.clone())
}

pub fn with_pool(pool : Arc<Mutex<PooledConn>>) -> impl Filter<Extract = (Arc<Mutex<PooledConn>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move ||  pool.clone())
}

pub fn with_crossed(cross : Arc<Mutex<Vec<CityWithEvent>>>) -> impl Filter<Extract = (Arc<Mutex<Vec<CityWithEvent>>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || cross.clone())
}
use warp::Filter;
use std::sync::Arc;
use crate::{executor, storage};
use serde_json::Value;

pub fn server(
    storage: Arc<storage::Storage>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let register = warp::post()
        .and(warp::path("register"))
        .and(warp::body::json())
        .and(with_storage(storage.clone()))
        .and_then(register_function);
    let invoke = warp::post()
        .and(warp::path("invoke"))
        .and(warp::body::json())
        .and(with_storage(storage.clone()))
        .and_then(invoke_function);
    register.or(invoke)
}

fn with_storage(
    storage: Arc<storage::Storage>,
) -> impl Filter<Extract = (Arc<storage::Storage>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || storage.clone())
}

async fn register_function(
    body: Value,
    storage: Arc<storage::Storage>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let function_name = body["name"].as_str().ok_or_else(warp::reject::not_found)?;
    let code = body["code"].as_str().ok_or_else(warp::reject::not_found)?;
    storage
        .save_function(function_name.to_string(), code.to_string())
        .map_err(|_| warp::reject::custom(warp::reject()))?;
    Ok(warp::reply::json(&format!("Function {} registered!", function_name)))
}

async fn invoke_function(
    body: Value,
    storage: Arc<storage::Storage>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let function_name = body["name"].as_str().ok_or_else(warp::reject::not_found)?;
    let input = body["input"].as_array().ok_or_else(warp::reject::not_found)?;
    let code = storage.load_function(function_name).map_err(|_| warp::reject::not_found())?;
    let result = executor::execute(&code, function_name, input).map_err(|_| warp::reject::not_found())?;
    Ok(warp::reply::json(&result))
}

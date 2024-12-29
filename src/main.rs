mod api;
mod executor;
mod storage;

use std::sync::Arc;

#[tokio::main]
async fn main() {
    let storage = Arc::new(storage::Storage::init().unwrap());
    let routes = api::server(storage);

    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}

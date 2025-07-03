use axum::http::Request;
use axum::{
    Router,
    routing::{delete, get, post, put},
};
use dotenv::dotenv;
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};
use tower::ServiceBuilder;
use tower_http::{
    ServiceBuilderExt,
    request_id::{MakeRequestId, RequestId},
    trace::{DefaultOnResponse, TraceLayer},
};
use tracing::{Level, info, info_span};
mod api;
mod db;
#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();
    let pool = db::establish_connection().await;

    let app = Router::new()
        .route("/api/wx_counter/login", post(api::user::login))
        .route("/api/wx_counter/counters", get(api::counter::list))
        .route("/api/wx_counter/counters", post(api::counter::add))
        .route("/api/wx_counter/counters/{id}", get(api::counter::show))
        .route("/api/wx_counter/counters/{id}", put(api::counter::update))
        .route(
            "/api/wx_counter/counters/{id}",
            delete(api::counter::destroy),
        )
        .route("/api/wx_counter/counters/top/{id}", post(api::counter::top))
        .route(
            "/api/wx_counter/counters_records/{id}",
            post(api::counter_record::add),
        )
        .route(
            "/api/wx_counter/counters_records/{id}",
            get(api::counter_record::list),
        )
        .layer(
            ServiceBuilder::new()
                .set_x_request_id(MyMakeRequestId::default())
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(
                            |request: &Request<_>|{
                                let reqid = request.headers()
                                    .get("x-request-id")
                                    .map(|v| v.to_str().unwrap_or(""))
                                    .unwrap_or("");
                                info_span!("request", method = %request.method(),uri = %request.uri(),reqid = ?reqid,)
                            })
                        .on_response(
                            DefaultOnResponse::new()
                                .level(Level::INFO)
                                .include_headers(true),
                        ),
                )
                // propagate the header to the response before the response reaches `TraceLayer`
                .propagate_x_request_id(),
        )
        .with_state(pool);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3132")
        .await
        .unwrap();
    info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

#[derive(Clone, Default)]
struct MyMakeRequestId {
    counter: Arc<AtomicU64>,
}

impl MakeRequestId for MyMakeRequestId {
    fn make_request_id<B>(&mut self, request: &Request<B>) -> Option<RequestId> {
        let request_id = self
            .counter
            .fetch_add(1, Ordering::SeqCst)
            .to_string()
            .parse()
            .unwrap();

        Some(RequestId::new(request_id))
    }
}

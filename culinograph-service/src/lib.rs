mod auth;
mod error;
mod models;
mod routes;
mod state;
mod ws;

pub use auth::AccessPolicy;
use axum::{
    Json, Router,
    http::{HeaderValue, Method, header},
    middleware,
    routing::{delete, get, post, put},
};
pub use error::ApiError;
pub use state::ServiceState;
use std::{io, net::SocketAddr};
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    trace::TraceLayer,
};
pub use ws::WebSocketState;

#[derive(Clone)]
pub struct ServiceConfig {
    pub state: ServiceState,
    pub access: AccessPolicy,
    pub allowed_origins: Vec<String>,
}

pub struct BoundService {
    listener: TcpListener,
    router: Router,
}

impl BoundService {
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.listener.local_addr()
    }

    pub async fn serve(self, shutdown: CancellationToken) -> io::Result<()> {
        axum::serve(self.listener, self.router)
            .with_graceful_shutdown(shutdown.cancelled_owned())
            .await
    }
}

pub async fn bind(config: ServiceConfig, address: SocketAddr) -> io::Result<BoundService> {
    let listener = TcpListener::bind(address).await?;
    Ok(BoundService {
        listener,
        router: router(config),
    })
}

pub fn router(config: ServiceConfig) -> Router {
    let origins = config
        .allowed_origins
        .iter()
        .filter_map(|origin| origin.parse::<HeaderValue>().ok())
        .collect::<Vec<_>>();

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]);

    let websocket = Router::new()
        .route("/ws", get(ws::upgrade))
        .with_state(WebSocketState::new(
            config.state.clone(),
            config.access.clone(),
        ));

    let api = Router::new()
        .route("/health", get(health))
        .route(
            "/api/v1/recipes",
            get(routes::recipes::list).post(routes::recipes::create),
        )
        .route(
            "/api/v1/books",
            get(routes::books::list).post(routes::books::create),
        )
        .route(
            "/api/v1/books/{id}",
            put(routes::books::update).delete(routes::books::delete),
        )
        .route("/api/v1/recipes/{id}/book", put(routes::books::move_recipe))
        .route(
            "/api/v1/recipes/{id}",
            get(routes::recipes::get)
                .put(routes::recipes::save)
                .delete(routes::recipes::delete),
        )
        .route("/api/v1/validation", post(routes::recipes::validate))
        .route(
            "/api/v1/import/settings",
            get(routes::imports::get_settings).put(routes::imports::update_settings),
        )
        .route("/api/v1/import/translate", post(routes::imports::translate))
        .route(
            "/api/v1/recipes/{id}/export",
            post(routes::exports::export_recipe),
        )
        .route(
            "/api/v1/formulas/calculate",
            post(routes::formulas::calculate),
        )
        .route(
            "/api/v1/formulas/percentages",
            post(routes::formulas::percentages),
        )
        .route("/api/v1/formulas", put(routes::formulas::save))
        .route(
            "/api/v1/recipes/{recipe_id}/formulas",
            get(routes::formulas::list_for_recipe),
        )
        .route("/api/v1/formulas/{formula_id}", get(routes::formulas::get))
        .route(
            "/api/v1/formulas/{formula_id}/runs",
            post(routes::formulas::calculate_and_record),
        )
        .route(
            "/api/v1/recipes/{recipe_id}/haccp",
            get(routes::haccp::list_for_recipe).post(routes::haccp::create),
        )
        .route(
            "/api/v1/haccp/{plan_id}",
            get(routes::haccp::get)
                .put(routes::haccp::save)
                .delete(routes::haccp::delete),
        )
        .route(
            "/api/v1/haccp/ccps/{ccp_id}/records",
            post(routes::haccp::add_monitoring_record),
        )
        .route(
            "/api/v1/recipes/{recipe_id}/tries",
            get(routes::kitchen::list_for_recipe).post(routes::kitchen::start),
        )
        .route(
            "/api/v1/tries/{try_id}",
            get(routes::kitchen::get)
                .put(routes::kitchen::update)
                .delete(routes::kitchen::delete),
        )
        .route(
            "/api/v1/tries/{try_id}/operations/{operation_id}",
            put(routes::kitchen::update_operation),
        )
        .route(
            "/api/v1/tries/{try_id}/observations",
            post(routes::kitchen::add_observation),
        )
        .route("/api/v1/nutrition/status", get(routes::nutrition::status))
        .route("/api/v1/nutrition/search", get(routes::nutrition::search))
        .route(
            "/api/v1/recipes/{recipe_id}/nutrition/links",
            get(routes::nutrition::list_links).post(routes::nutrition::link_resource),
        )
        .route(
            "/api/v1/recipes/{recipe_id}/nutrition/links/{resource_symbol}",
            delete(routes::nutrition::unlink_resource),
        )
        .route(
            "/api/v1/recipes/{recipe_id}/nutrition/calculate",
            post(routes::nutrition::calculate),
        )
        .layer(middleware::from_fn_with_state(
            config.access,
            auth::require_local_client,
        ))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(config.state);

    websocket.merge(api)
}

async fn health() -> Json<models::HealthResponse> {
    Json(models::HealthResponse {
        status: "ok",
        service: "culinograph",
        api_version: "v1",
    })
}
#[cfg(test)]
mod test;

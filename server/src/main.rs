use server::routes;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = routes::get_router();
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| String::from("3000"))
        .parse()
        .unwrap();
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    let server = axum::Server::bind(&addr);
    server.serve(app.into_make_service()).await.unwrap();
}

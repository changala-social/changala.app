use atrg_core::AtrgApp;

#[allow(dead_code)]
mod generated;
mod handlers;
mod routes;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    AtrgApp::new()
        .with_auth_routes(atrg_auth::routes::auth_router())
        .with_cleanup_task(atrg_auth::routes::spawn_cleanup_task)
        .mount(routes::api())
        .on_event(handlers::events::handle_event)
        .run()
        .await
}

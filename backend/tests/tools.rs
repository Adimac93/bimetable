use bimetable::app;
use bimetable::modules::Modules;
use dotenv::dotenv;
use reqwest::Client;
use secrecy::Secret;
use sqlx::PgPool;
use std::net::{SocketAddr, TcpListener};

async fn spawn_app(pool: PgPool) -> SocketAddr {
    // dotenv().ok();

    // let listener = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 0))).unwrap();
    // let addr = listener.local_addr().unwrap();

    // let origin = String::from("http://localhost:3000");

    // let access = Secret::from(String::from("SECRET"));
    // let refresh = Secret::from(String::from("VERY_SECRET"));

    // let modules = Modules::use_custom(pool, addr, origin, access, refresh);

    // tokio::spawn(async move {
    //     axum::Server::from_tcp(listener)
    //         .unwrap()
    //         .serve(
    //             app(modules.state(), modules.extensions())
    //                 .await
    //                 .into_make_service(),
    //         )
    //         .await
    //         .unwrap()
    // });

    // addr
    todo!()
}

pub struct AppData {
    pub addr: SocketAddr,
}

impl AppData {
    pub async fn new(pool: PgPool) -> Self {
        Self {
            addr: spawn_app(pool).await,
        }
    }

    pub fn client(&self) -> Client {
        Client::builder()
            .cookie_store(true)
            .build()
            .expect("Failed to build reqwest client")
    }

    pub fn api(&self, uri: &str) -> String {
        format!("http://{}{uri}", self.addr)
    }
}

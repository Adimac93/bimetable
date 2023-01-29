mod tools;

use sqlx::PgPool;
use tools::AppData;
use tracing::{debug, error, info, trace, warn};
use tracing_test::traced_test;

#[test]
fn sync_test() {
    assert_eq!(2 + 2, 4);
}

#[traced_test] // global logger for `tracing`
#[tokio::test] // tokio runtime test
async fn async_test() {
    let (tx, mut rx) = tokio::sync::mpsc::channel::<&str>(10);

    let task = tokio::task::spawn(async move {
        while let Some(msg) = rx.recv().await {
            debug!("New message!");
            if msg == "close" {
                debug!("Closing task");
                break;
            }
        }
    });

    tx.send("hello").await.unwrap();
    tx.send("close").await.unwrap();
    let res = task.await.unwrap();
    assert_eq!((), res);
}

#[traced_test]
#[sqlx::test] // tokio test included, DATABASE_URL env needed!
async fn database_trace_test(pool: PgPool) {
    let app = AppData::new(pool).await;
    let client = app.client();

    let res = client.get(app.api("/ex/")).send().await.unwrap();
    let html = res.text().await.unwrap();
    trace!("HTML: {}", html);

    assert_eq!(html, String::from("Hello"));
}

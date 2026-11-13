use miniweb_server::{config::Config, server::serve_stream};
use tokio::{net::TcpListener, task::JoinSet};

#[tokio::test]
async fn concurrent_requests_round_trip_on_ephemeral_port() {
    const CLIENTS: usize = 50;
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let config = Config::default();
    let server = tokio::spawn(async move {
        let mut connections = JoinSet::new();
        for _ in 0..CLIENTS {
            let (stream, _) = listener.accept().await.unwrap();
            connections.spawn(serve_stream(stream, config.clone()));
        }
        while let Some(result) = connections.join_next().await {
            result.unwrap().unwrap();
        }
    });
    let client = reqwest::Client::new();
    let mut requests = JoinSet::new();
    for _ in 0..CLIENTS {
        let client = client.clone();
        let url = format!("http://{addr}/health");
        requests.spawn(async move {
            let response = client
                .get(url)
                .header(reqwest::header::CONNECTION, "close")
                .send()
                .await
                .unwrap();
            assert_eq!(response.status(), reqwest::StatusCode::OK);
            let body: serde_json::Value = response.json().await.unwrap();
            assert_eq!(body["server"], "MiniWeb");
            assert_eq!(body["status"], "online");
        });
    }
    while let Some(result) = requests.join_next().await {
        result.unwrap();
    }
    server.await.unwrap();
}

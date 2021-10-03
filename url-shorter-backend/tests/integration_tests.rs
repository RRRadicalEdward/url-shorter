use lib::web::HealthController;
use saphir::server::Server;
use std::time::Duration;
use tokio::time;

fn get_random_address() -> (String, String) {
    let address = "127.0.0.1".to_owned()
        + ":"
        + portpicker::pick_unused_port()
            .expect("Failed to pick unused port")
            .to_string()
            .as_str();

    let target_url = "http://".to_owned() + address.as_str();
    (address, target_url)
}

#[tokio::test]
async fn test_http_connectivity() {
    let (address_to_bind, target_url) = get_random_address();

    let server = Server::builder()
        .configure_listener(|l| {
            l.interface(address_to_bind.as_str())
                .server_name("test_http_connectivity")
        })
        .configure_router(|router| router.controller(HealthController {}))
        .build();

    tokio::spawn(async move { server.run().await.unwrap() });

    // just to make sure Saphir server has started
    time::delay_for(Duration::from_millis(1)).await;

    let get_response = reqwest::get(target_url.as_str()).await;

    assert_eq!(get_response.unwrap().status(), reqwest::StatusCode::OK);
}

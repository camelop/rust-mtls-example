use std::env;

use tokio::fs::File;
use tokio::io::AsyncReadExt;

use reqwest::Identity;
use warp::Filter;

async fn run_server() {
    let routes = warp::any().map(|| "Hello, mTLS World!");

    warp::serve(routes)
        .tls()
        .key_path("ca/localhost.key")
        .cert_path("ca/localhost.bundle.crt")
        .client_auth_required_path("ca/ca.crt")
        .run(([0, 0, 0, 0], 3030))
        .await
}

async fn run_client() -> Result<(), reqwest::Error> {
    let mut buf = Vec::new();
    File::open("ca/ca.crt")
        .await
        .unwrap()
        .read_to_end(&mut buf)
        .await
        .unwrap();
    let cert = reqwest::Certificate::from_pem(&buf)?;

    #[cfg(feature = "native-tls")]
    async fn get_identity() -> Identity {
        let mut buf = Vec::new();
        File::open("ca/client_0.p12")
            .await
            .unwrap()
            .read_to_end(&mut buf)
            .await
            .unwrap();
        reqwest::Identity::from_pkcs12_der(&buf, "123456").unwrap()
    }

    #[cfg(feature = "rustls-tls")]
    async fn get_identity() -> Identity {
        panic!("I don't know why 'Identity' with rustls-tls does not work.");
        let mut buf = Vec::new();
        File::open("ca/client_0.pem")
            .await
            .unwrap()
            .read_to_end(&mut buf)
            .await;
        reqwest::Identity::from_pem(&buf).unwrap()
    }

    let identity = get_identity().await;

    let client = reqwest::Client::builder()
        .tls_built_in_root_certs(false)
        .add_root_certificate(cert)
        .identity(identity)
        .https_only(true)
        .build()?;

    let res = client.get("https://localhost:3030").send().await.unwrap();
    println!("Received:");
    println!("{:?}", res.text().await.unwrap());

    Ok(())
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args[1] == "server" {
        let server = run_server();
        server.await;
    } else if args[1] == "client" {
        let client = run_client();
        client.await.unwrap();
    };
}

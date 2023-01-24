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
        .client_auth_required_path("ca/second_ca.crt")
        .run(([0, 0, 0, 0], 3030))
        .await
}

async fn run_client() -> Result<(), reqwest::Error> {
    let server_ca_file_loc = "ca/ca.crt";
    let mut buf = Vec::new();
    File::open(server_ca_file_loc)
        .await
        .unwrap()
        .read_to_end(&mut buf)
        .await
        .unwrap();
    let cert = reqwest::Certificate::from_pem(&buf)?;

    #[cfg(feature = "native-tls")]
    async fn get_identity() -> Identity {
        let client_p12_file_loc = "ca/second_client.p12";
        let mut buf = Vec::new();
        File::open(client_p12_file_loc)
            .await
            .unwrap()
            .read_to_end(&mut buf)
            .await
            .unwrap();
        reqwest::Identity::from_pkcs12_der(&buf, "123456").unwrap()
    }

    #[cfg(feature = "rustls-tls")]
    async fn get_identity() -> Identity {
        let client_pem_file_loc = "ca/second_client.pem";
        let mut buf = Vec::new();
        File::open(client_pem_file_loc)
            .await
            .unwrap()
            .read_to_end(&mut buf)
            .await
            .unwrap();
        reqwest::Identity::from_pem(&buf).unwrap()
    }

    let identity = get_identity().await;

    #[cfg(feature = "native-tls")]
    let client = reqwest::Client::builder().use_native_tls();

    #[cfg(feature = "rustls-tls")]
    let client = reqwest::Client::builder().use_rustls_tls();

    let client = client
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

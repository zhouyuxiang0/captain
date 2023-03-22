use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufReader},
};

use actix_files::Files;
use actix_web::{
    self, error, get, middleware, post, web, App, Error, HttpResponse, HttpServer, Responder,
};
use actix_web_lab::respond::Html;
use rust_embed::RustEmbed;
use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile::{certs, pkcs8_private_keys};
use tera::Tera;

#[derive(RustEmbed)]
#[folder = "public/"]
struct Public;

#[actix_web::main]
async fn main() -> io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    tracing_subscriber::fmt::init();
    let config = load_rustls_config();
    // tera
    HttpServer::new(|| {
        App::new()
            // .service(Files::new)
            .wrap(middleware::Logger::default())
            .service(hello)
    })
    .bind_rustls("127.0.0.1:8000", config)?
    .run()
    .await
}

#[get("/{_:.*}")]
async fn hello(path: web::Path<String>) -> Result<impl Responder, Error> {
    Ok(Html(handle_embedded_file(path.as_str())))
}

fn load_rustls_config() -> rustls::ServerConfig {
    // init server config builder with safe defaults
    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth();

    // load TLS key/cert files
    let cert_file = &mut BufReader::new(File::open("cert.pem").unwrap());
    let key_file = &mut BufReader::new(File::open("key.pem").unwrap());

    // convert files to key/cert objects
    let cert_chain = certs(cert_file)
        .unwrap()
        .into_iter()
        .map(Certificate)
        .collect();
    let mut keys: Vec<PrivateKey> = pkcs8_private_keys(key_file)
        .unwrap()
        .into_iter()
        .map(PrivateKey)
        .collect();

    // exit if no keys could be parsed
    if keys.is_empty() {
        eprintln!("Could not locate PKCS 8 private keys.");
        std::process::exit(1);
    }

    config.with_single_cert(cert_chain, keys.remove(0)).unwrap()
}

fn handle_embedded_file(path: &str) -> String {
    let html = Public::get(path).unwrap();
    let b = std::str::from_utf8(&html.data.as_ref()).unwrap();
    String::from(b)
    // match Templates::get(path) {
    //     Some(content) => HttpResponse::Ok()
    //         .content_type(from_path(path).first_or_octet_stream().as_ref())
    //         .body(content.data.into_owned()),
    //     None => HttpResponse::NotFound().body("404 Not Found"),
    // }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{http::header::ContentType, test};

    #[actix_web::test]
    async fn test_index_ok() {
        let app = test::init_service(App::new().service(hello)).await;
        let req = test::TestRequest::default()
            .insert_header(ContentType::plaintext())
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success())
    }
}

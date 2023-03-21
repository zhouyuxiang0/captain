use std::{collections::HashMap, io};

use actix_files::Files;
use actix_web::{
    self, error, get, middleware, post, web, App, Error, HttpResponse, HttpServer, Responder,
};
use actix_web_lab::respond::Html;
use rust_embed::RustEmbed;
use tera::Tera;

#[derive(RustEmbed)]
#[folder = "templates/"]
struct Templates;

#[actix_web::main]
async fn main() -> io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    tracing_subscriber::fmt::init();
    // tera
    HttpServer::new(|| {
        let mut tera = Tera::new("./templates/*").unwrap();
        tera.add_raw_template("index.html", handle_embedded_file("index.html").as_str())
            .unwrap();
        App::new()
            // .service(Files::new)
            .app_data(web::Data::new(tera))
            .wrap(middleware::Logger::default())
            .service(hello)
    })
    .bind(("127.0.0.1", 8000))?
    .run()
    .await
}

#[get("/")]
async fn hello(
    tmpl: web::Data<tera::Tera>,
    query: web::Query<HashMap<String, String>>,
) -> Result<impl Responder, Error> {
    let mut context = tera::Context::default();
    context.insert("test", "val");
    let s = tmpl
        .render("index.html", &context)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(Html(s))
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

fn handle_embedded_file(path: &str) -> String {
    let html = Templates::get(path).unwrap();
    let b = std::str::from_utf8(html.data.as_ref()).unwrap();
    String::from(b)
    // match Templates::get(path) {
    //     Some(content) => HttpResponse::Ok()
    //         .content_type(from_path(path).first_or_octet_stream().as_ref())
    //         .body(content.data.into_owned()),
    //     None => HttpResponse::NotFound().body("404 Not Found"),
    // }
}

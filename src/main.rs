use std::{collections::HashMap, io};

use actix_files::Files;
use actix_web::{
    self, error, get, middleware, post, web, App, Error, HttpResponse, HttpServer, Responder,
};
use actix_web_lab::respond::Html;
use include_dir::{include_dir, Dir};
use tera::Tera;

static TEMPLATES_DIR: Dir = include_dir!("templates");

#[actix_web::main]
async fn main() -> io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    tracing_subscriber::fmt::init();
    // tera
    HttpServer::new(|| {
        let mut tera = Tera::new("./*").unwrap();
        tera.add_raw_templates(vec![(
            "index.html",
            TEMPLATES_DIR
                .get_file("index.html")
                .unwrap()
                .contents_utf8()
                .unwrap(),
        )]);
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
    let mut context = tera::Context::new();
    context.insert("test", &"测试");
    let string = tmpl
        .render("index.html", &context)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    // let h = tmpl.render_str("{}", &mut tera::Context::new());
    Ok(Html(string))
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

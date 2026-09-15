//! Web interface: a single page embedded in the binary (no file to install, no external
//! resource), that works on top of the HTTP API. Enabled and placed by `[web_ui]`.

use actix_web::http::header;
use actix_web::{HttpResponse, web};

const INDEX_HTML: &str = include_str!("../../web/index.html");
const APP_CSS: &str = include_str!("../../web/app.css");
const APP_JS: &str = include_str!("../../web/app.js");

/// Scripts and styles come from the page itself only
const CONTENT_SECURITY_POLICY: &str = "default-src 'self'; script-src 'self'; \
     style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self'; \
     frame-ancestors 'none'; base-uri 'none'; form-action 'none'";

fn asset(content_type: &'static str, body: &'static str) -> HttpResponse {
    HttpResponse::Ok()
        .insert_header((header::CONTENT_TYPE, content_type))
        // Always revalidated: the files change with the binary
        .insert_header((header::CACHE_CONTROL, "no-cache"))
        .insert_header((header::X_CONTENT_TYPE_OPTIONS, "nosniff"))
        .insert_header((header::CONTENT_SECURITY_POLICY, CONTENT_SECURITY_POLICY))
        .insert_header((header::REFERRER_POLICY, "no-referrer"))
        .body(body)
}

async fn index() -> HttpResponse {
    asset("text/html; charset=utf-8", INDEX_HTML)
}

async fn css() -> HttpResponse {
    asset("text/css; charset=utf-8", APP_CSS)
}

async fn js() -> HttpResponse {
    asset("text/javascript; charset=utf-8", APP_JS)
}

/// Routes of the web interface
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/", web::get().to(index))
        .route("/ui/app.css", web::get().to(css))
        .route("/ui/app.js", web::get().to(js));
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::App;
    use actix_web::http::StatusCode;
    use actix_web::test as atest;

    #[actix_web::test]
    async fn pages_are_served_with_their_type() {
        let app = atest::init_service(App::new().configure(configure)).await;
        for (path, content_type, marker) in [
            ("/", "text/html", "ui/app.js"),
            ("/ui/app.css", "text/css", ":root"),
            ("/ui/app.js", "text/javascript", "api/status"),
        ] {
            let response =
                atest::call_service(&app, atest::TestRequest::get().uri(path).to_request()).await;
            assert_eq!(response.status(), StatusCode::OK, "{}", path);
            let headers = response.headers();
            assert!(
                headers
                    .get(header::CONTENT_TYPE)
                    .and_then(|v| v.to_str().ok())
                    .is_some_and(|v| v.starts_with(content_type)),
                "{}",
                path
            );
            assert!(headers.contains_key(header::CONTENT_SECURITY_POLICY));
            let body = atest::read_body(response).await;
            assert!(
                std::str::from_utf8(&body).unwrap().contains(marker),
                "{}",
                path
            );
        }
    }
}

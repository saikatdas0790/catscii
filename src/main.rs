use std::str::FromStr;

use artem::options::TargetType;
use axum::{
    Router,
    body::BoxBody,
    response::{IntoResponse, Response},
    routing::get,
};
use image::load_from_memory;
use reqwest::{StatusCode, header};
use tracing::{Level, info};
use tracing_subscriber::{filter::Targets, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    let filter = Targets::from_str(std::env::var("RUST_LOG").as_deref().unwrap_or("info"))
        .expect("RUST_LOG should be a valid tracing filter");
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .json()
        .finish()
        .with(filter)
        .init();

    let app = Router::new().route("/", get(root_get));

    let addr = "0.0.0.0:8080".parse().unwrap();
    info!("Listening on http://{addr}");

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root_get() -> Response<BoxBody> {
    match get_cat_ascii_art().await {
        Ok(art) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
            art,
        )
            .into_response(),
        Err(e) => {
            println!("Something went wrong: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong").into_response()
        }
    }
}

async fn get_cat_ascii_art() -> color_eyre::Result<String> {
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct CatImage {
        url: String,
    }
    let api_url = "https://api.thecatapi.com/v1/images/search";
    let client = reqwest::Client::default();

    let image = client
        .get(api_url)
        .send()
        .await?
        .error_for_status()?
        .json::<Vec<CatImage>>()
        .await?
        .pop()
        .ok_or_else(|| color_eyre::eyre::eyre!("The cat API returned no images"))?;

    let image_bytes = client
        .get(image.url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?
        .to_vec();

    let image = load_from_memory(&image_bytes)?;
    let ascii_art = artem::convert(
        image,
        artem::options::OptionBuilder::new()
            .target(TargetType::HtmlFile(true, true))
            .build(),
    );

    Ok(ascii_art)
}

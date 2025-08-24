use axum::{
    extract::Path,
    http::{Response, StatusCode},
    routing::get,
    Router,
};
use tokio;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Player {
    id: u64,
    name: String,
    ratings: Vec<Rating>,
    platform: String,
    top_global: u32,
    tags: Vec<Tag>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Tag {
    tag: String,
    style: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Rating {
    rating: f64,
    char_short: String,
    character: String,
    match_count: u32,
    top_char: u32,
    top_defeated: TopDefeated,
    top_rating: TopRating,
}

#[derive(Serialize, Deserialize, Debug)]
struct TopDefeated {
    timestamp: String,
    id: u64,
    name: String,
    char_short: String,
    value: f64,
}

#[derive(Serialize, Deserialize, Debug)]
struct TopRating {
    timestamp: String,
    value: f64,
}

async fn player(
    Path((player_id, char_id)): Path<(i64, String)>,
) -> Result<Response<String>, (StatusCode, String)> {
    let url = format!("https://puddle.farm/api/player/{}", player_id);
    let response = reqwest::get(&url).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("E1 {}", e.to_string()),
        )
    })?;
    let body = response.text().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("E2 {}", e.to_string()),
        )
    })?;

    let player: Player = serde_json::from_str(&body).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("E3 {} : {}", e.to_string(), body),
        )
    })?;

    let rating = player
        .ratings
        .iter()
        .find(|r| r.char_short == char_id)
        .ok_or((StatusCode::NOT_FOUND, "Character not found".to_string()))?;

    let rating_str = format!("{:.1}", rating.rating);

    let html = format!(
        r#"<!DOCTYPE html>
    <html>
    <head>
        <meta property="og:title" content="{} - {}" />
        <meta property="og:type" content="website" />
        <meta property="og:description" content="Rating: {} | Games: {}" />
        <meta property="og:site_name" content="puddle.farm" />
        <meta property="og:url" content="https://puddle.farm/player/{}/{}" />
        <meta property="og:image" content="https://puddle.farm/api/avatar/{}" />
    </head>
    <body>
        <p>Player stats for {}</p>
    </body>
    </html>"#,
        player.name,
        rating.character,
        rating_str,
        rating.match_count,
        player_id,
        rating.char_short,
        player_id,
        player.name,
    );

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html; charset=utf-8")
        .body(html)
        .unwrap())
}

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new().route("/player/:player_id/:char_id", get(player));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8002").await?;
    axum::serve(listener, app).await?;
    Ok(())
}

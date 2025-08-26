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
    rating: i64,
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

#[derive(Debug)]
struct RankInfo {
    name: String,
}

fn get_rank_from_rating(rating: i64) -> RankInfo {
    let rank_thresholds = [
        (45000, "Vanquisher"),
        (40800, "Diamond 3"),
        (36000, "Diamond 2"),
        (32400, "Diamond 1"),
        (28400, "Platinum 3"),
        (24400, "Platinum 2"),
        (20400, "Platinum 1"),
        (18000, "Gold 3"),
        (15600, "Gold 2"),
        (13200, "Gold 1"),
        (11000, "Silver 3"),
        (8800, "Silver 2"),
        (6600, "Silver 1"),
        (5400, "Bronze 3"),
        (4200, "Bronze 2"),
        (3000, "Bronze 1"),
        (2000, "Iron 3"),
        (1000, "Iron 2"),
        (1, "Iron 1"),
        (0, "Placement"),
    ];

    for (threshold, name) in rank_thresholds.iter() {
        if rating >= *threshold {
            return RankInfo {
                name: name.to_string(),
            };
        }
    }

    RankInfo {
        name: "Placement".to_string(),
    }
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

    let rank_info = get_rank_from_rating(rating.rating);
    let rating_str = if rank_info.name == "Vanquisher" {
        format!("{} DR", rating.rating - 10000000)
    } else {
        format!("{} RP", rating.rating)
    };

    let html = format!(
        r#"<!DOCTYPE html>
    <html>
    <head>
        <meta property="og:title" content="{} - {}" />
        <meta property="og:type" content="website" />
        <meta property="og:description" content="{}, {} | Games: {}" />
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
        rank_info.name,
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

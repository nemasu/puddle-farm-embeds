use axum::{extract::Path, http::StatusCode, routing::get, Json, Router};
use tokio;

use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize, Debug)]
struct Player {
    id: u64,
    name: String,
    ratings: Vec<Rating>,
    platform: String,
    status: String,
    top_global: u32,
    tags: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Rating {
    rating: f64,
    deviation: f64,
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
    deviation: f64,
}

#[derive(Serialize, Deserialize, Debug)]
struct TopRating {
    timestamp: String,
    value: f64,
    deviation: f64,
}

async fn player(
    Path((player_id, char_id)): Path<(i64, String)>,
) -> Result<Json<String>, (StatusCode, String)> {
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
    let deviation_str = format!("{:.1}", rating.deviation);

    let embed_json = json!({
        "title": player.name,
        "description": "Ratings for Guilty Gear Strive",
        "color": 0x9f2b68,
        "fields": [
            {
                "name": rating.character.clone(),
                "value": format!("{} ±{}", rating_str, deviation_str),
                "inline": true
            },
            {
                "name": "Games",
                "value": rating.match_count.to_string(),
                "inline": true
            }
        ],
        "footer": {
            "text": "puddle.farm is open source"
        }
    });

    let embed_json = json!({ "embeds": [embed_json] }); // Discord expects an array of embeds
    let embed_json = serde_json::to_string(&embed_json).unwrap();
    Ok(Json(embed_json))
}

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new().route("/player/:player_id/:char_id", get(player));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8002").await?;
    axum::serve(listener, app).await?;
    Ok(())
}

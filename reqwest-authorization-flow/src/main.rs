use rand::Rng;
use std::iter;
use axum::{response::IntoResponse, routing::get, Json, Router};

// produce random string of characters:
// https://stackoverflow.com/questions/54275459/how-do-i-create-a-random-string-by-sampling-from-alphanumeric-characters
fn random_string_generation(len: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut rng = rand::thread_rng();
    let one_char = || CHARSET[rng.random_range(0..CHARSET.len())] as char;
    iter::repeat_with(one_char).take(len).collect()
}

async fn status_handler() -> impl IntoResponse{
    let message_status: &str = "greetings earthlings app running: 👽";

    let json_response = serde_json::json!(
        {
            "status": "ok", 
            "message": message_status
        }
    );
    println!("Server Status: OK");

    Json(json_response)
}

#[tokio::main]
async fn main() {
    // Documentation: https://developer.spotify.com/documentation/web-api/tutorials/code-flow
    println!("Authorization Code Flow:");


    // get the variables needed to make the authorization request
    let client_credentials = dotenvy::var("SPOTIFY_CLIENT_ID").expect("SPOTIFY_CLIENT_ID must be set");
    let client_secret = dotenvy::var("SPOTIFY_CLIENT_SECRET").expect("SPOTIFY_CLIENT_SECRET must be set");
    let sp_callback = dotenvy::var("SPOTIFY_REDIRECT_URI").expect("REDIRECT URI must be set");

    // produce a random string from the rand crate
    let state: String = random_string_generation(16);

    println!("🛸 Server started successfully");

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8000").await.unwrap();

    // create the app
    let api_routes = Router::new()
        .route("/status", get(status_handler));

    // return the router
    let app = Router::new().nest("/api", api_routes);

    axum::serve(listener, app).await.unwrap();

}

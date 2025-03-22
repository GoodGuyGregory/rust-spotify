use rand::Rng;
use std::iter;
use axum::{response::{IntoResponse, Redirect}, routing::get, Json, Router};

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

async fn get_spotify_login() -> impl IntoResponse {

    println!("Server: Attempting Spotify Login 🔊");

     // get the variables needed to make the authorization request
    let client_credentials = dotenvy::var("SPOTIFY_CLIENT_ID").expect("SPOTIFY_CLIENT_ID must be set");
    let client_secret = dotenvy::var("SPOTIFY_CLIENT_SECRET").expect("SPOTIFY_CLIENT_SECRET must be set");
    let sp_callback = dotenvy::var("SPOTIFY_REDIRECT_URI").expect("REDIRECT URI must be set");
    
     // produce a random string from the rand crate
    let state: String = random_string_generation(16);
    
    let query_params = vec![("response_type", "code"), 
                            ("client_id", &client_credentials), 
                            ("scope", "user-top-read"),
                            ("redirect_uri", &sp_callback),
                            ("state", &state)];

    
    // build the redirection string needed for spotify request
    let mut authorize_route = String::from("https://accounts.spotify.com/authorize?");
    authorize_route.push_str(&querystring::stringify(query_params));

    Redirect::to(&authorize_route)

}

#[tokio::main]
async fn main() {
    // Documentation: https://developer.spotify.com/documentation/web-api/tutorials/code-flow

    // Code flow is all for this method: https://developer.spotify.com/documentation/web-api/reference/get-users-top-artists-and-tracks

    println!("Authorization Code Flow:");


    println!("🛸 Server started successfully");

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8000").await.unwrap();

    // create the app
    let api_routes = Router::new()
        .route("/status", get(status_handler))
        .route("/spotify-auth", get(get_spotify_login));

    // return the router
    let app = Router::new().nest("/api", api_routes);

    axum::serve(listener, app).await.unwrap();

}

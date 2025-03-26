use rand::Rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::sync::Arc;
use sqlx::{postgres::PgPoolOptions, PgPool, Pool, Postgres};
use std::iter;
use std::collections::HashMap;
use std::error::Error;
use serde_json::json;
use reqwest::{self, header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE}, Client, Response, StatusCode};
use axum::{response::{IntoResponse, Redirect}, extract::Query, routing::get, Json, Router};
use tower_cookies::{cookie, Cookie, CookieManagerLayer, Cookies};




// used to establish the Callback Response Attributes 
// from the SPOTIFY-API
#[derive(Deserialize, Serialize)]
struct CallbackAuth {
    // happy path
    code: Option<String>,
    // error in case of trouble
    error: Option<String>,
    state: Option<String>
}


pub struct AppState {
    db: Pool<Postgres>,
}

#[derive(Serialize, Deserialize)]
struct UserSessionAuth {
    user_id: Uuid,
    state: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AuthResponse {
    access_token: String,
    token_type: String,
    scope: String,
    // expiration of the token as integer
    expires_in: i32,
    // https://developer.spotify.com/documentation/web-api/tutorials/refreshing-tokens
    refresh_token: String,

}

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

// database connection for session details:
async fn connect_to_database() -> Result<PgPool, Box<dyn Error>> {

    let database_url = dotenvy::var("DATABASE_URL").expect("ERROR: DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    println!("✅ Connection to the database is successful!");
    Ok(pool)
}


async fn get_spotify_login() -> impl IntoResponse {

    println!("Server: Attempting Spotify Login 🔊");

     // produce a random string from the rand crate
    let state: String = random_string_generation(16);

    let  client_credentials: String = dotenvy::var("SPOTIFY_CLIENT_ID").expect("SPOTIFY_CLIENT_ID must be set");
    let sp_callback: String = dotenvy::var("SPOTIFY_REDIRECT_URI").expect("REDIRECT URI must be set");

    
    let query_params = vec![("response_type", "code"),
                            ("client_id", &client_credentials),
                            ("scope", "user-top-read"),
                            ("redirect_uri", &sp_callback),
                            ("state", &state)];

    // build the redirection string needed for spotify request
    let mut authorize_route = String::from("https://accounts.spotify.com/authorize?");
    // use stringify to produce the values for the url
    authorize_route.push_str(&querystring::stringify(query_params));

    Redirect::to(&authorize_route)

}

// callback endpoint for our application
// https://docs.rs/axum/latest/axum/extract/struct.Query.html
async fn spotify_callback_handler(Query(callback_auth): Query<CallbackAuth>, cookie: Cookies) -> Result<impl IntoResponse, (StatusCode, String)> {
    
    println!("Server: Determining Spotify Callback Authentication");
    match callback_auth.code {
        None => {
            println!("Server: Spotify Authentication Failure 🚳");
            let json_response = json!({
                "status": "error",
                "message": callback_auth.error.unwrap_or_else(|| "authentication failed for unspecified reason".to_string())
            });
            println!("Authentication Failed");
            Ok((StatusCode::UNAUTHORIZED, Json(json_response)))
        }
        Some(exchange_code) => {
            let client_id = dotenvy::var("SPOTIFY_CLIENT_ID").expect("SPOTIFY_CLIENT_ID is not set");
            let client_secret = dotenvy::var("SPOTIFY_CLIENT_SECRET").expect("SPOTIFY_CLIENT_SECRET is not set");
            let sp_callback = dotenvy::var("SPOTIFY_REDIRECT_URI").expect("SPOTIFY_REDIRECT_URI is not set");
            let authorization_type = String::from("authorization_code");

            let mut params_body = HashMap::new();
            // add the body required
            params_body.insert("code", exchange_code.clone());
            params_body.insert("redirect_uri", sp_callback.clone());
            params_body.insert("grant_type", "authorization_code".to_string());

            let client = reqwest::Client::new();

            let encoded_client_id_secret = base64::encode(format!("{}:{}",client_id,client_secret));
            let basic_auth_encoded = format!("Basic {}", encoded_client_id_secret);

            let spotify_response = client
                .post("https://accounts.spotify.com/api/token")
                .header("Authorization", &basic_auth_encoded)
                .header("Content-Type", "application/x-www-form-urlencoded")
                .header("Accept", "application/json")
                .form(&params_body)
                .send()
                .await;

            match spotify_response {
                Ok(resp) => {
                    let status = resp.status();

                    if status.is_success() {
                        let raw_text = resp.text().await.unwrap_or_else(|_| "Failed to read response".to_string());
                        println!("Spotify API raw response: {}", raw_text);

                        let auth_response: AuthResponse = serde_json::from_str(&raw_text).map_err(|e| {
                            println!("Spotify API JSON parse error: {}", e);
                            (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to parse JSON: {}", e))
                        })?;

                        println!("Server Status: Success Setting Access_Token Attributes!");
                        Ok((StatusCode::OK, Json(json!({
                            "status": "success",
                            "message": auth_response
                        }))))
                    } else {
                        let error_text = resp.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                        println!("Spotify API error: {}", error_text);
                        Ok((StatusCode::UNAUTHORIZED, Json(json!({
                            "status": "error",
                            "message": error_text
                        }))))
                    }
                },
                Err(err) => {
                    println!("Spotify API request error: {}", err);
                    Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
                }
            }

        }
    }
}

#[tokio::main]
async fn main() {

    // Documentation: https://developer.spotify.com/documentation/web-api/tutorials/code-flow

    // Code flow is all for this method: https://developer.spotify.com/documentation/web-api/reference/get-users-top-artists-and-tracks

    println!("Authorization Code Flow:");

    match connect_to_database().await {
        Ok(pool) => {
            let app_state = Arc::new(AppState {db: pool.clone() });

            println!("🛸 Server started successfully");

            let listener = tokio::net::TcpListener::bind("127.0.0.1:8000").await.unwrap();

            // create the app
            let api_routes = Router::new()
                .route("/status", get(status_handler))
                .route("/spotify-auth", get(get_spotify_login))
                .route("/callback", get(spotify_callback_handler))
                .layer(CookieManagerLayer::new());

            // return the router
            let app = Router::new().nest("/api", api_routes);

            axum::serve(listener, app).await.unwrap();      
        }   
        Err(err) => {
            println!("🔥 Failed to connect to the database: {:?}", err);
        }

    }

    

}

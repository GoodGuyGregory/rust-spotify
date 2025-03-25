use rand::Rng;
use serde::Deserialize;
use std::iter;
use axum::{response::{IntoResponse, Redirect}, extract::Query, routing::get, Json, Router};


   // get the variables needed to make the authorization request
let client_credentials = dotenvy::var("SPOTIFY_CLIENT_ID").expect("SPOTIFY_CLIENT_ID must be set");
let client_secret = dotenvy::var("SPOTIFY_CLIENT_SECRET").expect("SPOTIFY_CLIENT_SECRET must be set");
let sp_callback = dotenvy::var("SPOTIFY_REDIRECT_URI").expect("REDIRECT URI must be set");



// used to establish the Callback Response Attributes 
// from the SPOTIFY-API
#[derive(Deserialize)]
struct Callback_Auth {
    // happy path
    code: Option<String>,
    // error in case of trouble
    error: Option<String>,
    state: Option<String>
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

async fn get_spotify_login() -> impl IntoResponse {

    println!("Server: Attempting Spotify Login 🔊");

    
     // produce a random string from the rand crate
    let state: String = random_string_generation(16);
    
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
async fn spotify_callback_handler(callback_auth: Query<Callback_Auth>) -> impl IntoResponse {

    match callback_auth.code {
        // need to return error for user verification:
        None =>  {
            match callback_auth.error {
                Some(error_reason ) => {
                    let json_response = serde_json::json!(
                        {
                            "status": "error", 
                            "message": error_reason
                        }
                    );
                    println!("Authentication Failed");
                
                    Json(json_response)
                },
                None => {
                    let json_response = serde_json::json!(
                        {
                            "status": "error", 
                            "message": "authentication failed for unspecified reason"
                        }
                    );
                    println!("Authentication Failed");
                
                    Json(json_response)
                }
            }
            
        },
        Some(exchange_code) =>  {
            // use the reqwest to make the authentication request:
            // https://docs.rs/reqwest/0.11.5/reqwest/#making-post-requests-or-setting-request-bodies
            // get the client credentials

            
            let mut params_body = HashMap::new();
            // add the body required
            params_body.insert("code", exchange_code);
            params_body.insert("redirect_uri", &sp_callback);
            params_body.insert("grant_type","authorization_code" );

            // create the base64 encoded string
            let encoded_client_id_secret = encode(format!("{}:{}",&client_credentials, &client_secret));
            let basic_auth_encoded = format!("{} {}", "Basic ", encoded_client_id_secret);

            let bearer_token = client.post("https://accounts.spotify.com/api/token")
                .header(AUTHORIZATION, &basic_auth_encoded )
                .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
                .header(ACCEPT, "application/json")
                .form(&params_body)
                .send()
                .await
                .unwrap()
                .text()
                .await;

        
        }

    }

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
        .route("/spotify-auth", get(get_spotify_login))
        .route("/callback", get(spotify_callback_handler));

    // return the router
    let app = Router::new().nest("/api", api_routes);

    axum::serve(listener, app).await.unwrap();

}

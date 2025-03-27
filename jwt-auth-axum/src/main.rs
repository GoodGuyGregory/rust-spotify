use axum::{Router, routing::{post, get}, http::StatusCode, response::IntoResponse, Json};
use serde_json::json;
use serde::{Serialize, Deserialize};


#[derive(Debug)]
enum AuthError {
    WrongCredentials,
    MissingCredentials,
    TokenCreation,
    InvalidToken
}

#[derive(Debug, Serialize, Deserialize)]
struct PrivateMessage {
    auth_token: String,
    private_message: String,
}


// implement a struct for AuthError with the IntoResponse trait overridden
impl IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            AuthError::WrongCredentials => (StatusCode::UNAUTHORIZED, "Wrong credentials"),
            AuthError::MissingCredentials => (StatusCode::BAD_REQUEST, "Missing credentials"),
            AuthError::TokenCreation => (StatusCode::INTERNAL_SERVER_ERROR, "Token creation error"),
            AuthError::InvalidToken => (StatusCode::BAD_REQUEST, "Invalid token"),
        };

        let body = Json(json!({
            "error": message,
        }));
        // call the response to return the StatusCode and the message
        (status, body).into_response()
    }
}

async fn public_handler() -> impl IntoResponse {
    println!("Server GET: public ");

    let json_response = json!({
        "status": "ok",
        "message": "Welcome to the Public Side of the Application, here you wouldn't require authentication"
    });
    Json(json_response)
}

async fn login_user() {
    println!("Server POST: login user");
}

async fn private_message() -> Result<Json<serde_json::Value>, AuthError> {
    println!("Server GET: private message ");
    // demonstrate why authentication is important:

    let private_message = "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢠⠆⠀⠀⠀⠀⠀⠄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
                                ⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⠴⠋⠀⠀⠀⠀⠀⠀⢠⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
                                ⠀⠀⠀⠀⠀⠀⠀⠀⢠⠐⠉⠀⠀⠀⠀⠀⠀⠀⠀⠀⢰⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
                                ⠀⠀⠀⠀⠀⠀⠀⢰⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⠇⠀⠀⠀⠀⠀⠀⠀⠀⠀⡆⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
                                ⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡏⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
                                ⠀⠀⠀⠀⠀⠀⠘⡄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢠⠇⠀⠀⠀⠀⠀⡼⠇⠀⠀⠀⠘⡆⠀⠀⠐⠀⠀⠀⢀⠀⠀⠀
                                ⠀⠀⠀⠀⠀⠀⠀⠈⠓⠢⢼⠀⠀⠀⠀⠀⠀⠀⠀⣾⠀⠀⠀⠀⣠⠎⠀⠀⠀⠀⠀⠀⠸⡄⠀⠀⠀⠀⠀⠈⣇⠀⠀
                                ⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⠀⠀⠀⠀⠀⠀⠀⢠⠇⠀⠀⠀⡰⠃⠀⠀⠀⠀⠀⠀⢀⡼⡇⠀⠀⠀⠀⠀⠀⢸⠀⠀
                                ⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⣸⠀⠀⠀⡼⠁⠀⠀⠀⠀⠀⠀⢠⠞⠀⢹⡄⠀⠀⠀⠀⠀⠘⡇⠀
                                ⠀⠀⠀⠀⠀⠀⠀⠀⠀⣾⠀⠀⠀⠀⠀⠀⠀⢀⡇⠀⠀⣰⠁⠀⠀⠀⠀⠀⠀⣰⠋⠀⠀⠘⣧⠀⠀⠀⠀⠀⠀⢹⠀
                                ⠀⠀⠀⠀⠀⠀⠀⠀⣼⠋⠀⠀⠀⠀⠀⠀⠀⡟⠀⠀⠀⣿⠀⠀⠀⠀⠀⠀⡴⠃⠀⠀⠀⠀⣿⠀⠀⠀⠀⠀⠀⢸⡇
                                ⠀⠀⠀⠀⠀⠀⠀⢰⠋⠀⠀⠀⠀⠀⠀⢀⡾⠀⠀⠀⠀⠛⠀⠀⠀⠀⠀⢸⡁⠀⠀⠀⠀⠀⣿⠀⠀⠀⠀⠀⠀⢸⡇
                                ⠀⠀⠀⠀⠀⠀⢠⡏⠀⠀⠀⠀⠀⠀⢀⡞⠁⠀⡿⣯⡷⡴⢦⣤⡠⣶⡶⠀⢷⠀⠀⠀⠀⢰⡇⠀⠀⠀⠀⠀⠀⡾⠀
                                ⠀⠀⠀⠀⠀⠀⡞⠀⠀⠀⠀⠀⠀⠀⣼⣥⣤⣤⣤⣤⣤⣤⣤⣀⣀⣀⣀⠀⠈⢧⠀⠀⠀⢸⡇⠀⠀⠀⠀⠀⢀⡇⠀
                                ⠀⠀⠀⠀⠀⢸⠁⠀⠀⠀⠀⠀⠀⡼⠁⠀⠀⠀⠀⠉⠙⠻⢿⣿⣿⣿⣿⣿⣿⠛⢦⠀⠀⢸⡇⠀⠀⠀⠀⠀⢸⡇⠀
                                ⠀⠀⠀⠀⢠⡏⠀⠀⠀⠀⠀⠀⡼⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠙⢿⣿⣿⣿⣿⠳⠀⢳⡀⢹⡇⠀⠀⠀⠀⠀⡾⡇⠀
                                ⠀⠀⠀⠀⡞⠀⠀⠀⠀⠀⠀⡼⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢻⣿⣿⡿⠘⠀⠀⠹⣼⡇⠀⠀⠀⠀⢠⠇⠀⠀
                                ⠀⠀⠀⣰⠃⠀⠀⠀⠀⠀⡾⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⣿⡿⠁⠀⠀⠀⠀⠘⣇⠀⠀⠀⠀⡾⠀⠀⠀
                                ⠀⠀⢠⡏⠀⠀⠀⠀⠀⡼⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢿⠁⠀⠀⠀⠀⠀⠀⠸⡄⠀⠀⢸⠁⠀⠀⠀
                                ⠀⠀⡾⠀⠀⠀⠀⠀⡾⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣼⠀⠀⠀⠀⠀⠀⠀⠀⢻⠀⠀⡟⠀⠀⠀⠀
                                ⠀⣴⠓⣾⣳⣀⢀⡼⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢻⡇⠃⠀⠀⠀⠀⠀⠀⢸⡇⢀⠇⠀⠀⠀⠀
                                ⣾⠃⠀⠀⠀⠑⡟⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣾⠃⠀⠀⠀⠀⠀⠀⠀⠈⡇⢸⠀⠀⠀⠀⠀
                                ⠹⡀⠀⠀⠀⠀⠹⣆⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⡏⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⣾⠀⠀⠀⠀⠀
                                ⠀⢳⡄⠀⠀⠀⠀⠘⣄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡼⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⡇⣿⠀⠀⠀⠀⠀
                                ⠀⠀⣷⡄⠀⠀⠀⠀⠙⢦⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⡞⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⠃⡏⠀⠀⠀⠀⠀
                                ⠀⢀⡇⢹⣄⠀⠀⠀⠀⣀⠉⠓⠶⢄⡀⠀⠀⠀⠀⠀⢀⣠⠴⠋⠣⣄⠀⠀⠀⠀⠀⠀⠀⠀⢠⠟⣸⣧⠀⠀⠀⠀⠀
                                ⠀⣴⣿⠋⠘⣆⠀⢰⠶⠤⢍⣛⣶⠤⠿⣷⣦⡀⠒⠚⡟⠀⠀⠀⠀⠈⠛⠢⠤⡄⠀⠀⢀⡴⢯⠴⣳⠇⠀⠀⠀⠀⠀
                                ⠀⠀⠉⠀⠀⠘⢦⡈⠻⣖⠤⣤⣉⣉⣹⣯⣭⠉⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢠⣾⠛⣫⣼⠃⠀⠀⠀⠀⠀⠀
                                ⠀⠀⠀⠀⠀⠀⠀⠑⣄⠉⢦⡀⠀⠀⠈⠉⠁⠀⠀⣸⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⣴⢿⣷⢚⡝⠁⠀⠀⠀⠀⠀⠀⠀
                                ⠀⠀⠀⠀⠀⠀⠀⠀⠀⠹⢶⣷⠇⠀⠀⠀⠀⠀⣠⠏⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣴⣿⠷⠉⠀⠀⠀⠀⠀⠀⠀⠀⠀
                                ⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠸⠋⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠀⠀⠀⠀⠀⠀⠀⠀⠀";

    let private_response = PrivateMessage {
        private_message: private_message.to_string(),
        auth_token: "".to_string(),
    };

    Ok(Json(serde_json::json!(
        private_response
    )))


}

//  JWT Authentication with AXUM
// https://docs.shuttle.dev/examples/axum-jwt-authentication
#[tokio::main]
async fn main() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:8000").await.unwrap();

        // create the app
        let api_routes = Router::new()
            .route("/public", get(public_handler))
            .route("/login", post(login_user))
            .route("/private", get(private_message));

        // return the router
        let app = Router::new().nest("/api", api_routes);

        println!("🛸 Server started successfully");
        axum::serve(listener, app).await.unwrap();

}

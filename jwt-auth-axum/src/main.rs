use std::time::SystemTime;
use axum::{Router, routing::{post, get}, http::{request::Parts, StatusCode},
           extract::FromRequestParts,  response::{IntoResponse}, RequestPartsExt, Json};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use serde_json::json;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Serialize, Deserialize};


#[derive(Debug)]
enum AuthError {
    WrongCredentials,
    MissingCredentials,
    TokenCreation,
    InvalidToken
}

#[derive(Debug, Serialize, Deserialize)]
struct UserAuth {
    username: String,
    email: String,
    password: String
}

// pass back to the user.
#[derive(Debug, Serialize, Deserialize)]
struct UserResponseToken {
    access_token: String,
    token_type: String,
}

impl UserResponseToken {
    fn new (access_token: String) -> Self {
        Self {
            access_token,
            token_type: "Bearer".to_string()
        }
    }
}


// Struct to represent our Keys
struct Keys {
    encoding: EncodingKey,
    decoding: DecodingKey,
}

// implementation to support Encoding and Decoding the Secret from the supplied constructor
impl Keys {
    fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret)
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    // email
    sub: String,
    // expiration time
    exp: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct PrivateMessage {
    auth_token: String,
    private_message: String,
}

// this is the implementation for the FromRequestParts for Claims,
// this is JWT struct specific but it will allow us to use Claims to decide
// if we want to consume the request for accessing specific endpoints.
// this is all possible because of FromRequestParts from Axum
// https://docs.rs/axum/latest/axum/extract/trait.FromRequestParts.html
// if the request has a valid claim we all the system to respond,
// if not we aren't going to allow contact for the user within the api
// version 8.0 removes the #[async_trait]
// https://tokio.rs/blog/2025-01-01-announcing-axum-0-8-0
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = AuthError;
    // this will extract the token from the authentication header
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract the token from the authorization header
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError::InvalidToken)?;

        // decode the secret token
        // read the secret JWT KEY
        let jwt_secret: String = dotenvy::var("JWT_SECRET").expect("JWT_SECRET must be set");

        // set the key from the internal provided tokens
        let jwt_key = Keys::new(jwt_secret.as_bytes());

        // decode the user data's token from the .env provided
        // leveraging the decoding from the jwt_secret Keys
        let token_date = decode::<Claims>(bearer.token(), &jwt_key.decoding, &Validation::default())
            .map_err(|_| AuthError::InvalidToken)?;

        Ok(token_date.claims)
    }
}



// implement a struct for AuthError with the IntoResponse trait overridden
// this is all in the effort to use a custom response for Axum Responses
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

async fn login_user(Json(user_auth): Json<UserAuth>) -> Result<Json<UserResponseToken>, AuthError> {
    println!("Server POST: login user");

    // read the secret JWT KEY
    let jwt_secret: String = dotenvy::var("JWT_SECRET").expect("JWT_SECRET must be set");
    // set the key
    let jwt_key = Keys::new(jwt_secret.as_bytes());

    // check to ensure auth is supplied...
    if user_auth.username.is_empty() || user_auth.password.is_empty() || user_auth.email.is_empty() {
        return Err(AuthError::MissingCredentials);
    }

    // also check the db for the user details
    let expiration = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() + 300;

    // set the claims
    let claims = Claims {
        sub: user_auth.email.clone(),
        exp: usize::try_from(expiration).unwrap(),
    };

    // create an authorization token
    let token = encode(&Header::default(), &claims, &jwt_key.encoding).map_err(|_| AuthError::TokenCreation)?;

    Ok(Json(UserResponseToken::new(token)))
}

async fn private_message(_claims: Claims) -> Result<Json<serde_json::Value>, AuthError> {
    println!("Server GET: private message ");
    // demonstrate why authentication is important:

    let private_message = "🍑";

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

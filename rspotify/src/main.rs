use rspotify::{
    model::{AdditionalType, Country, Market},
    prelude::*,
    scopes, AuthCodeSpotify, Credentials, OAuth,
};

// Documentation: 
// https://github.com/ramsayleung/rspotify/blob/master/examples/auth_code.rs

#[tokio::main]
async fn main() {

    // Set RSPOTIFY_CLIENT_ID and RSPOTIFY_CLIENT_SECRET in an .env file (after
    // enabling the `env-file` feature) or export them manually:
    // Credentials Struct Documentation:
    // https://docs.rs/rspotify/latest/rspotify/struct.Credentials.html#method.from_env
    let creds = Credentials::from_env().unwrap();

    // these are the possible scopes from the crate
    // let scopes = scopes!(
    //     "user-read-email",
    //     "user-read-private",
    //     "user-top-read",
    //     "user-read-recently-played",
    //     "user-follow-read",
    //     "user-library-read",
    //     "user-read-currently-playing",
    //     "user-read-playback-state",
    //     "user-read-playback-position",
    //     "playlist-read-collaborative",
    //     "playlist-read-private",
    //     "user-follow-modify",
    //     "user-library-modify",
    //     "user-modify-playback-state",
    //     "playlist-modify-public",
    //     "playlist-modify-private",
    //     "ugc-image-upload"
    // );

    let oauth = OAuth::from_env(scopes!("user-read-currently-playing")).unwrap();

    let spotify = AuthCodeSpotify::new(creds, oauth);

    // Obtaining the access token
    let url = spotify.get_authorize_url(false).unwrap();
    // This function requires the `cli` feature enabled.
    // within the cargo.toml
    spotify.prompt_for_token(&url).await.unwrap();

    // Running the requests
    let market = Market::Country(Country::Spain);
    let additional_types = [AdditionalType::Episode];
    let artists = spotify
        .current_playing(Some(market), Some(&additional_types))
        .await;

    println!("Response: {artists:?}");
}
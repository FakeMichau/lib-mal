use lib_mal::*;
use tokio;
use std::env;

#[tokio::main]
async fn main() {
    let mut c = ClientBuilder::new().secret(include_str!("secret").to_string()).build_no_refresh();
    let parts = c.get_auth_parts();
    println!("URL: {}", parts.0);
    c.auth("localhost:2561", &parts.1, &parts.2).await.unwrap();
    println!("{}", c.get_access_token());
}

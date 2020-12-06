use authorizers::token::lookup_token;
use std::env;

type Error = Box<dyn std::error::Error + 'static>;
type Result<T> = std::result::Result<T,Error>;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let args: Vec<String> = env::args().collect();
    if let Some(token) = args.get(1) {
        let user_id = lookup_token(&token).await?;
        println!("user_id: {:?}", user_id);
    } else {
        println!("usage: lookup_token <token>");
    }
    Ok(())
}
use api::tokens;

#[tokio::main]
async fn main() {
    let token = tokens::generate_token().await.expect("generate");
    println!("{:?}", token);
}
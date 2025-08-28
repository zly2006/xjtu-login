use crate::login::Service;

mod login;

fn main() {
    let _ = dotenv::dotenv();
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();
    let client = login::login(
        Service::AiPlatform,
        &std::env::var("USERNAME").unwrap(),
        &std::env::var("PASSWORD").unwrap(),
    )
    .expect("login failed");
}

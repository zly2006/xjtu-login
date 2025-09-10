use std::time::Duration;
use tokio::time::sleep;
use crate::login::Service;
use xjtu_login::course::get_batch_list;

mod login;

#[tokio::main]
async fn main() {
    let _ = dotenv::dotenv();
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();
    let login = login::login(
        Service::CourseSelection,
        &std::env::var("USERNAME").unwrap(),
        &std::env::var("PASSWORD").unwrap(),
    )
    .await
    .expect("login failed");
    let session = xjtu_login::course::CourseSession::fron_client(login.client)
        .await
        .unwrap();
    let batch = get_batch_list(&session.client)
        .await
        .unwrap()
        .into_iter()
        .skip(1)
        .next()
        .unwrap();
    session
        .list_course(&batch, xjtu_login::course::CourseType::TJKC, 0, "国际结算")
        .await;
    session
        .delete_volunteer(&batch, "202520261FINA52091901")
        .await;
    sleep(Duration::from_secs_f32(0.3)).await;
    println!("{}", session.get_capacity("202520261FINA52091901").await);
    session
        .add_volunteer(
            &batch,
            "202520261FINA52091901",
            xjtu_login::course::CourseType::TJKC,
        )
        .await;
    println!("{}", session.get_capacity("202520261FINA52091901").await);
}

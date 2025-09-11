use crate::course::get_batch_list;
use crate::login::Service;
use std::time::Duration;
use tokio::time::sleep;

mod course;
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
    let session = course::CourseSession::fron_client(login.client)
        .await
        .unwrap();
    let batch = get_batch_list(&session.client)
        .await
        .unwrap()
        .into_iter()
        .nth(1)
        .unwrap();
    let courses = session
        .list_course(&batch, course::CourseType::TJKC, 0, "国际结算")
        .await;
    session
        .delete_volunteer(&batch, &courses[0].tc_list[0].teaching_class_id)
        .await;
    sleep(Duration::from_secs_f32(0.3)).await;
    println!(
        "{}",
        session
            .get_capacity(&courses[0].tc_list[0].teaching_class_id)
            .await
    );
    session
        .add_volunteer(
            &batch,
            &courses[0].tc_list[0].teaching_class_id,
            course::CourseType::TJKC,
        )
        .await;
    println!(
        "{}",
        session
            .get_capacity(&courses[0].tc_list[0].teaching_class_id)
            .await
    );
}

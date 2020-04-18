// use moonboard_api::moonboard_api::MoonboardAPI;

use moonboard::Moonboard;

use chrono::{DateTime, Utc};
use env_logger::{Builder, Env};
use failure::Error;
use std::env;

use serde::{Deserialize, Serialize};

// TODO(robin): add creation of bootstrap db binary

#[tokio::main]
async fn main() -> Result<(), Error> {
    Builder::from_env(Env::default().default_filter_or("info"))
        .format_indent(Some(4))
        .init();

    let mut board = Moonboard::new(env::var("MB_USER")?, env::var("MB_PASS")?, ".".to_owned()).await?;

    for i in 0..256 {
        println!("{:#?}", board.search_problems("r".to_owned()).await?.len());
    }
    /*
    println!("{:#?}", board.search_problems("r".to_owned()).await?.len());
    println!("{:#?}", board.search_problems("r".to_owned()).await?.len());
    println!("{:#?}", board.search_problems("r".to_owned()).await?.len());
    println!("{:#?}", board.search_problems("r".to_owned()).await?.len());
    */


    // let _create_sql = include_str!("schema.sql");
    // let mut conn = SqliteConnection::connect("sqlite://test6.db").await?;

    /*

    let mut conn = SqliteConnection::connect("sqlite::memory:").await?;
    conn.execute(create_sql).await?;

    use glob::glob;

    let mut tx = conn.begin().await?;

    for entry in glob("problems_*.json")? {
        let p = std::fs::read_to_string(entry?)?;
        let p: Problems = serde_json::from_str(&p)?;
        let p = p.data;

        for pp in p {
            problems::insert!(pp, |q| { q.execute(&mut tx).await? });
        }
    }

    tx.commit().await?;

    */

    // let users: Vec<User> = serde_json::from_str(&std::fs::read_to_string("users_beautiful.json")?)?;

    // let user_db = UserDB { last_update: Utc::now(), users };

    // let user_db: UserDB = bincode::deserialize(&std::fs::read("users.db")?)?;
    // println!("{}", user_db.users.len());
    // let encoded = bincode::serialize(&user_db)?;
    // std::fs::write("users.db", encoded)?;

    // println!("users {:#?}", users);

    // let api = MoonboardAPI::new(env::var("MB_USER")?, env::var("MB_PASS")?);
    // println!("holdsetups {:#?}", api.holdsetups().await?);

    // println!("all_problems: {:?}", api.all_problems().await?.len());

    // println!(
    //     "updates: {:?}",
    //     api.problem_updates(
    //         DateTime::parse_from_rfc3339("2020-04-01T00:00:00-00:00")?.naive_utc(),
    //         Some(DateTime::parse_from_rfc3339("2020-04-01T00:00:00-00:00")?.naive_utc()),
    //         Some(DateTime::parse_from_rfc3339("2020-04-01T00:00:00-00:00")?.naive_utc())
    //     )
    //     .await?
    //     .len()
    // );

    // println!("search username: {:?}", api.search_user("username").await?);

    // println!(
    //     "problem comments: {:?}",
    //     api.problem_comments(20153).await?.len()
    // );

    // println!(
    //     "problem repeats: {:?}",
    //     api.problem_repeats(20153).await?.len()
    // );

    Ok(())
}

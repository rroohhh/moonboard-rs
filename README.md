# moonboard-rs
This is a rust wrapper around the internal MoonBoard API that is used by the app.

Dissatisfaction with the existing app spawned this reverse engineering effort, that will hopefully help building a better app and better tools for using the MoonBoard.

This is still very much WIP and only a subset of the API is implemented. Currently supported:
- problem database download
- problem update download
- hold setup download
- user search
- repeats download
- comments download

Above the raw API layer a more ergonomic API that allows for fast queries and automatic syncing is planned.

## raw API layer basic usage
```rust
let api = MoonboardAPI::new(env::var("MB_USER")?, env::var("MB_PASS")?);
println!("holdsetups {:#?}", api.holdsetups().await?);

println!("all_problems: {:?}", api.all_problems().await?.len());

println!(
    "updates: {:?}",
    api.problem_updates(
        DateTime::parse_from_rfc3339("2020-04-01T00:00:00-00:00")?.naive_utc(),
        Some(DateTime::parse_from_rfc3339("2020-04-01T00:00:00-00:00")?.naive_utc()),
        Some(DateTime::parse_from_rfc3339("2020-04-01T00:00:00-00:00")?.naive_utc())
    )
    .await?
    .len()
);

println!("search username: {:?}", api.search_user("username").await?);

println!(
    "problem comments: {:?}",
    api.problem_comments(20153).await?.len()
);

println!(
    "problem repeats: {:?}",
    api.problem_repeats(20153).await?.len()
);
```

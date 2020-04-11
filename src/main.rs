use chrono::{Date, DateTime, FixedOffset, NaiveDate, NaiveDateTime};
use env_logger::{Builder, Env};
use epochs;
use failure::{format_err, Error, Fail};
use log::{debug, error, info};
use reqwest::Client;
use rgb::RGB8;
use serde::{
    de::{self, DeserializeOwned},
    Deserialize, Deserializer, Serialize,
};

use std::{cell::RefCell, env, fmt::Debug, str::FromStr, time::Duration};
use uuid::Uuid;

type Result<T> = std::result::Result<T, Error>;

#[derive(Deserialize, Serialize, Debug)]
enum ClientID {
    #[serde(rename = "com.moonclimbing.mb")]
    Moonclimbing,
}

#[derive(Serialize, Debug)]
enum GrantType {
    #[serde(rename = "password")]
    Password,
}

#[derive(Serialize, Debug)]
struct LoginRequest<'a> {
    username: &'a str,
    password: &'a str,
    grant_type: GrantType,
    client_id: ClientID,
}

#[derive(Deserialize, Debug)]
enum Role {
    #[serde(rename = "MoonBoard User")]
    User,
}

#[derive(Deserialize, Debug)]
enum TokenType {
    #[serde(rename = "bearer")]
    Bearer,
}

#[derive(Deserialize, Debug, Fail)]
#[serde(deny_unknown_fields, tag = "error", rename_all = "snake_case")]
enum TokenError {
    #[fail(display = "Token error: invalid_grant: {}", error_description)]
    InvalidGrant { error_description: String },
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
struct Token {
    #[serde(rename = ".expires", deserialize_with = "de_datetime_from_rfc2822")]
    expires: DateTime<FixedOffset>,
    #[serde(rename = ".issued", deserialize_with = "de_datetime_from_rfc2822")]
    issued: DateTime<FixedOffset>,
    #[serde(deserialize_with = "de_bool_from_str")]
    agree_terms: bool,
    firstname: String,
    lastname: String,
    #[serde(deserialize_with = "de_bool_from_str")]
    is_commercial: bool,
    nickname: String,
    role: Role,
    user_id: Uuid,
    #[serde(rename = "access_token")]
    access_token: String,
    #[serde(rename = "as:client_id")]
    as_client_id: ClientID,
    #[serde(rename = "expires_in", deserialize_with = "de_duration_seconds")]
    expires_in: Duration,
    #[serde(rename = "refresh_token")]
    refresh_token: String,
    #[serde(rename = "token_type")]
    token_type: TokenType,
    #[serde(rename = "userName")]
    username: String,
}

impl Token {
    fn is_expired(&self) -> bool {
        let now = chrono::Utc::now();

        now > self.expires
    }
}

fn de_rgb8_from_string<'de, D>(deserializer: D) -> std::result::Result<RGB8, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;

    if s.len() == 7 {
        if &s[0..1] == "#" {
            let r = u8::from_str_radix(&s[1..3], 16).map_err(de::Error::custom)?;
            let g = u8::from_str_radix(&s[3..5], 16).map_err(de::Error::custom)?;
            let b = u8::from_str_radix(&s[5..7], 16).map_err(de::Error::custom)?;

            return Ok(RGB8 { r, b, g });
        }
    }

    return Err(serde::de::Error::custom(format!(
        "invalid html color: {}",
        s
    )));
}

fn de_duration_seconds<'de, D>(deserializer: D) -> std::result::Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let seconds = u64::deserialize(deserializer)?;
    Ok(Duration::from_secs(seconds))
}

fn de_num_from_str<'de, D>(deserializer: D) -> std::result::Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    debug!("input: {}", s);
    u64::from_str(&s).map_err(de::Error::custom)
}

fn de_bool_from_str<'de, D>(deserializer: D) -> std::result::Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?.to_ascii_lowercase();
    bool::from_str(&s).map_err(de::Error::custom)
}

fn de_datetime_from_rfc2822<'de, D>(
    deserializer: D,
) -> std::result::Result<DateTime<FixedOffset>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    DateTime::parse_from_rfc2822(&s).map_err(serde::de::Error::custom)
}

fn de_datetime_from_rfc3339_no_tz<'de, D>(
    deserializer: D,
) -> std::result::Result<DateTime<FixedOffset>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;

    let fmt = if s.contains(".") {
        "%Y-%m-%dT%H:%M:%S.%f"
    } else {
        "%Y-%m-%dT%H:%M:%S"
    };

    let d = NaiveDateTime::parse_from_str(&s, fmt).map_err(serde::de::Error::custom)?;
    Ok(DateTime::from_utc(d, FixedOffset::east(0)))
}

fn de_datetime_from_rfc3339_no_tz_option<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<DateTime<FixedOffset>>, D::Error>
where
    D: Deserializer<'de>,
{
    <Option<String>>::deserialize(deserializer)?
        .map(|s| {
            let fmt = if s.contains(".") {
                "%Y-%m-%dT%H:%M:%S.%f"
            } else {
                "%Y-%m-%dT%H:%M:%S"
            };

            let d = NaiveDateTime::parse_from_str(&s, fmt).map_err(serde::de::Error::custom)?;
            Ok(DateTime::from_utc(d, FixedOffset::east(0)))
        })
        .map_or(Ok(None), |r| r.map(Some))
}

fn de_datetime_unix_timestamp<'de, D>(
    deserializer: D,
) -> std::result::Result<DateTime<FixedOffset>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;

    let timestamp = s
        .replace("/Date(", "")
        .replace(")/", "")
        .parse::<i64>()
        .map_err(serde::de::Error::custom)?;

    // milli seconds unix timestamp
    let d = epochs::java(timestamp)
        .ok_or_else(|| format_err!("could not parse time from timestamp {}", timestamp))
        .map_err(serde::de::Error::custom)?;

    Ok(DateTime::from_utc(d, FixedOffset::east(0)))
}

fn de_date_from_str<'de, D>(deserializer: D) -> std::result::Result<Date<FixedOffset>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;

    let fmt = "%d %b %Y";

    let d = NaiveDate::parse_from_str(&s, fmt).map_err(serde::de::Error::custom)?;
    Ok(Date::from_utc(d, FixedOffset::east(0)))
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged, deny_unknown_fields)]
enum SerdeUntaggedResult<A, B> {
    Ok(A),
    Err(B),
}

impl<A, B, C> Into<std::result::Result<A, C>> for SerdeUntaggedResult<A, B>
where
    C: From<B>,
{
    fn into(self) -> std::result::Result<A, C> {
        match self {
            SerdeUntaggedResult::Ok(a) => Ok(a),
            SerdeUntaggedResult::Err(b) => Err(b.into()),
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct Problems {
    total: u64,
    data: Vec<Problem>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct HoldSetFromProblem {
    api_id: HoldSetID,
    description: String,
    locations: Option<()>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct HoldSetupFromProblem {
    api_id: HoldSetupID,
    description: String,
    holdsets: Option<()>,
}

type HoldDirection = u64;
type HoldNumber = String;
type HoldRotation = u64;
type HoldType = u64;
type HoldId = u64;

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct HoldLocation {
    color: Option<()>,
    description: String,
    direction: HoldDirection,
    direction_string: String,
    // #[serde(deserialize_with = "de_num_from_str")]
    hold_number: HoldNumber,
    id: u64,
    rotation: HoldRotation,
    #[serde(rename = "type")]
    ty: u64,
    x: f64,
    y: f64,
    holdset: Option<()>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Hold {
    hold_type: HoldType,
    holdset_description: Option<()>,
    id: HoldId,
    location: HoldLocation,
    // #[serde(deserialize_with = "de_num_from_str")]
    number: HoldNumber,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct HoldSet {
    id: HoldSetID,
    #[serde(deserialize_with = "de_rgb8_from_string")]
    color: RGB8,
    api_id: Option<HoldSetID>,
    description: String,
    holds: Vec<Hold>,
}

type MoonBoardConfigurationID = u64;

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct MoonBoardConfiguration {
    description: String,
    high_grade: BoulderGrade,
    low_grade: BoulderGrade,
    id: MoonBoardConfigurationID,
}

type HoldLayoutId = u64;

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct HoldSetup {
    id: HoldSetupID,
    is_locked: bool,
    setby: Option<()>,
    api_id: Option<HoldSetupID>,
    description: String,
    holdsets: Vec<HoldSet>,
    active: bool,
    allow_climb_methods: bool,
    date_deleted: Option<()>,
    #[serde(deserialize_with = "de_datetime_from_rfc3339_no_tz")]
    date_inserted: DateTime<FixedOffset>,
    #[serde(deserialize_with = "de_datetime_from_rfc3339_no_tz")]
    date_updated: DateTime<FixedOffset>,
    hold_layout_id: HoldLayoutId,
    moon_board_configurations: Vec<MoonBoardConfiguration>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
enum BoulderMethod {
    #[serde(rename = "Feet follow hands")]
    FeetFollowHands,
    #[serde(rename = "Screw ons only")]
    ScrewOnsOnly,
    #[serde(rename = "Feet follow hands + screw ons")]
    FeetFollowHandsAndScrewOns,
    #[serde(rename = "Footless + kickboard")]
    FootlessAndKickBoard,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Move {
    description: MoveCoordinate,
    is_end: bool,
    is_start: bool,
    problem_id: ProblemID,
}

type BoulderGrade = String;
type Rating = u64;
type MoveCoordinate = String;
type ProblemID = u64;
type HoldSetID = u64;
type HoldSetupID = u64;

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Problem {
    api_id: ProblemID,
    #[serde(deserialize_with = "de_datetime_from_rfc3339_no_tz_option")]
    date_deleted: Option<DateTime<FixedOffset>>,
    #[serde(deserialize_with = "de_datetime_from_rfc3339_no_tz")]
    date_inserted: DateTime<FixedOffset>,
    #[serde(deserialize_with = "de_datetime_from_rfc3339_no_tz_option")]
    date_updated: Option<DateTime<FixedOffset>>,
    downgraded: bool,
    grade: BoulderGrade,
    has_beta_video: bool,
    holdsets: Vec<HoldSetFromProblem>,
    holdsetup: HoldSetupFromProblem,
    is_benchmark: bool,
    is_master: bool,
    method: BoulderMethod,
    moon_board_configuration_id: u64,
    moves: Vec<Move>,
    name: String,
    repeats: u64,
    setby: String,
    setby_id: Uuid,
    upgraded: bool,
    user_grade: Option<BoulderGrade>,
    user_rating: Option<Rating>,
}

#[derive(Serialize, Debug)]
struct UserSearch<'a> {
    #[serde(rename = "Query")]
    name: &'a str,
}

// #[derive(Deserialize, Debug)]
// #[serde(deny_unknown_fields, untagged)]
// enum UserStatus {
//     #[serde(rename = "0")]
//     Status0
// }

type UserStatus = u64;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct User {
    action_by_moon_id: Option<()>,
    #[serde(deserialize_with = "de_datetime_from_rfc3339_no_tz_option")]
    date_deleted: Option<DateTime<FixedOffset>>,
    #[serde(deserialize_with = "de_datetime_from_rfc3339_no_tz_option")]
    date_inserted: Option<DateTime<FixedOffset>>,
    #[serde(deserialize_with = "de_datetime_from_rfc3339_no_tz_option")]
    date_updated: Option<DateTime<FixedOffset>>,
    firstname: String,
    id: Uuid,
    lastname: String,
    nickname: String,
    status: UserStatus,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
enum NumberOfTries {
    Flashed,
    #[serde(rename = "more than 3 tries")]
    MoreThanThreeTries,
    #[serde(rename = "3rd try")]
    ThirdTry,
    #[serde(rename = "2nd try")]
    SecondTry,
    Project,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
struct RepeatOrComment {
    comment: Option<String>,
    attempts: u64,
    #[serde(deserialize_with = "de_datetime_unix_timestamp")]
    date_climbed: DateTime<FixedOffset>,
    #[serde(deserialize_with = "de_date_from_str")]
    date_climbed_as_string: Date<FixedOffset>,
    #[serde(deserialize_with = "de_datetime_from_rfc3339_no_tz_option")]
    date_inserted: Option<DateTime<FixedOffset>>,
    grade: Option<BoulderGrade>,
    id: u64,
    is_suggested_benchmark: bool,
    moon_board: Option<()>,
    number_of_tries: NumberOfTries,
    problem: Option<()>,
    rating: Option<Rating>,
    user: Option<UserFromRepeatOrComment>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
struct UserFromRepeatOrComment {
    can_share_data: bool,
    city: Option<String>,
    country: Option<String>,
    firstname: String,
    id: Uuid,
    lastname: String,
    nickname: String,
    profile_image_url: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
struct Paged<T> {
    aggregate_results: Option<()>,
    data: Vec<T>,
    errors: Option<()>,
    total: u64,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PagedQuery<'a> {
    sort: &'a str,
    page: u64,
    page_size: u64,
    group: &'a str,
    filter: String,
}

impl<'a> PagedQuery<'a> {
    fn comments_query(page: u64) -> PagedQuery<'a> {
        PagedQuery {
            sort: "",
            page,
            page_size: PAGE_SIZE,
            group: "",
            filter: "".to_string(),
        }
    }

    fn repeats_query(page: u64, problem_id: ProblemID) -> PagedQuery<'a> {
        PagedQuery {
            sort: "",
            page,
            page_size: PAGE_SIZE,
            group: "",
            filter: format!("Id~eq~{}", problem_id),
        }
    }
}

struct MoonboardAPI {
    token: RefCell<Option<Token>>,
    client: Client,
    username: String,
    password: String,
}

const WEBSITE_URL: &str = "https://moonboard.com";
const API_URL: &str = "https://restapimoonboard.ems-x.com";
const API_PATH: &str = "v1/_moonapi";
const PAGE_SIZE: u64 = 1000; // TODO(robin): seems to work for now

macro_rules! api_path {
    ($fmt: expr $(, $exprs:expr)*) => {
        format!("{}/{}/{}", API_URL, API_PATH, format!($fmt, $($exprs),*))
    };
}

macro_rules! website_path {
    ($fmt: expr $(, $exprs:expr)*) => {
        format!("{}/{}", WEBSITE_URL, format!($fmt, $($exprs),*))
    };
}

impl MoonboardAPI {
    fn new(username: String, password: String) -> MoonboardAPI {
        MoonboardAPI {
            token: RefCell::new(None),
            client: Client::new(),
            username,
            password,
        }
    }

    async fn refresh_token(&self, _refresh_token: &str) -> Result<Token> {
        unimplemented!();
    }

    async fn initial_login(&self) -> Result<Token> {
        let login_url = format!("{}/token", API_URL);

        let login_request = LoginRequest {
            username: &self.username,
            password: &self.password,
            client_id: ClientID::Moonclimbing,
            grant_type: GrantType::Password,
        };

        let response = self
            .client
            .post(&login_url)
            .form(&login_request)
            .send()
            .await?;

        let token: SerdeUntaggedResult<Token, TokenError> = response.json().await?;

        debug!("got initial login token: {:#?}", token);

        token.into()
    }

    async fn bearer_token(&self) -> Result<String> {
        let mut t = self.token.borrow_mut();

        if t.is_none() {
            *t = Some(self.initial_login().await?);
        }

        let t = t.as_mut().unwrap();

        if t.is_expired() {
            *t = self.refresh_token(&t.refresh_token).await?;
        }

        Ok(t.access_token.clone())
    }

    async fn api_get<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        info!("api get {}", url);

        let response = self
            .client
            .get(url)
            .bearer_auth(self.bearer_token().await?)
            .send()
            .await?;

        let parsed: T = response.json().await?;

        Ok(parsed)
    }

    async fn api_post_json<B: Serialize + Debug, T: DeserializeOwned>(
        &self,
        url: &str,
        body: B,
    ) -> Result<T> {
        info!("api post json {}, body: {:?}", url, body);

        let response = self
            .client
            .post(url)
            .bearer_auth(self.bearer_token().await?)
            .json(&body)
            .send()
            .await?;

        let parsed: T = response.json().await?;

        Ok(parsed)
    }

    async fn api_post_urlencoded<B: Serialize + Debug, T: DeserializeOwned>(
        &self,
        url: &str,
        body: B,
    ) -> Result<T> {
        info!("api post urlencoded {}, body: {:?}", url, body);

        let response = self.client.post(url).form(&body).send().await?;

        // println!("{}", response.text().await?);
        // unimplemented!()

        let parsed: T = response.json().await?;

        Ok(parsed)
    }

    // TODO(robin): this api seems to have atleast two more java timestamps as arguments,
    // but unsure what they do
    // (for example Holdsetup/637086364747630000/637117513200000000 )
    async fn holdsetups(&self) -> Result<Vec<HoldSetup>> {
        self.api_get(&api_path!("Holdsetup")).await
    }

    // TODO(robin): stall detection
    async fn download_problem(
        &self,
        next_url: &dyn Fn(ProblemID) -> String,
    ) -> Result<Vec<Problem>> {
        let mut problem_id: ProblemID = 0;
        let mut all_problems = Vec::new();

        loop {
            info!("downloading problems with offset: {}", problem_id);
            let mut problems: Problems = self.api_get(&next_url(problem_id)).await?;

            info!("problems left: {}", problems.total);

            // TODO(robin): maybe we want a set over the id?
            all_problems.append(&mut problems.data);

            problem_id = all_problems
                .last()
                .ok_or_else(|| format_err!("Got no problems from problem_id {}", problem_id))?
                .api_id;

            if problems.total <= 0 {
                break;
            }
        }

        Ok(all_problems)
    }

    async fn all_problems(&self) -> Result<Vec<Problem>> {
        self.download_problem(&|id| api_path!("problems/v2/{}", id))
            .await
    }

    async fn problem_updates(
        &self,
        date_inserted: NaiveDateTime,
        date_updated: Option<NaiveDateTime>,
        date_deleted: Option<NaiveDateTime>,
    ) -> Result<Vec<Problem>> {
        if date_updated.is_none() && date_deleted.is_some() {
            Err(format_err!(
                "Got a date_deleted, but no date_updated, that is not possible"
            ))
        } else {
            let mut postfix = format!("/{}", epochs::to_windows_date(date_inserted));

            if let Some(date_updated) = date_updated {
                postfix.push_str(&format!("/{}", epochs::to_windows_date(date_updated)));

                if let Some(date_deleted) = date_deleted {
                    postfix.push_str(&format!("/{}", epochs::to_windows_date(date_deleted)));
                }
            }

            self.download_problem(&|id| {
                let mut url = api_path!("problems/v2/{}", id);
                url.push_str(&postfix);

                url
            })
            .await
        }
    }

    async fn search_user(&self, pattern: &str) -> Result<Vec<User>> {
        self.api_post_json(&api_path!("Users/Search"), &UserSearch { name: pattern })
            .await
    }

    async fn all_users(&self) -> Result<Vec<User>> {
        // TODO(robin): is the api actually that dumb and gives us everything?
        self.search_user("").await
    }

    async fn download_paged<'a, T: DeserializeOwned>(
        &self,
        url: String,
        next_query: &dyn Fn(u64) -> PagedQuery<'a>,
    ) -> Result<Vec<T>> {
        let mut page = 1;
        let mut total = 0;
        let mut all_elems = Vec::new();

        loop {
            info!("downloading page {}", page);

            let mut elems: Paged<T> = self.api_post_urlencoded(&url, &next_query(page)).await?;

            if let Some(errors) = elems.errors {
                error!("error while downloading page: {:?}", errors);
            }

            if let Some(aggregate_results) = elems.aggregate_results {
                info!("aggregate_results: {:?}", aggregate_results);
            }

            total = total.max(elems.total);
            info!("new total: {}", elems.total);

            all_elems.append(&mut elems.data);

            if all_elems.len() >= total as usize {
                break;
            } else {
                page += 1;
            }
        }

        Ok(all_elems)
    }

    async fn problem_comments(&self, id: ProblemID) -> Result<Vec<RepeatOrComment>> {
        info!("downloading comments of problem {}", id);

        self.download_paged(
            website_path!("Problems/GetComments?problemId={}", id),
            &PagedQuery::comments_query,
        )
        .await
    }

    async fn problem_repeats(&self, id: ProblemID) -> Result<Vec<RepeatOrComment>> {
        info!("downloading repeats of problem {}", id);

        self.download_paged(website_path!("Problems/GetRepeats"), &|page| {
            PagedQuery::repeats_query(page, id)
        })
        .await
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    Builder::from_env(Env::default().default_filter_or("info"))
        .format_indent(Some(4))
        .init();

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

    Ok(())
}

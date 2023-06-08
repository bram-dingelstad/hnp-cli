use std::fs;

use lazy_static::lazy_static;
use regex::{Regex, RegexBuilder};
use serde::Serialize;
use serde_json::json;

type Id = i64;

#[derive(Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
struct Ticket {
    title: String,
    description: String,
    parent_id: Id,
    is_story: bool,
    category_id: Id,
    estimated_cost: f32,
    importance_level_id: Id,
    board_id: Id,
    start_date: String, // TODO: Convert to chrono / iso8601
    due_date: String,   // TODO: Convert to chrono / iso8601
    assigned_user_ids: Vec<Id>,
    tag_ids: Vec<Id>,
    sub_tasks: Vec<String>,
    dependency_ids: Vec<Id>,
}

#[derive(Debug)]
enum Tag {
    Category(Id, String),
    Tag(Id, String),
    UnaddedTag(String),
}

const API_ENDPOINT: &str = "https://api.hacknplan.com/v0";

lazy_static! {
    static ref API_KEY: String = std::env::var("HACKNPLAN_API_KEY")
        .expect("you to have set HACKNPLAN_API_KEY to a valid value");
    static ref PROJECT_ID: Id = std::env::var("HACKNPLAN_PROJECT_ID")
        .expect("you to set HACKNPLAN_PROJECT_ID to a valid value")
        .parse::<Id>()
        .expect("you to set HACKNPLAN_PROJECT_ID to a valid value");
    static ref HASH_TAG_MATCHER: Regex = Regex::new(r"#\w+").expect("Hash tag Regex to compile");
    static ref MENTION_MATCHER: Regex = Regex::new(r"@\w+").expect("Mention Regex to compile");
    static ref SUBTASK_MATCHER: Regex = RegexBuilder::new(r"^\[\].*$")
        .multi_line(true)
        .build()
        .expect("Subtask Regex to compile");
    static ref ESTIMATE_MATCHER: Regex =
        Regex::new(r"~((?<days>\d+)d)?((?<hours>\d+)h)?((?<minutes>\d+)m)?((?<seconds>\d+)s)?")
            .expect("Estimate Regex to compile");
    static ref URGENCY_MATCHER: Regex = Regex::new(r"!\w+").expect("Urgency Regex to compile");
}

async fn get_available_categories(client: &reqwest::Client) -> Vec<(Id, String)> {
    client
        .get(format!(
            "{API_ENDPOINT}/projects/{PROJECT_ID}/categories",
            PROJECT_ID = *PROJECT_ID
        ))
        .header(
            "Authorization",
            format!("ApiKey {API_KEY}", API_KEY = *API_KEY),
        )
        .send()
        .await
        .expect("To get categories from Hack'n'Plan")
        .json::<serde_json::Value>()
        .await
        .expect("To deserialize categories into JSON")
        .as_array()
        .expect("Categories results to be an array")
        .iter()
        .map(|value| {
            (
                value
                    .get("categoryId")
                    .expect("categoryId to be available")
                    .as_i64()
                    .expect("categoryId to be i64"),
                value
                    .get("name")
                    .expect("name to be available")
                    .as_str()
                    .expect("name to be String")
                    .to_owned(),
            )
        })
        .collect::<Vec<(Id, String)>>()
}

async fn get_available_users(client: &reqwest::Client) -> Vec<(Id, String, String)> {
    client
        .get(format!(
            "{API_ENDPOINT}/projects/{PROJECT_ID}/users",
            PROJECT_ID = *PROJECT_ID
        ))
        .header(
            "Authorization",
            format!("ApiKey {API_KEY}", API_KEY = *API_KEY),
        )
        .send()
        .await
        .expect("To get tags from Hack'n'Plan")
        .json::<serde_json::Value>()
        .await
        .expect("To deserialize tags into JSON")
        .as_array()
        .expect("Tags results to be an array")
        .iter()
        .map(|value| {
            let user = value
                .get("user")
                .expect("user to be available")
                .as_object()
                .expect("user to be an Object");

            (
                user.get("id")
                    .expect("user id to be available")
                    .as_i64()
                    .expect("tagId to be i64"),
                user.get("name")
                    .expect("name to be available")
                    .as_str()
                    .expect("name to be String")
                    .to_owned(),
                user.get("username")
                    .expect("username to be available")
                    .as_str()
                    .expect("username to be String")
                    .to_owned(),
            )
        })
        .collect::<Vec<(Id, String, String)>>()
}

// TODO: Perhaps implement board assignment?
#[allow(unused)]
async fn get_available_boards(client: &reqwest::Client) -> Vec<(Id, String)> {
    client
        .get(format!(
            "{API_ENDPOINT}/projects/{PROJECT_ID}/boards",
            PROJECT_ID = *PROJECT_ID
        ))
        .header(
            "Authorization",
            format!("ApiKey {API_KEY}", API_KEY = *API_KEY),
        )
        .send()
        .await
        .expect("To get boards from Hack'n'Plan")
        .json::<serde_json::Value>()
        .await
        .expect("To deserialize boards into JSON")
        .as_array()
        .expect("boards results to be an array")
        .iter()
        .map(|value| {
            (
                value
                    .get("boardId")
                    .expect("boardId to be available")
                    .as_i64()
                    .expect("boardId to be i64"),
                value
                    .get("name")
                    .expect("name to be available")
                    .as_str()
                    .expect("name to be String")
                    .to_owned(),
            )
        })
        .collect::<Vec<(Id, String)>>()
}

async fn get_available_importance_levels(client: &reqwest::Client) -> Vec<(Id, String, bool)> {
    client
        .get(format!(
            "{API_ENDPOINT}/projects/{PROJECT_ID}/importancelevels",
            PROJECT_ID = *PROJECT_ID
        ))
        .header(
            "Authorization",
            format!("ApiKey {API_KEY}", API_KEY = *API_KEY),
        )
        .send()
        .await
        .expect("To get importanceLevels from Hack'n'Plan")
        .json::<serde_json::Value>()
        .await
        .expect("To deserialize importanceLevels into JSON")
        .as_array()
        .expect("importanceLevels results to be an array")
        .iter()
        .map(|value| {
            (
                value
                    .get("importanceLevelId")
                    .expect("importanceLevelId to be available")
                    .as_i64()
                    .expect("importanceLevelId to be i64"),
                value
                    .get("name")
                    .expect("name to be available")
                    .as_str()
                    .expect("name to be String")
                    .to_owned(),
                value
                    .get("isDefault")
                    .expect("isDefault to be available")
                    .as_bool()
                    .expect("isDefault to be a bool"),
            )
        })
        .collect::<Vec<(Id, String, bool)>>()
}

async fn get_available_tags(client: &reqwest::Client) -> Vec<(Id, String)> {
    client
        .get(format!(
            "{API_ENDPOINT}/projects/{PROJECT_ID}/tags",
            PROJECT_ID = *PROJECT_ID
        ))
        .header(
            "Authorization",
            format!("ApiKey {API_KEY}", API_KEY = *API_KEY),
        )
        .send()
        .await
        .expect("To get tags from Hack'n'Plan")
        .json::<serde_json::Value>()
        .await
        .expect("To deserialize tags into JSON")
        .as_array()
        .expect("Tags results to be an array")
        .iter()
        .map(|value| {
            (
                value
                    .get("tagId")
                    .expect("tagId to be available")
                    .as_i64()
                    .expect("tagId to be i64"),
                value
                    .get("name")
                    .expect("name to be available")
                    .as_str()
                    .expect("name to be String")
                    .to_owned(),
            )
        })
        .collect::<Vec<(Id, String)>>()
}

async fn add_unmatched_tags(
    client: &reqwest::Client,
    unmatched_tags: Vec<String>,
    arguments: &Arguments,
) {
    for tag in unmatched_tags {
        let datum = json!({ "name": tag });
        if arguments.dry_run {
            println!("datum: {datum:#?}");
        } else {
            client
                .get(format!(
                    "{API_ENDPOINT}/projects/{PROJECT_ID}/categories",
                    PROJECT_ID = *PROJECT_ID
                ))
                .header(
                    "Authorization",
                    format!("ApiKey {API_KEY}", API_KEY = *API_KEY),
                )
                .json(&datum)
                .send()
                .await
                .expect("Creation of tag would go successfully");
        }
    }
}

fn match_tags_and_categories(
    title: &str,
    available_categories: &Vec<(Id, String)>,
    available_tags: &Vec<(Id, String)>,
) -> Vec<Tag> {
    HASH_TAG_MATCHER
        .find_iter(&title)
        .map(|hash_tag| {
            let hash_tag = hash_tag.as_str().replace("#", "").trim().to_owned();

            match available_categories
                .iter()
                .find(|(_, category)| category.to_lowercase() == hash_tag)
            {
                Some((id, category)) => Tag::Category(*id, category.to_owned()),
                None => match available_tags
                    .iter()
                    .find(|(_, tag)| tag.to_lowercase() == hash_tag)
                {
                    Some((id, tag)) => Tag::Tag(*id, tag.to_owned()),
                    None => Tag::UnaddedTag(hash_tag),
                },
            }
        })
        .collect::<Vec<Tag>>()
}

fn match_mentions<'a>(
    string: &'a str,
    available_users: &'a Vec<(Id, String, String)>,
) -> Vec<&'a (Id, String, String)> {
    MENTION_MATCHER
        .find_iter(&string)
        .map(|mention| {
            let user_name = mention.as_str().replace("@", "").trim().to_owned();

            available_users
                .iter()
                .find(|(_, name, _)| name.to_lowercase().matches(&user_name).count() != 0)
                .expect(&format!("To find a user for user_name: {user_name}"))
        })
        .collect::<Vec<&(Id, String, String)>>()
}

fn get_estimate(title: &str) -> f32 {
    let captures = ESTIMATE_MATCHER.captures(&title);
    if let Some(captures) = captures {
        let mut hours = 0.0;

        if let Some(days) = captures.name("days") {
            hours += days.as_str().parse::<f32>().expect("to parse days") * 8.0;
        }

        if let Some(hours_string) = captures.name("hours") {
            hours += hours_string
                .as_str()
                .parse::<f32>()
                .expect("to parse hours");
        }

        if let Some(minutes) = captures.name("minutes") {
            hours += minutes.as_str().parse::<f32>().expect("to parse minutes") / 60.0;
        }

        if let Some(seconds) = captures.name("seconds") {
            hours += seconds.as_str().parse::<f32>().expect("to parse seconds") / 3600.0;
        }

        hours
    } else {
        0.0
    }
}

fn get_importance_level(title: &str, available_importance_levels: &Vec<(Id, String, bool)>) -> Id {
    if let Some(urgency) = URGENCY_MATCHER.find(&title) {
        let urgency = urgency.as_str().replace('!', "");

        available_importance_levels
            .iter()
            .find(|(_, name, _)| name.to_lowercase().matches(&urgency).count() != 0)
            .expect(&format!("to find importance level for \"{urgency}\""))
            .0 // Access first element that represents the id
    } else {
        available_importance_levels
            .iter()
            .find(
                |level| level.2, // NOTE: This is where the isDefault bool lives, will auto-filter
            )
            .expect("atleast one importance level to be default")
            .0 // Access first element that represents the id
    }
}

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author = "Bram Dingelstad <bram@dingelstad.works>", version = "1.0")]
struct Arguments {
    #[arg(short, long)]
    dry_run: bool,

    #[arg(long)]
    default_category: Option<String>,

    file: std::path::PathBuf,
}

#[tokio::main]
async fn main() {
    let arguments = Arguments::parse();

    let contents = fs::read_to_string(&arguments.file).expect("To read file");
    let default_category: Option<&str> = None; //Some("programming");

    let texts = contents
        .split("---")
        .filter(|text| text.trim().len() > 0) // Remove empty texts (usually trailing)
        .collect::<Vec<&str>>();

    let client = reqwest::Client::new();

    let available_categories = get_available_categories(&client).await;
    let available_tags = get_available_tags(&client).await;
    let available_users = get_available_users(&client).await;
    let available_importance_levels = get_available_importance_levels(&client).await;

    // Pre-pass for checking tags and verifying data
    let mut unmatched_tags: Vec<String> = vec![];
    for text in &texts {
        // FIXME: Verify that there is only one '===' in the string (double tickets)
        let mut chunks = text.split("===");
        let title = chunks.next().unwrap().trim().to_owned();

        match_tags_and_categories(&title, &available_categories, &available_tags)
            .iter()
            .filter_map(|tag_or_category| {
                if let Tag::UnaddedTag(tag) = tag_or_category {
                    Some(tag.to_owned())
                } else {
                    None
                }
            })
            .for_each(|tag| unmatched_tags.push(tag));
    }

    unmatched_tags.sort();
    unmatched_tags.dedup();

    if !arguments.dry_run && unmatched_tags.len() > 0 {
        match inquire::Confirm::new(&format!("Could not find tags on Hack'n'Plan for the following list, would you like to add these in bulk?\n{unmatched_tags:#?}"))
                .with_default(false)
                .prompt() {
            Ok(true) => {},
            _ => return
        }
    }

    add_unmatched_tags(&client, unmatched_tags, &arguments).await;

    let available_tags = get_available_tags(&client).await;

    let mut tickets: Vec<Ticket> = vec![];
    for text in &texts {
        let mut chunks = text.split("===");
        let title = chunks.next().unwrap().trim().to_owned();

        let title = if let Some(category) = default_category {
            format!("{title} #{category}")
        } else {
            title
        };

        let categories_or_tags =
            match_tags_and_categories(&title, &available_categories, &available_tags);
        let mentions = match_mentions(&title, &available_users);
        let estimate = get_estimate(&title);
        let importance_level = get_importance_level(&title, &available_importance_levels);
        // TODO: Implement dependencies
        // let dependencies =

        // Remove all entries of tags, mentions
        let title = HASH_TAG_MATCHER.replace_all(&title, "");
        let title = MENTION_MATCHER.replace_all(&title, "");
        let title = ESTIMATE_MATCHER.replace_all(&title, "");
        let title = URGENCY_MATCHER.replace_all(&title, "");

        // Remove all double spaces
        let title = title
            .trim()
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ");

        let description = chunks.next().unwrap_or("").trim().to_owned();

        let description = MENTION_MATCHER
            .replace_all(&description, |capture: &regex::Captures| {
                let mention = capture.get(0).unwrap().as_str().replace('@', "");

                let user_name = available_users
                    .iter()
                    .find(|(_, name, _)| name.to_lowercase().matches(&mention).count() != 0)
                    .expect(&format!("To find a user for user_name: {mention}"))
                    .2 // NOTE: This is the third entry in the tuple: the `user_name`
                    .to_owned();

                format!("@{user_name}")
            })
            .to_string();

        let subtasks = SUBTASK_MATCHER
            .find_iter(&description)
            .map(|subtask| subtask.as_str().replace("[]", "").trim().to_owned())
            .collect::<Vec<String>>();

        let description = SUBTASK_MATCHER
            .replace_all(&description, "")
            .trim()
            .to_string();

        tickets.push(Ticket {
            title: title.to_owned(),
            description,
            assigned_user_ids: mentions.iter().map(|(id, _, _)| *id).collect::<Vec<Id>>(),
            tag_ids: categories_or_tags
                .iter()
                .filter_map(|entry| match entry {
                    Tag::Tag(id, _) => Some(*id),
                    _ => None,
                })
                .collect::<Vec<Id>>(),
            category_id: categories_or_tags
                .iter()
                .find_map(|entry| {
                    if let Tag::Category(id, _) = entry {
                        Some(*id)
                    } else {
                        None
                    }
                })
                .expect(&format!(
                    "To have atleast one category available for ticket: {title}"
                )),
            estimated_cost: estimate,
            sub_tasks: subtasks,
            importance_level_id: importance_level,
            ..Default::default()
        });
    }

    for ticket in tickets {
        if !arguments.dry_run {
            println!(
                "☁️ Uploading ticket:\n{}",
                serde_json::to_string_pretty(&ticket).unwrap()
            );

            client
                .get(format!(
                    "{API_ENDPOINT}/projects/{PROJECT_ID}/categories",
                    PROJECT_ID = *PROJECT_ID
                ))
                .header(
                    "Authorization",
                    format!("ApiKey {API_KEY}", API_KEY = *API_KEY),
                )
                .json(&ticket)
                .send()
                .await
                .expect(&format!(
                    r#"to send ticket "{}" successfully"#,
                    ticket.title
                ))
                .error_for_status()
                .expect(&format!(
                    r#"to send ticket "{}" successfully"#,
                    ticket.title
                ));
        } else {
            println!(
                "💨 \"Pretend\" Uploading ticket:\n{}",
                serde_json::to_string_pretty(&ticket).unwrap()
            );
        }
    }
}

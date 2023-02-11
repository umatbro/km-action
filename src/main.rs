mod github_pull_request;

use chrono::Local;

use std::env;
use std::fs::File;
use std::io::BufReader;

use github_pull_request::Event;

#[tokio::main]
async fn main() {
    set_github_output_env();
    let event = get_pr_details();
    let github_token = get_github_token();
    let octo = octocrab::OctocrabBuilder::new()
        .personal_token(github_token)
        .build()
        .unwrap();
    let set_body_result = event
        .set_pr_body(&octo, &String::from("body from job"))
        .await;
    println!(
        "PR after update is: {:?}",
        set_body_result.expect("Error while updating PR")
    );
}

fn set_github_output_env() {
    let time = Local::now().to_string();
    println!("::set-output name=time::{}", time);
    println!("::set-output name=pr_number::[PR-NUM]");
}

/// `GITHUB_EVENT_PATH`
///
/// The path to the file on the runner that contains the full event webhook payload.
/// For example, `/github/workflow/event.json`.
///
/// https://docs.github.com/en/actions/learn-github-actions/variables
fn get_pr_details() -> Event {
    let event_path = env::var("GITHUB_EVENT_PATH");
    let p = event_path.expect("GITHUB_EVENT_PATH not found.");

    let f = File::open(p).unwrap();
    let reader = BufReader::new(f);
    let parsed = serde_json::from_reader(reader).unwrap();

    println!("Data from GITHUB_EVENT_PATH: {:?}", &parsed);
    parsed
}

fn get_github_token() -> String {
    let github_token = env::var("GITHUB_TOKEN").expect(
        "Env GITHUB_TOKEN not found. Modify your config file to pass it to the action.\n\
See example in https://github.com/marketplace/actions/github-api-request#usage",
    );
    println!("Github token: {}", github_token);
    github_token
}

mod common_lib_handler;
mod description_manipulator;
mod github_pull_request;

use octocrab::Octocrab;
use std::env;
use std::fs::File;
use std::io::BufReader;

use crate::common_lib_handler::{get_client_for_repo_from_installations, GithubSetupError};
use crate::github_pull_request::Event;

#[macro_use]
extern crate pest_derive;

#[tokio::main]
async fn main() {
    set_github_output_env();
    let event = get_pr_details();
    let github_token = get_github_token();
    let octo = octocrab::OctocrabBuilder::new()
        .personal_token(github_token)
        .build()
        .unwrap();

    let lib_repo_octo = get_client_for_repo_from_installations(&octo, "km-action").await;
    let lib_repo_octo = match lib_repo_octo {
        Ok(oct) => oct,
        Err(e) => panic!("There was an error authenticating lib repo: {:?}", e),
    };
    let pulls = lib_repo_octo
        .pulls(event.repository.get_owner(), event.repository.name)
        .list()
        .send()
        .await
        .expect("There was an error downloading pull requests from lib repo.")
        .take_items();
    println!("Pulls from lib repo: {:?}", pulls);
    let body_to_set = description_manipulator::get_update_body(&event.pull_request);
    let set_body_result = event.set_pr_body(&octo, &body_to_set).await;
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
/// <https://docs.github.com/en/actions/learn-github-actions/variables>
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

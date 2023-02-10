use std::env;
use std::fs::{write, File};
use std::io::Read;
use chrono::Local;

fn main() {
    set_github_output_env();
    print_pr_link();
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
fn print_pr_link() {
    let github_token = env::var("GITHUB_TOKEN").expect(
        "Env GITHUB_TOKEN not found. Modify your config file to pass it to the action.\n\
See example in https://github.com/marketplace/actions/github-api-request#usage",
    );
    println!("Github token: {}", github_token);
    let event_path = env::var("GITHUB_EVENT_PATH");
    if let Err(e) = event_path {
        panic!("GITHUB_EVENT_PATH not found.");
    }
    let p = event_path.unwrap();
    let mut f = File::open(p).unwrap();
    let mut buf = String::new();
    f.read_to_string(&mut buf).unwrap();
    println!("Data from GITHUB_EVENT_PATH: {}", buf);
}

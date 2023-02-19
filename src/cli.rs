use octocrab::models::AppId;
use std::env;
use url::{ParseError, Url};

pub struct PemContents(pub String);
pub struct LibRepoName(pub String);
pub struct JiraLink {
    host: Url,
}

impl JiraLink {
    pub fn parse(input: &str) -> Result<Self, ParseError> {
        Ok(Self {
            host: Url::parse(input)?,
        })
    }

    pub fn ticket_url(&self, ticket_num: &str) -> String {
        self.host
            .join("browse/")
            .unwrap()
            .join(ticket_num)
            .unwrap()
            .to_string()
    }
}

pub fn read_cli_args() -> Result<(AppId, PemContents, LibRepoName, Option<JiraLink>), String> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 5 {
        return Err(String::from(
            "Usage: program <app_id> <pem file contents> <lib_repo_name>",
        ));
    }

    let app_id = match args[1].parse::<u64>() {
        Ok(n) => n,
        Err(_) => {
            return Err(format!("Invalid input: {}", args[1]));
        }
    };

    let pem_contents = &args[2];

    let lib_name = &args[3];

    let jira_link = JiraLink::parse(&args[4]);
    let jira_link = match jira_link {
        Ok(v) => Some(v),
        Err(e) => {
            eprintln!("There was an error parsing JIRA host: {}", e);
            None
        }
    };

    Ok((
        AppId(app_id),
        PemContents(pem_contents.clone()),
        LibRepoName(lib_name.clone()),
        jira_link,
    ))
}

#[cfg(test)]
mod tests {
    use crate::cli::JiraLink;
    use rstest::rstest;

    #[rstest]
    #[case("BACK-1234", "https://test.com/browse/BACK-1234")]
    #[case("", "https://test.com/browse/")]
    #[case("xx", "https://test.com/browse/xx")]
    fn test_generate_link(#[case] ticket_num: &str, #[case] expected_result: &str) {
        let jira_url = JiraLink::parse("https://test.com").unwrap();
        let result = jira_url.ticket_url(ticket_num);

        assert_eq!(result, expected_result);
    }
}

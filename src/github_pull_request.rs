use octocrab;
use octocrab::Octocrab;
use pest::Parser;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Event {
    pub pull_request: PullRequest,
    pub repository: Repository,
}

#[derive(Deserialize, Debug)]
pub struct PullRequest {
    pub number: i32,
    pub body: String,
    pub title: String,
}

impl PullRequest {
    /// Get ticket number from the PR title.
    /// * PR title has to start with the ticket number.
    /// * Ticket number has to be inside square brackets.
    pub fn get_ticket_number(&self) -> Result<Vec<String>, pest::error::Error<Rule>> {
        parse_pr_title(&self.title)
    }
}

#[derive(Parser)]
#[grammar = "pr_title.pest"]
struct PrTitleParser;

fn parse_pr_title(input: &str) -> Result<Vec<String>, pest::error::Error<Rule>> {
    let parse_result = PrTitleParser::parse(Rule::pr_title, input)?;
    Ok(parse_result
        .flatten()
        .filter_map(|pair| match pair.as_rule() {
            Rule::ticket_num => Some(String::from(pair.as_str())),
            _ => None,
        })
        .collect())
}

#[derive(Deserialize, Debug)]
pub struct Repository {
    pub name: String,
    pub full_name: String,
}

impl Repository {
    pub fn get_owner(&self) -> Result<String, String> {
        let parts: Vec<&str> = self.full_name.split('/').collect();
        if parts.len() != 2 {
            return Err(String::from(&self.full_name));
        }
        let result = parts[0];
        Ok(result.to_string())
    }
}

impl Event {
    pub async fn set_pr_body(
        &self,
        octo: &Octocrab,
        body: &String,
    ) -> octocrab::Result<octocrab::models::pulls::PullRequest> {
        let result = octo
            .pulls(&self.repository.get_owner().unwrap(), &self.repository.name)
            .update(self.pull_request.number as u64)
            .body(body)
            .send()
            .await;
        result
    }
}

#[cfg(test)]
mod tests {
    use super::Event;
    use super::Rule;
    use crate::github_pull_request::parse_pr_title;
    use pest::error::ErrorVariant;
    use rstest::rstest;
    use serde_json;
    use std::fs::File;
    use std::io::BufReader;

    #[test]
    fn test_parse_event() {
        let file = File::open("src/test_resources/action_payload.json").unwrap();
        let reader = BufReader::new(file);
        let result: Event = serde_json::from_reader(reader).unwrap();

        assert_eq!(result.pull_request.number, 2);
        assert_eq!(result.pull_request.body, "next trigger");
        assert_eq!(result.pull_request.title, "update cargo");
        assert_eq!(result.repository.name, "km-dep");
        assert_eq!(result.repository.get_owner().unwrap(), "umatbro");
        assert_eq!(result.repository.full_name, "umatbro/km-dep");
    }

    #[rstest]
    #[case("[BACK-1337] Test", vec!["BACK-1337"])]
    #[case("[BACK-1337][MD-1212] Test", vec!["BACK-1337", "MD-1212"])]
    fn test_parse_pr_title(#[case] pr_title: &str, #[case] expected_ticket_nums: Vec<&str>) {
        let result = parse_pr_title(pr_title);
        assert_eq!(expected_ticket_nums, result.unwrap());
    }

    #[rstest]
    #[case("No ticket number", Rule::ticket_num_section)]
    #[case("", Rule::ticket_num_section)]
    #[case("Pr number at the end [PRD-30]", Rule::ticket_num_section)]
    #[case("[99] Wrong ticket num format", Rule::ticket_num)]
    fn test_failed_parse_pr_title(#[case] pr_title: &str, #[case] rule: Rule) {
        let result = parse_pr_title(pr_title);
        assert!(result.is_err());
        let variant = result.err().unwrap().variant;
        match variant {
            ErrorVariant::ParsingError {
                positives,
                negatives,
            } => {
                assert_eq!(positives.len(), 1);
                assert_eq!(negatives.len(), 0);
                match rule {
                    Rule::ticket_num_section => {
                        assert!(matches!(positives[0], Rule::ticket_num_section))
                    }
                    Rule::ticket_num => assert!(matches!(positives[0], Rule::ticket_num)),
                    _ => panic!("Unexpected variant."),
                }
            }
            _ => panic!("The error variant is incorrect."),
        }
    }
}

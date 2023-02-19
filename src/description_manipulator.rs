use crate::cli::JiraLink;
use crate::github_pull_request::PullRequest;


/// This package contains code that updates the PR's body.

const COMMENT_START: &'static str = "<!-- START KM-ACTION -->";
const COMMENT_END: &'static str = "<!-- END KM-ACTION -->";

/// Get a updated body of Pull Request description.
/// * Add a line saying that Github action modified the body.
/// * Add a line with found ticket numbers (based on `pull_request` parameter).
/// * Add a line with found open PRs in the lib repo (based on `lib_repo_pull_requests` parameter).
pub fn get_update_body(
    pull_request: &PullRequest,
    lib_repo_pull_requests: &Vec<PullRequest>,
    jira_link: &Option<JiraLink>,
) -> String {
    let current_body = &pull_request.body;
    let mut lines: Vec<String> = current_body.lines().map(|i| i.to_string()).collect();

    let mut lines_added = vec![];

    lines_added.push(COMMENT_START.to_string());
    lines_added.push(String::from("---"));
    lines_added.push(String::from("### 🤖 This is update from km-action."));
    lines_added.push(String::from(""));
    lines_added.push(get_ticket_number_line(pull_request, jira_link));
    for line in get_lib_prs_lines(pull_request, lib_repo_pull_requests) {
        lines_added.push(line);
    }
    lines_added.push(COMMENT_END.to_string());

    let added_description = find_lines_assigned_by_action(&lines);

    match added_description {
        Some(lines_found) => {
            lines.drain(lines_found.from..=lines_found.to);
            lines.splice(lines_found.from..lines_found.from, lines_added);
        }
        None => lines.extend(lines_added),
    }

    let mut result = lines.join("\n");
    if !(result.ends_with("\n")) {
        result.push_str("\n");
    }

    result
}

fn get_ticket_number_line(pull_request: &PullRequest, jira_link: &Option<JiraLink>) -> String {
    let ticket_numbers = pull_request.get_ticket_number();
    if let Err(_) = ticket_numbers {
        return String::from("❓Ticket number: **Not Found**");
    }
    let mut result = String::from("✅ Ticket number: **");
    let mut ticket_numbers = ticket_numbers
        .unwrap()
        .into_iter()
        .map(|tick_num| match jira_link {
            None => tick_num,
            Some(jl) => format!("[{}]({})", tick_num, jl.ticket_url(&tick_num)),
        })
        .collect::<Vec<String>>();
    ticket_numbers.sort();
    let ticket_numbers_str = ticket_numbers.join(", ");
    result.push_str(&ticket_numbers_str);
    result.push_str("**");

    result
}

fn get_lib_prs_lines(
    main_pull_request: &PullRequest,
    lib_pull_requests: &Vec<PullRequest>,
) -> Vec<String> {
    let main_pr_ticket_nums = main_pull_request.get_ticket_number();
    if let Err(_) = main_pr_ticket_nums {
        return vec![];
    }
    let main_pr_ticket_nums = main_pr_ticket_nums.unwrap();
    let matching_lib_pull_requests: Vec<&PullRequest> = lib_pull_requests
        .iter()
        .filter(|pr| {
            let ticket_numbers = pr.get_ticket_number();
            if let Err(_) = ticket_numbers {
                return false;
            }
            let ticket_numbers = ticket_numbers.unwrap();
            // if ticket numbers from lib PR intersect with ticket numbers in main repo - this PR should be included.
            return !ticket_numbers.is_disjoint(&main_pr_ticket_nums);
        })
        .collect();
    if matching_lib_pull_requests.len() == 0 {
        return vec![];
    }

    let mut lines = Vec::with_capacity(matching_lib_pull_requests.len() + 2);
    lines.push(String::from(""));
    lines.push(String::from("📦 Related lib PRs:"));
    for lib_pull_request in matching_lib_pull_requests
        .into_iter()
        .filter(|pr| pr.html_url.is_some())
    {
        let pr_link = lib_pull_request.html_url.as_ref().unwrap();
        lines.push(format!("* {}", pr_link));
    }
    lines
}

struct LinesAssignedByAction {
    from: usize,
    to: usize,
}

/// The action adds text to a PR body. This text is wrapped in distinct tags (see `COMMENT_START`
/// and `COMMENT_END` const values).
/// The `find_lines_assigned_by_action` identifies those lines and returns indices of lines containing
/// contents added automatically by the action.
fn find_lines_assigned_by_action(lines: &Vec<String>) -> Option<LinesAssignedByAction> {
    let mut iter = lines.into_iter().enumerate();
    let mut first_line_index = None;
    let mut last_line_index = None;

    while let Some(item) = iter.next() {
        let (index, line) = item;
        if line.eq(COMMENT_START) {
            first_line_index = Some(index);
            break;
        }
    }
    while let Some(item) = iter.next() {
        let (index, line) = item;
        if line.eq(COMMENT_END) {
            last_line_index = Some(index);
            break;
        }
    }

    match (first_line_index, last_line_index) {
        (Some(from), Some(to)) => Some(LinesAssignedByAction { from, to }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::cli::JiraLink;
    use crate::description_manipulator::{find_lines_assigned_by_action, get_update_body};
    use crate::github_pull_request::PullRequest;
    use rstest::rstest;
    use std::fs::File;
    use std::io::Read;
    use std::path::Path;

    fn read_test_file_content(file_name: &str) -> String {
        let path = Path::new("src/test_resources").join(file_name);
        let mut f = File::open(path).unwrap();
        let mut contents = String::new();
        f.read_to_string(&mut contents).unwrap();
        contents
    }

    #[rstest]
    #[case(
        "data1_in.md",
        "data1_out.md",
        "[BACK-1] Test PR",
        vec![
            String::from("Title with no ticket number"),
            String::from("[BACK-2] Title with non-matching other ticket number.")
        ],
        Some(JiraLink::parse("https://test.com").unwrap())
    )]
    #[case("data2_in.md", "data2_out.md", "", vec![String::from("[BACK-1] Non matching ticket.")], None)]
    #[case("data3_in.md", "data3_out.md", "Invalid title", vec![], None)]
    #[case(
        "data4_in.md",
        "data4_out.md",
        "[BACK-42][MD-1337] Two ticket numbers.",
        vec![
            String::from("One invalid PR title"),
            String::from("[BACK-42] Valid ticket number from lib repo"),
        ],
        None,
    )]
    fn test_get_updated_body(
        #[case] data_in: &str,
        #[case] data_out: &str,
        #[case] pull_request_title: &str,
        #[case] lib_pull_requests_titles: Vec<String>,
        #[case] jira_host: Option<JiraLink>,
    ) {
        let data_in = read_test_file_content(data_in);
        let data_out = read_test_file_content(data_out);
        let pull_request = PullRequest {
            number: 0,
            body: data_in,
            title: pull_request_title.to_string(),
            html_url: None,
        };

        let lib_pull_requests: Vec<PullRequest> = lib_pull_requests_titles
            .iter()
            .map(|title| PullRequest {
                title: title.clone(),
                html_url: Some(String::from("https://test.com")),
                ..Default::default()
            })
            .collect();
        let result = get_update_body(&pull_request, &lib_pull_requests, &jira_host);
        assert_eq!(data_out, result);
    }

    #[rstest]
    #[case("data1_in.md", None)]
    #[case("data1_out.md", Some((0, 5)))]
    #[case("data2_in.md", None)]
    #[case("data2_out.md", Some((1, 6)))]
    #[case("data3_in.md", Some((2, 5)))]
    fn test_find_lines_assigned_by_action(
        #[case] data_in: &str,
        #[case] line_from_to: Option<(usize, usize)>,
    ) {
        let lines: Vec<String> = read_test_file_content(data_in)
            .lines()
            .map(|i| i.to_string())
            .collect();
        let result = find_lines_assigned_by_action(&lines);
        match line_from_to {
            None => assert!(result.is_none()),
            Some((expected_from, expected_to)) => {
                assert!(result.is_some());
                let result = result.unwrap();
                assert_eq!(expected_from, result.from);
                assert_eq!(expected_to, result.to);
            }
        }
    }
}

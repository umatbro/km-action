/// This package contains code that updates the PR's body.

const COMMENT_START: &'static str = "<!-- START KM-ACTION -->";
const COMMENT_END: &'static str = "<!-- END KM-ACTION -->";

pub fn get_update_body(input: &str) -> String {
    let mut lines: Vec<String> = input.lines().map(|i| i.to_string()).collect();
    let added_description = find_lines_assigned_by_action(&lines);

    if let None = added_description {
        lines.push(COMMENT_START.to_string());
        lines.push(String::from("---"));
        lines.push(String::from("🤖 This is update from km-action."));
        lines.push(COMMENT_END.to_string());
        lines.push(String::from(""));
    }
    let mut result = lines.join("\n");
    if !(result.ends_with("\n")) {
        result.push_str("\n");
    }

    result
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
    use crate::description_manipulator::{find_lines_assigned_by_action, get_update_body};
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
    #[case("data1_in.md", "data1_out.md")]
    #[case("data2_in.md", "data2_out.md")]
    #[case("data3_in.md", "data3_out.md")]
    #[case("data4_in.md", "data4_out.md")]
    fn test_get_updated_body(#[case] data_in: &str, #[case] data_out: &str) {
        let data_in = read_test_file_content(data_in);
        let data_out = read_test_file_content(data_out);

        let result = get_update_body(&data_in);
        assert_eq!(data_out, result);
    }

    #[rstest]
    #[case("data1_in.md", None)]
    #[case("data1_out.md", Some((0, 3)))]
    #[case("data2_in.md", None)]
    #[case("data2_out.md", Some((1, 4)))]
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

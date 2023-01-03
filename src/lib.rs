use regex::Regex;

pub fn path_to_regex(pattern: &str) -> Regex {
    let mut regex = String::new();

    // Special case backslash can match a backslash file or directory
    if pattern.starts_with("\\") {
        return Regex::new(r"\\(?:\z|/)").unwrap();
    }

    let anchored = pattern
        .find("/")
        .map_or(false, |pos| pos != pattern.len() - 1);

    if anchored {
        regex += r"\A";
    } else {
        regex += r"(?:\A|/)";
    }

    let matches_dir = pattern.ends_with("/");
    let mut pattern = pattern;
    if matches_dir {
        pattern = pattern.trim_end_matches("/");
    }

    // patterns ending with "/*" are special. They only match items directly in the directory
    // not deeper
    let trailing_slash_star = pattern.len() > 1 && pattern.ends_with("/*");

    let mut iterator = pattern.chars().enumerate();

    // Anchored paths may or may not start with a slash
    if anchored && pattern.starts_with("/") {
        iterator.next();
        regex += r"/?";
    }

    let mut num_to_skip = None;
    for (i, ch) in iterator {
        if let Some(skip_amount) = num_to_skip {
            num_to_skip = Some(skip_amount - 1);
            continue;
        }
        if ch == '*' {
            // Handle double star (**) case properly
            if i + 1 < pattern.len() && pattern.chars().nth(i + 1) == Some('*') {
                let left_anchored = i == 0;
                let leading_slash = i > 0 && pattern.chars().nth(i - 1) == Some('/');
                let right_anchored = i + 2 == pattern.len();
                let trailing_slash =
                    i + 2 < pattern.len() && pattern.chars().nth(i + 2) == Some('/');

                if (left_anchored || leading_slash) && (right_anchored || trailing_slash) {
                    regex += ".*";
                    num_to_skip = Some(2);
                    continue;
                }
            }
            regex += r"[^/]*";
        } else if ch == '?' {
            regex += r"[^/]";
        } else {
            regex += &regex::escape(ch.to_string().as_str());
        }
    }

    if matches_dir {
        regex += "/";
    } else if trailing_slash_star {
        regex += r"\z";
    } else {
        regex += r"(?:\z|/)";
    }
    Regex::new(&regex).unwrap()
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_to_regex() {
        let pattern = "*.txt";
        let regex = path_to_regex(pattern);
        assert!(regex.is_match("file.txt"));
        assert!(regex.is_match("file.txt/"));
        assert!(regex.is_match("dir/file.txt"));

        let pattern = "/dir/*.txt";
        let regex = path_to_regex(pattern);
        assert!(regex.is_match("/dir/file.txt"));
        assert!(regex.is_match("dir/file.txt"));
        assert!(!regex.is_match("/dir/subdir/file.txt"));

        let pattern = "apps/";
        let regex = path_to_regex(pattern);
        assert!(regex.is_match("apps/file.txt"));
        assert!(regex.is_match("/apps/file.txt"));
        assert!(regex.is_match("/dir/apps/file.txt"));
        assert!(regex.is_match("/dir/subdir/apps/file.txt"));

        let pattern = "docs/*";
        let regex = path_to_regex(pattern);
        assert!(regex.is_match("docs/getting-started.md"));
        // should not match on nested files
        assert!(!regex.is_match("docs/build-app/troubleshooting.md"));

        let pattern = "/docs/";
        let regex = path_to_regex(pattern);
        assert!(regex.is_match("/docs/file.txt"));
        assert!(regex.is_match("/docs/subdir/file.txt"));
        assert!(!regex.is_match("app/docs/file.txt"));
    }
}

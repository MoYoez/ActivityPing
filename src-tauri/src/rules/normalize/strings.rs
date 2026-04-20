pub(super) fn normalize_string_list(values: &[String], lowercase: bool) -> Vec<String> {
    let mut result = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for value in values {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            continue;
        }
        let key = trimmed.to_lowercase();
        if !seen.insert(key.clone()) {
            continue;
        }
        result.push(if lowercase { key } else { trimmed.to_string() });
    }

    result
}

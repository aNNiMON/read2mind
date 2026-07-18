use chrono::Local;

pub(crate) fn get_non_empty_title(v1: Option<String>, v2: Option<String>) -> String {
    filter_non_blank(v1)
        .or_else(|| filter_non_blank(v2))
        .unwrap_or_else(|| Local::now().format("%Y-%m-%d %H:%M:%S").to_string())
}

pub(crate) fn filter_non_blank(v: Option<String>) -> Option<String> {
    v.filter(|s| !s.trim().is_empty())
}

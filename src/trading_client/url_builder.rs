use std::collections::HashMap;

/// Build a URL from a base, path template, optional path params, and optional query params.
///
/// Path params: `<param>` for required, `<param?>` for optional.
/// Query params: supports single values and lists.
pub fn build_url(
    base: &str,
    path: &str,
    path_params: &HashMap<&str, String>,
    query_params: &[(&str, QueryValue)],
) -> String {
    let mut url = format!("{}{}", base, path);

    // Replace path parameters
    for (key, value) in path_params {
        // Try required param first: <key>
        let required_placeholder = format!("<{}>", key);
        if url.contains(&required_placeholder) {
            url = url.replace(&required_placeholder, value);
        }
        // Try optional param: <key?>
        let optional_placeholder = format!("<{}?>", key);
        if url.contains(&optional_placeholder) {
            if value.is_empty() {
                url = url.replace(&optional_placeholder, "");
            } else {
                url = url.replace(&optional_placeholder, value);
            }
        }
    }

    // Clean up any remaining optional params
    while let Some(start) = url.find('<') {
        if let Some(end) = url.find('>') {
            let param = &url[start + 1..end];
            if param.ends_with('?') {
                url = url.replace(&url[start..=end], "");
            } else {
                break;
            }
        } else {
            break;
        }
    }

    // Remove trailing slashes
    url = url.trim_end_matches('/').to_string();

    // Build query string
    let mut query_parts: Vec<String> = Vec::new();
    for (key, value) in query_params {
        match value {
            QueryValue::None => {}
            QueryValue::Single(v) => {
                query_parts.push(format!("{}={}", key, v));
            }
            QueryValue::List(values) => {
                for v in values {
                    query_parts.push(format!("{}={}", key, v));
                }
            }
        }
    }

    if !query_parts.is_empty() {
        url.push('?');
        url.push_str(&query_parts.join("&"));
    }

    url
}

#[derive(Debug)]
pub enum QueryValue {
    None,
    Single(String),
    List(Vec<String>),
}

#[allow(dead_code)]
impl QueryValue {
    pub fn from_opt_str(v: Option<&str>) -> Self {
        match v {
            Some(s) => QueryValue::Single(s.to_string()),
            None => QueryValue::None,
        }
    }

    pub fn from_opt_i64(v: Option<i64>) -> Self {
        match v {
            Some(n) => QueryValue::Single(n.to_string()),
            None => QueryValue::None,
        }
    }

    pub fn from_opt_list(v: Option<&[String]>) -> Self {
        match v {
            Some(list) if !list.is_empty() => {
                QueryValue::List(list.iter().map(|s| s.to_string()).collect())
            }
            _ => QueryValue::None,
        }
    }
}

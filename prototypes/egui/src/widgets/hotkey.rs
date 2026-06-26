//! Format XUI accelerator specs (`"control|alt|shift|KEY"`) for display only; no key handling.

/// Mac-symbol form: `control` becomes `⌘`, `alt` becomes `⌥`, `shift` becomes `⇧`, emitted as `⌥⇧⌘` then the key.
pub fn mac(spec: &str) -> String {
    if spec.is_empty() {
        return String::new();
    }

    let mut alt = false;
    let mut shift = false;
    let mut cmd = false;
    let mut key = "";

    for token in spec.split('|') {
        match token {
            "control" => cmd = true,
            "alt" => alt = true,
            "shift" => shift = true,
            other => key = other,
        }
    }

    let mut out = String::new();
    if alt {
        out.push('⌥');
    }
    if shift {
        out.push('⇧');
    }
    if cmd {
        out.push('⌘');
    }

    if key.chars().count() == 1 {
        out.extend(key.chars().flat_map(char::to_uppercase));
    } else {
        out.push_str(key);
    }

    out
}

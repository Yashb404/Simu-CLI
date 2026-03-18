pub fn generate_iframe_snippet(src: &str, width: &str, height: &str) -> String {
    format!(
        "<iframe src=\"{}\" sandbox=\"allow-scripts allow-same-origin\" loading=\"lazy\" referrerpolicy=\"no-referrer\" style=\"width:{};height:{};border:0;border-radius:12px;\"></iframe>",
        src, width, height
    )
}

pub fn generate_script_snippet(src: &str, demo_id: &str) -> String {
    let target_id = format!("cli-demo-{}", demo_id);
    format!(
        "<div id=\"{}\"></div>\n<script src=\"{}\" data-demo-id=\"{}\" data-target=\"#{}\" async></script>",
        target_id, src, demo_id, target_id
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iframe_snippet_contains_src_and_sandbox() {
        let html = generate_iframe_snippet("https://example.com/d/abc", "100%", "480px");
        assert!(html.contains("https://example.com/d/abc"));
        assert!(html.contains("sandbox=\"allow-scripts allow-same-origin\""));
    }

    #[test]
    fn script_snippet_contains_target_and_demo_id() {
        let html = generate_script_snippet("https://example.com/static/bootstrap.js", "demo-123");
        assert!(html.contains("data-demo-id=\"demo-123\""));
        assert!(html.contains("data-target=\"#cli-demo-demo-123\""));
    }
}

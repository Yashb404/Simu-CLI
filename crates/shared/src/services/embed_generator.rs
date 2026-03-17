pub fn generate_iframe_snippet(src: &str, width: &str, height: &str) -> String {
    format!(
        "<iframe src=\"{}\" sandbox=\"allow-scripts allow-same-origin\" loading=\"lazy\" referrerpolicy=\"no-referrer\" style=\"width:{};height:{};border:0;border-radius:12px;\"></iframe>",
        src, width, height
    )
}

pub fn generate_script_snippet(src: &str, demo_id: &str) -> String {
    format!(
        "<div id=\"cli-demo\"></div>\n<script src=\"{}\" data-demo-id=\"{}\"></script>",
        src, demo_id
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
}

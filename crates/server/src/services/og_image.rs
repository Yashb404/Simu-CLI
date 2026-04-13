pub fn generate_og_svg(title: &str, version: i32) -> String {
    format!(
        r#"<svg xmlns='http://www.w3.org/2000/svg' width='1200' height='630'>
<defs>
  <linearGradient id='g' x1='0' x2='1' y1='0' y2='1'>
    <stop offset='0%' stop-color='#0f172a' />
    <stop offset='100%' stop-color='#111827' />
  </linearGradient>
</defs>
<rect width='1200' height='630' fill='url(#g)'/>
<text x='80' y='220' fill='#e5e7eb' font-size='64' font-family='JetBrains Mono'>SimuCLI</text>
<text x='80' y='320' fill='#93c5fd' font-size='42' font-family='JetBrains Mono'>{}</text>
<text x='80' y='390' fill='#9ca3af' font-size='28' font-family='JetBrains Mono'>Version {}</text>
</svg>"#,
        html_escape::encode_text(title),
        version
    )
}

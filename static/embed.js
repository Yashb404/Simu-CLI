(function embedCliDemo() {
  var script = document.currentScript;
  if (!script) {
    var scripts = document.getElementsByTagName("script");
    script = scripts[scripts.length - 1] || null;
  }

  if (!script) {
    console.error("[cli-demo-studio] Unable to locate embed script.");
    return;
  }

  var demoId = script.getAttribute("data-demo") || script.getAttribute("data-demo-id");
  if (!demoId) {
    console.error("[cli-demo-studio] Missing data-demo attribute.");
    return;
  }

  var scriptUrl = new URL(script.src, window.location.href);
  var apiBaseAttr = script.getAttribute("data-api-base");
  var apiBase = apiBaseAttr ? new URL(apiBaseAttr, scriptUrl.origin).origin : scriptUrl.origin;
  var runtimeUrl = new URL("/embed-runtime/index.html", apiBase);
  runtimeUrl.searchParams.set("demo_id", demoId);
  runtimeUrl.searchParams.set("api_base", apiBase);

  var targetSelector = script.getAttribute("data-target");
  var mountNode = targetSelector ? document.querySelector(targetSelector) : null;

  if (!mountNode && script.previousElementSibling && script.previousElementSibling.tagName === "DIV") {
    mountNode = script.previousElementSibling;
  }

  if (!mountNode) {
    mountNode = document.createElement("div");
    if (script.parentNode) {
      script.parentNode.insertBefore(mountNode, script.nextSibling);
    }
  }

  var iframe = document.createElement("iframe");
  iframe.src = runtimeUrl.toString();
  iframe.loading = "lazy";
  iframe.referrerPolicy = "strict-origin-when-cross-origin";
  iframe.sandbox = script.getAttribute("data-sandbox") || "allow-scripts allow-same-origin";
  iframe.style.width = script.getAttribute("data-width") || "100%";
  iframe.style.height = script.getAttribute("data-height") || "480px";
  iframe.style.border = "2px solid #000";
  iframe.style.borderRadius = "0";
  iframe.style.background = "#fff";

  mountNode.innerHTML = "";
  mountNode.appendChild(iframe);
})();

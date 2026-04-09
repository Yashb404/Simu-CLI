(function bootstrapCliDemo() {
  var script = document.currentScript;
  if (!script) {
    var scripts = document.getElementsByTagName("script");
    script = scripts[scripts.length - 1] || null;
  }

  if (!script) {
    return;
  }

  var demoId = script.getAttribute("data-demo") || script.getAttribute("data-demo-id");
  if (!demoId) {
    console.warn("[cli-demo-studio] Missing data-demo attribute on bootstrap script.");
    return;
  }

  var scriptUrl = new URL(script.src, window.location.href);
  var iframeUrl = new URL("/embed/index.html", scriptUrl.origin);
  iframeUrl.searchParams.set("demo_id", demoId);

  var targetSelector = script.getAttribute("data-target");
  var mountNode = null;

  if (targetSelector) {
    mountNode = document.querySelector(targetSelector);
  }

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
  iframe.src = iframeUrl.toString();
  iframe.loading = "lazy";
  iframe.referrerPolicy = "no-referrer";
  iframe.sandbox = "allow-scripts allow-same-origin";
  iframe.style.width = script.getAttribute("data-width") || "100%";
  iframe.style.height = script.getAttribute("data-height") || "480px";
  iframe.style.border = "2px solid #000";
  iframe.style.background = "#fff";

  mountNode.innerHTML = "";
  mountNode.appendChild(iframe);
})();

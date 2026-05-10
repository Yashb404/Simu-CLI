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

  // data-demo-config contains the full demo JSON (PublicDemoResponse) inline.
  // This is the preferred mode: the embed runs entirely client-side after load.
  var demoConfigAttr = script.getAttribute("data-demo-config");
  // data-demo / data-demo-id is a legacy fallback that fetches demo data from
  // the server after the iframe loads. Prefer data-demo-config for offline use.
  var demoId = script.getAttribute("data-demo") || script.getAttribute("data-demo-id");

  if (!demoConfigAttr && !demoId) {
    console.error("[cli-demo-studio] Missing data-demo-config (or data-demo) attribute.");
    return;
  }

  var scriptUrl = new URL(script.src, window.location.href);

  // data-runtime-base must be an absolute URL pointing to the separately
  // deployed standalone terminal runtime (the compiled embed WASM app).
  // This decouples the iframe entirely from the main website: only the
  // standalone terminal origin is contacted after the page loads.
  // When omitted, falls back to /embed-runtime/index.html on the script
  // origin for backward compatibility.
  var runtimeBaseAttr = script.getAttribute("data-runtime-base");
  var runtimeUrl;
  if (runtimeBaseAttr) {
    try {
      runtimeUrl = new URL("/index.html", new URL(runtimeBaseAttr).origin);
    } catch (e) {
      console.error("[cli-demo-studio] data-runtime-base must be an absolute URL.", e);
      return;
    }
  } else {
    runtimeUrl = new URL("/embed-runtime/index.html", scriptUrl.origin);
  }

  var apiBaseAttr = script.getAttribute("data-api-base");
  var apiBase = apiBaseAttr ? new URL(apiBaseAttr, scriptUrl.origin).origin : null;

  if (demoConfigAttr) {
    // Pass the demo JSON directly in the URL so the runtime never needs to
    // contact the server for data after the page has loaded.
    runtimeUrl.searchParams.set("demo_config", demoConfigAttr);
  } else {
    // Legacy: runtime will fetch demo data from the server.
    runtimeUrl.searchParams.set("demo_id", demoId);
    if (apiBase) {
      runtimeUrl.searchParams.set("api_base", apiBase);
    }
  }

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

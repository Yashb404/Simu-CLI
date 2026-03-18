(function() {
    // 1. Find the script tag that invoked us
    var currentScript = document.currentScript;
    if (!currentScript) {
        console.error("CLI Demo Studio: Cannot find script tag.");
        return;
    }

    // 2. Extract the demo ID
    var demoId = currentScript.getAttribute("data-demo");
    if (!demoId) {
        console.error("CLI Demo Studio: Missing data-demo attribute.");
        return;
    }

    // 3. Determine host URL from the script source
    var scriptUrl = new URL(currentScript.src);
    var baseUrl = scriptUrl.origin;

    // 4. Construct the iframe for the embed runtime
    var iframe = document.createElement("iframe");
    iframe.src = baseUrl + "/embed-runtime/index.html?demo_id=" + encodeURIComponent(demoId) + "&api_base=" + encodeURIComponent(baseUrl);
    iframe.style.width = "100%";
    iframe.style.height = "480px";
    iframe.style.border = "2px solid #000";
    iframe.style.borderRadius = "0";
    iframe.setAttribute("sandbox", "allow-scripts allow-same-origin");
    iframe.setAttribute("loading", "lazy");

    // 5. Inject right after the script tag
    if (currentScript.parentNode) {
        currentScript.parentNode.insertBefore(iframe, currentScript.nextSibling);
    }
})();

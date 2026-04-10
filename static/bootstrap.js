// Legacy bootstrap entrypoint.
// Keep this file as a thin compatibility shim so existing embeds continue to work.
(function bootstrapCliDemo() {
  var script = document.currentScript;
  if (!script) {
    return;
  }

  var replacement = document.createElement("script");
  replacement.src = new URL("./embed.js", script.src).toString();

  var attrs = script.attributes;
  for (var i = 0; i < attrs.length; i += 1) {
    replacement.setAttribute(attrs[i].name, attrs[i].value);
  }

  if (script.parentNode) {
    script.parentNode.insertBefore(replacement, script);
    script.parentNode.removeChild(script);
  }
})();

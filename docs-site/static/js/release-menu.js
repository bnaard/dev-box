(function () {
  "use strict";

  var menu = document.querySelector(".td-release-menu");
  if (!menu) return;

  function pagePath(siteBase) {
    var root = new URL(siteBase).pathname;
    var path = window.location.pathname;
    if (path.indexOf(root) === 0) path = path.slice(root.length);
    path = path.replace(/^v[01]\.x\/v\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?\//, "");
    path = path.replace(/^v1\.x\//, "");
    return path;
  }

  function item(label, url, active, muted) {
    var li = document.createElement("li");
    var link = document.createElement("a");
    link.className = "dropdown-item" + (active ? " active" : "");
    link.href = new URL(pagePath(url.siteBase), url.base).href;
    link.textContent = label;
    if (muted) link.classList.add("td-release-menu__alias");
    if (active) link.setAttribute("aria-current", "page");
    li.appendChild(link);
    return li;
  }

  fetch(menu.dataset.releaseManifest, { credentials: "omit" })
    .then(function (response) {
      if (!response.ok) throw new Error("release manifest unavailable");
      return response.json();
    })
    .then(function (manifest) {
      var fragment = document.createDocumentFragment();
      var currentBase = menu.dataset.currentBase.replace(/\/$/, "");

      manifest.lines.forEach(function (line) {
        var headingItem = document.createElement("li");
        var heading = document.createElement("h6");
        heading.className = "dropdown-header";
        heading.textContent = line.line;
        headingItem.appendChild(heading);
        fragment.appendChild(headingItem);

        var currentActive = line.current.url.replace(/\/$/, "") === currentBase;
        fragment.appendChild(item(
          "Current (" + line.current.version + ")",
          { base: line.current.url, siteBase: manifest.siteBase },
          currentActive,
          true
        ));

        line.releases.forEach(function (release) {
          var active = release.url.replace(/\/$/, "") === currentBase;
          fragment.appendChild(item(
            release.version,
            { base: release.url, siteBase: manifest.siteBase },
            active,
            false
          ));
        });
      });

      menu.replaceChildren(fragment);
    })
    .catch(function () {
      // Hugo-rendered line aliases remain available as an offline fallback.
    });
})();

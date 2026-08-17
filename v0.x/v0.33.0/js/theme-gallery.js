(function () {
  "use strict";

  function button(label, active, select) {
    var element = document.createElement("button");
    element.type = "button";
    element.className = "btn btn-sm " + (active ? "btn-primary" : "btn-outline-secondary");
    element.textContent = label;
    element.addEventListener("click", select);
    return element;
  }

  function render(root, index) {
    var themes = index.themes || [];
    var family = "all";
    var mode = "all";
    var families = Array.from(new Set(themes.map(function (item) { return item.family; }))).sort();
    var modes = Array.from(new Set(themes.map(function (item) { return item.mode; }))).sort();

    function draw() {
      root.replaceChildren();
      var filters = document.createElement("div");
      filters.className = "theme-gallery-filters";

      [["Family", families, family, function (value) { family = value; draw(); }],
       ["Mode", modes, mode, function (value) { mode = value; draw(); }]]
        .forEach(function (group) {
          var row = document.createElement("div");
          row.className = "theme-filter-row";
          var label = document.createElement("strong");
          label.textContent = group[0];
          row.appendChild(label);
          row.appendChild(button("All", group[2] === "all", function () { group[3]("all"); }));
          group[1].forEach(function (value) {
            row.appendChild(button(value.charAt(0).toUpperCase() + value.slice(1),
              group[2] === value, function () { group[3](value); }));
          });
          filters.appendChild(row);
        });
      root.appendChild(filters);

      var filtered = themes.filter(function (theme) {
        return (family === "all" || theme.family === family) &&
          (mode === "all" || theme.mode === mode);
      });
      var count = document.createElement("p");
      count.className = "text-secondary";
      count.textContent = "Showing " + filtered.length + " of " + themes.length + " themes";
      root.appendChild(count);

      var grid = document.createElement("div");
      grid.className = "theme-gallery-grid";
      filtered.forEach(function (theme) {
        var card = document.createElement("article");
        card.className = "theme-card";
        var header = document.createElement("header");
        var title = document.createElement("strong");
        title.textContent = theme.slug;
        var meta = document.createElement("small");
        meta.textContent = [theme.family, theme.mode, theme.variant].filter(Boolean).join(" / ");
        header.append(title, meta);
        card.appendChild(header);

        var player = document.createElement("div");
        player.className = "theme-player";
        if (theme.cast) {
          player.classList.add("asciinema");
          player.dataset.cast = root.dataset.base + theme.slug + ".cast";
          player.dataset.autoplay = "false";
          player.dataset.poster = "npt:0:5";
          player.dataset.cols = "160";
          player.dataset.rows = "45";
        } else {
          player.textContent = "cast not yet recorded";
        }
        card.appendChild(player);
        grid.appendChild(card);
      });
      root.appendChild(grid);
      if (window.initAsciinemaPlayers) window.initAsciinemaPlayers();
    }

    draw();
  }

  document.addEventListener("DOMContentLoaded", function () {
    var root = document.getElementById("theme-gallery");
    if (!root) return;
    fetch(root.dataset.index)
      .then(function (response) {
        if (!response.ok) throw new Error("HTTP " + response.status);
        return response.json();
      })
      .then(function (index) { render(root, index); })
      .catch(function (error) {
        root.innerHTML = "<p class=\"alert alert-danger\">Unable to load theme gallery: " +
          error.message.replace(/[<>&]/g, "") + "</p>";
      });
  });
}());

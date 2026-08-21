(() => {
  const chooser = document.querySelector("[data-theme-chooser]");
  if (!chooser) return;

  const selections = { mode: "any", appearance: "any", contrast: "any", tool: "any" };
  const matches = [...chooser.querySelectorAll("[data-match]")];
  const count = chooser.querySelector("[data-match-count]");

  function update() {
    let visible = 0;
    for (const match of matches) {
      const tools = match.dataset.tools.split(" ");
      const selected = (selections.mode === "any" || match.dataset.mode === selections.mode)
        && (selections.appearance === "any" || match.dataset.appearance === selections.appearance)
        && (selections.contrast === "any" || match.dataset.contrast === selections.contrast)
        && (selections.tool === "any" || tools.includes(selections.tool));
      match.hidden = !selected;
      if (selected) visible += 1;
    }
    count.textContent = String(visible);
  }

  for (const group of chooser.querySelectorAll("[data-filter]")) {
    group.addEventListener("click", (event) => {
      const button = event.target.closest("button[data-value]");
      if (!button) return;
      for (const peer of group.querySelectorAll("button[data-value]")) peer.setAttribute("aria-pressed", "false");
      button.setAttribute("aria-pressed", "true");
      selections[group.dataset.filter] = button.dataset.value;
      update();
    });
  }

  chooser.querySelector("[data-theme-tool]").addEventListener("change", (event) => {
    selections.tool = event.target.value;
    update();
  });
})();

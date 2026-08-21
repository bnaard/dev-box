/* @ds-bundle: {"format":4,"namespace":"ProjectiousWorkDesignSystem_019de8","components":[{"name":"StatusPill","sourcePath":"ui_kits/agent-console/PipelineList.jsx"},{"name":"PipelineList","sourcePath":"ui_kits/agent-console/PipelineList.jsx"},{"name":"RunDetail","sourcePath":"ui_kits/agent-console/RunDetail.jsx"},{"name":"Sidebar","sourcePath":"ui_kits/agent-console/Sidebar.jsx"},{"name":"StatusBar","sourcePath":"ui_kits/agent-console/StatusBar.jsx"},{"name":"Topbar","sourcePath":"ui_kits/agent-console/Topbar.jsx"},{"name":"THEMES","sourcePath":"ui_kits/agent-console/theme.jsx"},{"name":"CTA","sourcePath":"ui_kits/marketing-site/CTA.jsx"},{"name":"CodeShowcase","sourcePath":"ui_kits/marketing-site/CodeShowcase.jsx"},{"name":"Convictions","sourcePath":"ui_kits/marketing-site/Convictions.jsx"},{"name":"Footer","sourcePath":"ui_kits/marketing-site/Footer.jsx"},{"name":"Header","sourcePath":"ui_kits/marketing-site/Header.jsx"},{"name":"Hero","sourcePath":"ui_kits/marketing-site/Hero.jsx"},{"name":"Pillars","sourcePath":"ui_kits/marketing-site/Pillars.jsx"}],"sourceHashes":{"preview/card-mode.js":"1f4fd56b7b3d","ui_kits/agent-console/PipelineList.jsx":"71965b6175b6","ui_kits/agent-console/RunDetail.jsx":"cc75d58f8f1f","ui_kits/agent-console/Sidebar.jsx":"598be646be7b","ui_kits/agent-console/StatusBar.jsx":"ce1d5ffe6468","ui_kits/agent-console/Topbar.jsx":"604b6fe9e02f","ui_kits/agent-console/theme.jsx":"cf625aebba93","ui_kits/marketing-site/CTA.jsx":"d110f781261b","ui_kits/marketing-site/CodeShowcase.jsx":"05de65e28826","ui_kits/marketing-site/Convictions.jsx":"bb0de458ad16","ui_kits/marketing-site/Footer.jsx":"750c3f5fc786","ui_kits/marketing-site/Header.jsx":"fb8c1189b294","ui_kits/marketing-site/Hero.jsx":"c600876a3bb1","ui_kits/marketing-site/Pillars.jsx":"63741835ee0c"},"inlinedExternals":[],"unexposedExports":[]} */

(() => {

const __ds_ns = (window.ProjectiousWorkDesignSystem_019de8 = window.ProjectiousWorkDesignSystem_019de8 || {});

const __ds_scope = {};

(__ds_ns.__errors = __ds_ns.__errors || []);

// preview/card-mode.js
try { (() => {
/* preview/card-mode.js — the appearance switch every card carries.
   Three appearances, matching the system's three supported modes:
     light      body[data-theme="light"]
     deep dark  body[data-theme="dark"]
     navy dark  body[data-theme="dark"][data-surface="navy"]
   The attributes go on <body>, not <html>: a card may be embedded as markup
   rather than loaded as a document, and an <html> attribute is lost when it is.
   Cards that show a fixed palette as their *content* (a light ramp, a dark
   ramp) opt out with `data-mode-lock` and only re-chrome themselves. */
(function () {
  var MODES = [{
    id: 'light',
    label: 'Light',
    theme: 'light',
    surface: null
  }, {
    id: 'navy',
    label: 'Navy dark',
    theme: 'dark',
    surface: null
  }, {
    id: 'deep',
    label: 'Deep dark',
    theme: 'dark',
    surface: 'deep'
  }];
  function apply(id) {
    var m = MODES.filter(function (x) {
      return x.id === id;
    })[0] || MODES[0];
    var b = document.body;
    b.setAttribute('data-theme', m.theme);
    if (m.surface) {
      b.setAttribute('data-surface', m.surface);
    } else {
      b.removeAttribute('data-surface');
    }
    b.setAttribute('data-mode', m.id);
    document.documentElement.setAttribute('data-theme', m.theme);
    if (m.surface) {
      document.documentElement.setAttribute('data-surface', m.surface);
    } else {
      document.documentElement.removeAttribute('data-surface');
    }
    Array.prototype.forEach.call(document.querySelectorAll('[data-mode-btn]'), function (btn) {
      btn.setAttribute('aria-pressed', btn.getAttribute('data-mode-btn') === m.id ? 'true' : 'false');
    });
    document.dispatchEvent(new CustomEvent('cardmode', {
      detail: m
    }));
  }
  function build() {
    var bar = document.createElement('div');
    bar.className = 'mode-switch';
    bar.setAttribute('role', 'group');
    bar.setAttribute('aria-label', 'Appearance');
    MODES.forEach(function (m) {
      var b = document.createElement('button');
      b.type = 'button';
      b.textContent = m.label;
      b.setAttribute('data-mode-btn', m.id);
      b.setAttribute('aria-pressed', 'false');
      b.addEventListener('click', function () {
        apply(m.id);
      });
      bar.appendChild(b);
    });
    document.body.insertBefore(bar, document.body.firstChild);
    apply(document.body.getAttribute('data-mode') || 'light');
  }
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', build);
  } else {
    build();
  }
  window.cardMode = {
    apply: apply,
    current: function () {
      return document.body.getAttribute('data-mode');
    }
  };
})();
})(); } catch (e) { __ds_ns.__errors.push({ path: "preview/card-mode.js", error: String((e && e.message) || e) }); }

// ui_kits/agent-console/PipelineList.jsx
try { (() => {
function StatusPill({
  status,
  theme
}) {
  const m = theme.pill[status] || theme.pill.idle;
  const labelMap = {
    healthy: 'Healthy',
    running: 'Running',
    warning: 'Warning',
    failed: 'Failed',
    idle: 'Idle'
  };
  return /*#__PURE__*/React.createElement("span", {
    style: {
      display: 'inline-flex',
      alignItems: 'center',
      gap: 6,
      padding: '2px 9px',
      borderRadius: 9999,
      background: m.bg,
      color: m.fg,
      fontFamily: 'var(--font-body)',
      fontSize: 11,
      fontWeight: 500
    }
  }, /*#__PURE__*/React.createElement("span", {
    style: {
      width: 6,
      height: 6,
      borderRadius: '50%',
      background: m.fg
    }
  }), labelMap[status] || 'Idle');
}
function PipelineList({
  rows,
  selectedId,
  onSelect,
  theme
}) {
  const t = theme;
  return /*#__PURE__*/React.createElement("div", {
    style: {
      background: t.surface,
      border: '1px solid ' + t.surfaceBorder,
      borderRadius: 9,
      overflow: 'hidden'
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'grid',
      gridTemplateColumns: '1.5fr 1fr 1fr 0.8fr 0.6fr',
      padding: '10px 18px',
      background: t.headerBg,
      borderBottom: '1px solid ' + t.surfaceBorder,
      fontFamily: 'var(--font-body)',
      fontSize: 10,
      fontWeight: 600,
      letterSpacing: '0.08em',
      textTransform: 'uppercase',
      color: t.headerFg
    }
  }, /*#__PURE__*/React.createElement("div", null, "Pipeline"), /*#__PURE__*/React.createElement("div", null, "Status"), /*#__PURE__*/React.createElement("div", null, "Last run"), /*#__PURE__*/React.createElement("div", null, "Owner"), /*#__PURE__*/React.createElement("div", {
    style: {
      textAlign: 'right'
    }
  }, "Runs")), rows.map(r => /*#__PURE__*/React.createElement("div", {
    key: r.id,
    onClick: () => onSelect(r.id),
    style: {
      display: 'grid',
      gridTemplateColumns: '1.5fr 1fr 1fr 0.8fr 0.6fr',
      padding: '14px 18px',
      borderBottom: '1px solid ' + t.rowBorder,
      cursor: 'pointer',
      background: selectedId === r.id ? t.rowSelected : 'transparent',
      alignItems: 'center'
    }
  }, /*#__PURE__*/React.createElement("div", null, /*#__PURE__*/React.createElement("div", {
    style: {
      fontFamily: 'var(--font-heading)',
      fontWeight: 600,
      fontSize: 13,
      color: t.title
    }
  }, r.name), /*#__PURE__*/React.createElement("div", {
    style: {
      fontFamily: 'var(--font-code)',
      fontSize: 11,
      color: t.text2,
      marginTop: 2
    }
  }, r.tag)), /*#__PURE__*/React.createElement("div", null, /*#__PURE__*/React.createElement(StatusPill, {
    status: r.status,
    theme: theme
  })), /*#__PURE__*/React.createElement("div", {
    style: {
      fontFamily: 'var(--font-body)',
      fontSize: 13,
      color: t.text1
    }
  }, r.lastRun), /*#__PURE__*/React.createElement("div", {
    style: {
      fontFamily: 'var(--font-body)',
      fontSize: 13,
      color: t.text2
    }
  }, r.owner), /*#__PURE__*/React.createElement("div", {
    style: {
      fontFamily: 'var(--font-code)',
      fontSize: 12,
      color: t.code,
      textAlign: 'right'
    }
  }, r.runs))));
}
Object.assign(__ds_scope, { StatusPill, PipelineList });
})(); } catch (e) { __ds_ns.__errors.push({ path: "ui_kits/agent-console/PipelineList.jsx", error: String((e && e.message) || e) }); }

// ui_kits/agent-console/RunDetail.jsx
try { (() => {
function RunDetail({
  pipeline,
  theme
}) {
  const t = theme;
  if (!pipeline) {
    return /*#__PURE__*/React.createElement("div", {
      style: {
        background: t.surface,
        border: '1px solid ' + t.surfaceBorder,
        borderRadius: 9,
        padding: 36,
        textAlign: 'center',
        color: t.text2,
        fontFamily: 'var(--font-body)',
        fontSize: 13
      }
    }, "Select a pipeline to view its latest run.");
  }
  const stages = [{
    name: 'Validate config',
    status: 'done',
    dur: '0.2s'
  }, {
    name: 'Policy check',
    status: 'done',
    dur: '0.4s'
  }, {
    name: 'Deploy to staging',
    status: 'done',
    dur: '0.6s'
  }, {
    name: 'Audit deploy',
    status: 'running',
    dur: '—'
  }];
  return /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'grid',
      gridTemplateColumns: '1fr',
      gap: 14
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      background: t.surface,
      border: '1px solid ' + t.surfaceBorder,
      borderRadius: 9,
      padding: '18px 22px'
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'space-between',
      marginBottom: 14
    }
  }, /*#__PURE__*/React.createElement("div", null, /*#__PURE__*/React.createElement("div", {
    style: {
      fontFamily: 'var(--font-heading)',
      fontWeight: 700,
      fontSize: 16,
      color: t.title
    }
  }, pipeline.name, " \xB7 run #", pipeline.runs), /*#__PURE__*/React.createElement("div", {
    style: {
      fontFamily: 'var(--font-code)',
      fontSize: 11,
      color: t.text2,
      marginTop: 3
    }
  }, "started ", pipeline.lastRun, " \xB7 by ", pipeline.owner)), /*#__PURE__*/React.createElement(__ds_scope.StatusPill, {
    status: pipeline.status,
    theme: theme
  })), /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      gap: 8
    }
  }, stages.map(s => /*#__PURE__*/React.createElement("div", {
    key: s.name,
    style: {
      flex: 1,
      padding: '10px 12px',
      borderRadius: 6,
      background: s.status === 'done' ? t.stageDone : s.status === 'running' ? t.stageRun : t.stageIdle,
      border: s.status === 'running' ? '1px solid ' + t.stageRunBorder : '1px solid transparent'
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      alignItems: 'center',
      gap: 6,
      minWidth: 0
    }
  }, /*#__PURE__*/React.createElement("span", {
    style: {
      width: 6,
      height: 6,
      borderRadius: '50%',
      background: s.status === 'done' ? t.stageDotDone : s.status === 'running' ? t.stageDotRun : t.stageDotIdle,
      flexShrink: 0
    }
  }), /*#__PURE__*/React.createElement("span", {
    style: {
      fontFamily: 'var(--font-heading)',
      fontWeight: 600,
      fontSize: 11,
      color: t.title,
      whiteSpace: 'nowrap',
      overflow: 'hidden',
      textOverflow: 'ellipsis'
    }
  }, s.name)), /*#__PURE__*/React.createElement("div", {
    style: {
      fontFamily: 'var(--font-code)',
      fontSize: 10,
      color: t.stageLabel || t.text2,
      marginTop: 4
    }
  }, s.dur))))), /*#__PURE__*/React.createElement("div", {
    style: {
      background: t.termBg,
      borderRadius: 9,
      padding: 18,
      fontFamily: 'var(--font-code)',
      fontSize: 12,
      lineHeight: 1.85,
      color: t.termText,
      border: '1px solid ' + (t.termBorder || (t.termBg === '#f8f9fb' ? t.surfaceBorder : 'transparent'))
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      color: t.termComment,
      marginBottom: 6
    }
  }, "// Live log \xB7 auditor agent"), /*#__PURE__*/React.createElement("div", null, /*#__PURE__*/React.createElement("span", {
    style: {
      color: t.termPrompt
    }
  }, "$"), " ", /*#__PURE__*/React.createElement("span", {
    style: {
      color: t.termCmd
    }
  }, "projectious run --pipeline ", pipeline.tag)), /*#__PURE__*/React.createElement("div", null, /*#__PURE__*/React.createElement("span", {
    style: {
      color: t.termOk
    }
  }, "\u2713"), " Config validated against schema v3.2"), /*#__PURE__*/React.createElement("div", null, /*#__PURE__*/React.createElement("span", {
    style: {
      color: t.termOk
    }
  }, "\u2713"), " Policy check: 12 rules passed"), /*#__PURE__*/React.createElement("div", null, /*#__PURE__*/React.createElement("span", {
    style: {
      color: t.termOk
    }
  }, "\u2713"), " Deploy to staging completed (1.2s)"), /*#__PURE__*/React.createElement("div", null, /*#__PURE__*/React.createElement("span", {
    style: {
      color: t.termRun
    }
  }, "\u25CF"), " Auditor agent reviewing deploy artifacts...")));
}
Object.assign(__ds_scope, { RunDetail });
})(); } catch (e) { __ds_ns.__errors.push({ path: "ui_kits/agent-console/RunDetail.jsx", error: String((e && e.message) || e) }); }

// ui_kits/agent-console/Sidebar.jsx
try { (() => {
function Sidebar({
  active = 'pipelines',
  onNav,
  theme
}) {
  const t = theme;
  const sections = [{
    group: 'Build',
    items: [{
      id: 'pipelines',
      label: 'Pipelines'
    }, {
      id: 'agents',
      label: 'Agents'
    }, {
      id: 'policies',
      label: 'Policies'
    }]
  }, {
    group: 'Observe',
    items: [{
      id: 'runs',
      label: 'Runs'
    }, {
      id: 'audit',
      label: 'Audit log'
    }, {
      id: 'metrics',
      label: 'Metrics'
    }]
  }, {
    group: 'Settings',
    items: [{
      id: 'team',
      label: 'Team'
    }, {
      id: 'connections',
      label: 'Connections'
    }]
  }];
  return /*#__PURE__*/React.createElement("aside", {
    style: {
      width: 240,
      background: t.sidebarBg,
      color: t.sidebarFg,
      display: 'flex',
      flexDirection: 'column',
      borderRight: '1px solid ' + t.sidebarBorder
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      padding: '18px 20px',
      borderBottom: '1px solid ' + t.sidebarBorder,
      display: 'flex',
      alignItems: 'center',
      gap: 6
    }
  }, /*#__PURE__*/React.createElement("img", {
    src: t.sidebarLogoText === '#fff' ? '../../assets/logo/icon-dark.svg' : '../../assets/logo/icon-light.svg',
    width: "28",
    height: "28",
    alt: ""
  }), /*#__PURE__*/React.createElement("span", {
    style: {
      fontFamily: 'var(--font-heading)',
      fontWeight: 700,
      fontSize: 16,
      lineHeight: 1,
      color: t.sidebarLogoText,
      letterSpacing: '-0.4px',
      display: 'inline-block',
      transform: 'translateY(-2px)'
    }
  }, "projectious"), /*#__PURE__*/React.createElement("span", {
    style: {
      width: 6,
      height: 6,
      borderRadius: '50%',
      background: t.sidebarLogoSep,
      display: 'inline-block',
      transform: 'translateY(0px)',
      margin: '0 -1px'
    }
  }), /*#__PURE__*/React.createElement("span", {
    style: {
      fontFamily: 'var(--font-heading)',
      fontWeight: 700,
      fontSize: 16,
      lineHeight: 1,
      color: '#E05232',
      letterSpacing: '-0.4px',
      display: 'inline-block',
      transform: 'translateY(-2px)'
    }
  }, "work")), /*#__PURE__*/React.createElement("nav", {
    style: {
      padding: '14px 12px',
      flex: 1,
      display: 'flex',
      flexDirection: 'column',
      gap: 18
    }
  }, sections.map(s => /*#__PURE__*/React.createElement("div", {
    key: s.group
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      fontFamily: 'var(--font-body)',
      fontSize: 10,
      fontWeight: 600,
      letterSpacing: '0.12em',
      textTransform: 'uppercase',
      color: t.sidebarSection,
      padding: '4px 10px',
      marginBottom: 4
    }
  }, s.group), s.items.map(it => /*#__PURE__*/React.createElement("a", {
    key: it.id,
    href: "#",
    onClick: e => {
      e.preventDefault();
      onNav && onNav(it.id);
    },
    style: {
      display: 'block',
      padding: '7px 10px',
      borderRadius: 6,
      fontFamily: 'var(--font-body)',
      fontSize: 13,
      color: active === it.id ? t.sidebarActive : t.sidebarFg,
      background: active === it.id ? t.sidebarActiveBg : 'transparent',
      textDecoration: 'none',
      borderLeft: active === it.id ? '2px solid #E05232' : '2px solid transparent',
      paddingLeft: active === it.id ? 8 : 10
    }
  }, it.label))))), /*#__PURE__*/React.createElement("div", {
    style: {
      padding: 14,
      borderTop: '1px solid ' + t.sidebarBorder,
      display: 'flex',
      alignItems: 'center',
      gap: 10
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      width: 30,
      height: 30,
      borderRadius: '50%',
      background: t.sidebarUserBg,
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'center',
      color: t.sidebarUserFg,
      fontFamily: 'var(--font-heading)',
      fontWeight: 600,
      fontSize: 12
    }
  }, "JS"), /*#__PURE__*/React.createElement("div", {
    style: {
      minWidth: 0
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      fontFamily: 'var(--font-body)',
      fontSize: 12,
      color: t.sidebarUserName,
      fontWeight: 500,
      whiteSpace: 'nowrap'
    }
  }, "Jan Schmidt"), /*#__PURE__*/React.createElement("div", {
    style: {
      fontFamily: 'var(--font-body)',
      fontSize: 11,
      color: t.sidebarMuted,
      whiteSpace: 'nowrap'
    }
  }, "Platform Eng"))));
}
Object.assign(__ds_scope, { Sidebar });
})(); } catch (e) { __ds_ns.__errors.push({ path: "ui_kits/agent-console/Sidebar.jsx", error: String((e && e.message) || e) }); }

// ui_kits/agent-console/StatusBar.jsx
try { (() => {
function StatusBar({
  theme
}) {
  const t = theme;
  return /*#__PURE__*/React.createElement("div", {
    style: {
      padding: '8px 28px',
      background: t.statusBarBg,
      color: t.statusBarFg,
      fontFamily: 'var(--font-code)',
      fontSize: 11,
      whiteSpace: 'nowrap',
      display: 'flex',
      alignItems: 'center',
      gap: 16,
      flexShrink: 0,
      borderTop: '1px solid ' + t.statusBarBorder
    }
  }, /*#__PURE__*/React.createElement("span", {
    style: {
      color: t.statusBarOk
    }
  }, "\u25CF API healthy"), /*#__PURE__*/React.createElement("span", null, "\xB7"), /*#__PURE__*/React.createElement("span", null, "4 agents active"), /*#__PURE__*/React.createElement("span", null, "\xB7"), /*#__PURE__*/React.createElement("span", null, "policy v3.2"), /*#__PURE__*/React.createElement("span", {
    style: {
      marginLeft: 'auto',
      color: t.statusBarMuted
    }
  }, "v2.1.0 \xB7 synced just now"));
}
Object.assign(__ds_scope, { StatusBar });
})(); } catch (e) { __ds_ns.__errors.push({ path: "ui_kits/agent-console/StatusBar.jsx", error: String((e && e.message) || e) }); }

// ui_kits/agent-console/Topbar.jsx
try { (() => {
// Tabler `sun` / `moon` (MIT) inlined — 24px grid, round caps.
// Tabler ships at stroke-width 2; the brand rule is 1.5, which the set supports.
function ModeIcon({
  mode,
  color
}) {
  const common = {
    width: 20,
    height: 20,
    viewBox: '0 0 24 24',
    fill: 'none',
    stroke: color,
    strokeWidth: 1.5,
    strokeLinecap: 'round',
    strokeLinejoin: 'round',
    'aria-hidden': 'true',
    focusable: 'false'
  };
  if (mode === 'dark' || mode === 'navy') {
    return /*#__PURE__*/React.createElement("svg", common, /*#__PURE__*/React.createElement("path", {
      d: "M8 12a4 4 0 1 0 8 0a4 4 0 1 0 -8 0"
    }), /*#__PURE__*/React.createElement("path", {
      d: "M3 12h1m8 -9v1m8 8h1m-9 8v1m-6.4 -15.4l.7 .7m12.1 -.7l-.7 .7m0 11.4l.7 .7m-12.1 -.7l-.7 .7"
    }));
  }
  return /*#__PURE__*/React.createElement("svg", common, /*#__PURE__*/React.createElement("path", {
    d: "M12 3c.132 0 .263 0 .393 0a7.5 7.5 0 0 0 7.92 12.446a9 9 0 1 1 -8.313 -12.454l0 .008"
  }));
}
function Topbar({
  title,
  theme,
  mode,
  onToggleMode
}) {
  const t = theme;
  // Three appearances, cycled in order: light → deep dark → navy dark.
  const nextLabel = mode === 'light' ? 'deep dark' : mode === 'dark' ? 'navy dark' : 'light';
  return /*#__PURE__*/React.createElement("header", {
    style: {
      padding: '14px 28px',
      borderBottom: '1px solid ' + t.topbarBorder,
      background: t.topbarBg,
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'space-between',
      gap: 16
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      minWidth: 0
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      fontFamily: 'var(--font-body)',
      fontSize: 11,
      color: t.topbarMuted,
      fontWeight: 500,
      whiteSpace: 'nowrap'
    }
  }, "Workspace \xB7 acme-corp"), /*#__PURE__*/React.createElement("h1", {
    style: {
      fontFamily: 'var(--font-heading)',
      fontWeight: 700,
      fontSize: 22,
      letterSpacing: '-0.3px',
      color: t.topbarTitle,
      margin: 0
    }
  }, title)), /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      alignItems: 'center',
      gap: 10
    }
  }, /*#__PURE__*/React.createElement("input", {
    placeholder: "Search pipelines, agents, runs\u2026",
    style: {
      fontFamily: 'var(--font-body)',
      fontSize: 13,
      padding: '8px 14px',
      width: 280,
      border: '1.5px solid ' + t.inputBorder,
      borderRadius: 6,
      background: t.inputBg,
      color: t.inputFg,
      outline: 0
    }
  }), /*#__PURE__*/React.createElement("button", {
    onClick: onToggleMode,
    "aria-label": 'Switch to ' + nextLabel + ' mode',
    title: 'Switch to ' + nextLabel + ' mode',
    style: {
      width: 36,
      height: 36,
      border: '1.5px solid ' + t.inputBorder,
      borderRadius: 6,
      background: t.inputBg,
      cursor: 'pointer',
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'center',
      padding: 0,
      flexShrink: 0
    }
  }, /*#__PURE__*/React.createElement(ModeIcon, {
    mode: mode,
    color: t.icon || t.text2
  })), /*#__PURE__*/React.createElement("button", {
    style: {
      fontFamily: 'var(--font-heading)',
      fontWeight: 500,
      fontSize: 13,
      padding: '8px 16px',
      border: 0,
      borderRadius: 6,
      whiteSpace: 'nowrap',
      flexShrink: 0,
      color: '#fff',
      background: '#cc4528',
      cursor: 'pointer'
    }
  }, "+ New pipeline")));
}
Object.assign(__ds_scope, { Topbar });
})(); } catch (e) { __ds_ns.__errors.push({ path: "ui_kits/agent-console/Topbar.jsx", error: String((e && e.message) || e) }); }

// ui_kits/agent-console/theme.jsx
try { (() => {
// Theme tokens — light + dark variants for the agent console.
// Light = soft warm-grey sidebar, white cards, light terminal, dark statusbar.

const LIGHT_TERMINAL = {
  termBg: '#f8f9fb',
  termComment: '#5c6f82',
  termPrompt: '#2f7d65',
  termCmd: '#1d3352',
  termText: '#142438',
  termOk: '#2f7d65',
  termRun: '#c04424'
};
const LIGHT_PILLS = {
  pill: {
    healthy: {
      bg: '#d1ebe0',
      fg: '#276754'
    },
    running: {
      bg: '#dae2ec',
      fg: '#1d3352'
    },
    warning: {
      bg: '#f5ecd0',
      fg: '#6f5106'
    },
    failed: {
      bg: '#fce8e8',
      fg: '#a8261c'
    },
    idle: {
      bg: '#f1f2f4',
      fg: '#5c6f82'
    }
  }
};
const THEMES = {
  light: {
    appBg: '#f8f9fb',
    sidebarBg: '#f0f3f8',
    sidebarBorder: '#cdd0d5',
    sidebarFg: '#3a4a5c',
    sidebarMuted: '#5c6f82',
    sidebarSection: '#5c6f82',
    sidebarActive: '#1d3352',
    sidebarActiveBg: 'rgba(224,82,50,0.10)',
    sidebarLogoText: '#1d3352',
    sidebarLogoSep: '#7490b2',
    sidebarUserBg: '#cdd0d5',
    sidebarUserFg: '#1d3352',
    sidebarUserName: '#1d3352',
    topbarBg: '#fff',
    topbarBorder: '#cdd0d5',
    topbarMuted: '#5c6f82',
    topbarTitle: '#1d3352',
    inputBg: '#f8f9fb',
    inputBorder: '#cdd0d5',
    inputFg: '#142438',
    surface: '#fff',
    surfaceBorder: '#cdd0d5',
    rowBorder: '#e6e8eb',
    rowSelected: '#fef0ea',
    headerBg: '#f8f9fb',
    headerFg: '#5c6f82',
    text1: '#142438',
    text2: '#5c6f82',
    icon: '#5c6f82',
    title: '#1d3352',
    code: '#1d3352',
    stageDone: '#d1ebe0',
    stageRun: '#dae2ec',
    stageIdle: '#f1f2f4',
    stageRunBorder: '#1d3352',
    stageDotDone: '#2f7d65',
    stageDotRun: '#1d3352',
    stageDotIdle: '#9299a4',
    stageLabel: '#1e2b38',
    ...LIGHT_TERMINAL,
    statusBarBg: '#f0f3f8',
    statusBarFg: '#5c6f82',
    statusBarMuted: '#5c6f82',
    statusBarBorder: '#cdd0d5',
    statusBarOk: '#276754',
    ...LIGHT_PILLS
  },
  dark: {
    appBg: '#0e1720',
    sidebarBg: '#131e2b',
    sidebarBorder: 'rgba(255,255,255,0.06)',
    sidebarFg: '#c5daf0',
    sidebarMuted: '#97a8b8',
    sidebarSection: '#97a8b8',
    sidebarActive: '#fff',
    sidebarActiveBg: 'rgba(224,82,50,0.18)',
    sidebarLogoText: '#fff',
    sidebarLogoSep: '#7490b2',
    sidebarUserBg: '#2b4d78',
    sidebarUserFg: '#fff',
    sidebarUserName: '#fff',
    topbarBg: '#131e2b',
    topbarBorder: 'rgba(255,255,255,0.06)',
    topbarMuted: '#97a8b8',
    topbarTitle: '#c5daf0',
    inputBg: '#1a2b3e',
    inputBorder: '#2e4b68',
    inputFg: '#c5daf0',
    surface: '#131e2b',
    surfaceBorder: 'rgba(255,255,255,0.10)',
    rowBorder: 'rgba(255,255,255,0.05)',
    rowSelected: 'rgba(224,82,50,0.10)',
    headerBg: '#1a2b3e',
    headerFg: '#97a8b8',
    text1: '#c5daf0',
    text2: '#97a8b8',
    icon: '#97a8b8',
    title: '#c5daf0',
    code: '#c5daf0',
    stageDone: 'rgba(108,192,144,0.15)',
    stageRun: 'rgba(122,148,176,0.18)',
    stageIdle: 'rgba(255,255,255,0.04)',
    stageRunBorder: '#7494b0',
    stageDotDone: '#6cc090',
    stageDotRun: '#7494b0',
    stageDotIdle: '#5a6a78',
    stageLabel: '#c5daf0',
    statusBarBg: '#131e2b',
    statusBarFg: '#97a8b8',
    statusBarMuted: '#97a8b8',
    statusBarBorder: 'rgba(255,255,255,0.06)',
    statusBarOk: '#6cc090',
    termBg: '#0e1720',
    termBorder: 'rgba(108,192,144,0.18)',
    termComment: '#7e8f9e',
    termPrompt: '#6cc090',
    termCmd: '#c5daf0',
    termText: '#9aaab8',
    termOk: '#6cc090',
    termRun: '#ea7558',
    pill: {
      healthy: {
        bg: 'rgba(108,192,144,0.16)',
        fg: '#6cc090'
      },
      running: {
        bg: 'rgba(122,148,176,0.18)',
        fg: '#8aacc8'
      },
      warning: {
        bg: 'rgba(230,185,74,0.16)',
        fg: '#e0a92a'
      },
      failed: {
        bg: 'rgba(234,117,88,0.16)',
        fg: '#f09878'
      },
      idle: {
        bg: 'rgba(255,255,255,0.05)',
        fg: '#97a8b8'
      }
    }
  }
};

// Navy dark — a derivative of deep dark, exactly as in colors_and_type.css:
// only the surfaces, borders and input/header steps move onto the navy ramp.
THEMES.navy = Object.assign({}, THEMES.dark, {
  appBg: '#132440',
  sidebarBg: '#1a2b3e',
  topbarBg: '#1a2b3e',
  surface: '#1a2b3e',
  inputBg: '#20354d',
  inputBorder: '#2e4b68',
  headerBg: '#20354d',
  statusBarBg: '#1a2b3e',
  termBg: '#0e1720'
});
Object.assign(__ds_scope, { THEMES });
})(); } catch (e) { __ds_ns.__errors.push({ path: "ui_kits/agent-console/theme.jsx", error: String((e && e.message) || e) }); }

// ui_kits/marketing-site/CTA.jsx
try { (() => {
function CTA() {
  return /*#__PURE__*/React.createElement("section", {
    style: {
      background: 'var(--fg-1)',
      padding: '80px 32px',
      position: 'relative',
      overflow: 'hidden'
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      position: 'absolute',
      top: -160,
      right: -160,
      width: 460,
      height: 460,
      background: 'radial-gradient(circle, rgba(224,82,50,0.10) 0%, rgba(224,82,50,0) 65%)',
      pointerEvents: 'none'
    }
  }), /*#__PURE__*/React.createElement("div", {
    style: {
      maxWidth: 1100,
      margin: '0 auto',
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'space-between',
      gap: 32,
      position: 'relative'
    }
  }, /*#__PURE__*/React.createElement("div", null, /*#__PURE__*/React.createElement("h2", {
    style: {
      fontFamily: 'var(--font-heading)',
      fontWeight: 700,
      fontSize: 36,
      color: 'var(--surface)',
      margin: 0,
      letterSpacing: '-0.4px'
    }
  }, "Ready to redesign how you work?"), /*#__PURE__*/React.createElement("p", {
    style: {
      fontFamily: 'var(--font-body)',
      fontSize: 16,
      color: '#97a8b8',
      margin: '8px 0 0'
    }
  }, "30-minute introduction. No deck.")), /*#__PURE__*/React.createElement("button", {
    style: {
      fontFamily: 'var(--font-heading)',
      fontWeight: 500,
      fontSize: 15,
      padding: '14px 28px',
      border: 0,
      borderRadius: 6,
      color: 'var(--surface)',
      background: 'var(--color-accent-solid)',
      cursor: 'pointer',
      flexShrink: 0
    }
  }, "Book an intro")));
}
Object.assign(__ds_scope, { CTA });
})(); } catch (e) { __ds_ns.__errors.push({ path: "ui_kits/marketing-site/CTA.jsx", error: String((e && e.message) || e) }); }

// ui_kits/marketing-site/CodeShowcase.jsx
try { (() => {
function CodeShowcase() {
  return /*#__PURE__*/React.createElement("section", {
    style: {
      padding: '96px 32px',
      background: 'var(--surface)'
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      maxWidth: 1100,
      margin: '0 auto',
      display: 'grid',
      gridTemplateColumns: '1fr 1fr',
      gap: 64,
      alignItems: 'center'
    }
  }, /*#__PURE__*/React.createElement("div", null, /*#__PURE__*/React.createElement("div", {
    style: {
      fontFamily: 'var(--font-body)',
      fontSize: 12,
      fontWeight: 600,
      letterSpacing: '0.12em',
      textTransform: 'uppercase',
      color: 'var(--orange-11)',
      marginBottom: 16
    }
  }, "What we ship"), /*#__PURE__*/React.createElement("h2", {
    style: {
      fontFamily: 'var(--font-heading)',
      fontWeight: 700,
      fontSize: 36,
      lineHeight: 1.2,
      letterSpacing: '-0.4px',
      color: 'var(--fg-1)',
      margin: 0
    }
  }, "Pipelines you can reason about."), /*#__PURE__*/React.createElement("p", {
    style: {
      fontFamily: 'var(--font-body)',
      fontSize: 15,
      lineHeight: 1.6,
      color: 'var(--fg-3)',
      marginTop: 16
    }
  }, "Declarative configs. Auditable runs. Policies that fail closed. Whether the operator is human or agent, the pipeline behaves the same way."), /*#__PURE__*/React.createElement("a", {
    href: "#",
    style: {
      display: 'inline-flex',
      alignItems: 'center',
      gap: 6,
      marginTop: 24,
      fontFamily: 'var(--font-heading)',
      fontSize: 14,
      fontWeight: 500,
      color: 'var(--orange-11)',
      textDecoration: 'none'
    }
  }, "View the docs ", /*#__PURE__*/React.createElement("span", {
    style: {
      fontSize: 16
    }
  }, "\u2192"))), /*#__PURE__*/React.createElement("div", {
    style: {
      background: '#0e1720',
      borderRadius: 9,
      padding: 22,
      fontFamily: 'var(--font-code)',
      fontSize: 13,
      lineHeight: 1.85,
      color: '#c5daf0',
      boxShadow: '0 8px 24px rgba(0,0,0,0.12)'
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      gap: 6,
      marginBottom: 14
    }
  }, /*#__PURE__*/React.createElement("span", {
    style: {
      width: 10,
      height: 10,
      borderRadius: '50%',
      background: '#3a454f'
    }
  }), /*#__PURE__*/React.createElement("span", {
    style: {
      width: 10,
      height: 10,
      borderRadius: '50%',
      background: '#3a454f'
    }
  }), /*#__PURE__*/React.createElement("span", {
    style: {
      width: 10,
      height: 10,
      borderRadius: '50%',
      background: '#3a454f'
    }
  })), /*#__PURE__*/React.createElement("div", null, /*#__PURE__*/React.createElement("span", {
    style: {
      color: '#7e8f9e'
    }
  }, `// Agent-validated deploy`)), /*#__PURE__*/React.createElement("div", null, /*#__PURE__*/React.createElement("span", {
    style: {
      color: '#8aacc8'
    }
  }, "const"), " pipeline = createPipeline(", '{'), /*#__PURE__*/React.createElement("div", null, "\xA0\xA0name: ", /*#__PURE__*/React.createElement("span", {
    style: {
      color: '#ea7558'
    }
  }, "\"validate-deploy\""), ","), /*#__PURE__*/React.createElement("div", null, "\xA0\xA0policy: ", /*#__PURE__*/React.createElement("span", {
    style: {
      color: '#ea7558'
    }
  }, "\"strict\""), ","), /*#__PURE__*/React.createElement("div", null, "\xA0\xA0agents: [", /*#__PURE__*/React.createElement("span", {
    style: {
      color: '#ea7558'
    }
  }, "\"auditor\""), ", ", /*#__PURE__*/React.createElement("span", {
    style: {
      color: '#ea7558'
    }
  }, "\"deployer\""), "]"), /*#__PURE__*/React.createElement("div", null, '}', ");"), /*#__PURE__*/React.createElement("div", null, "\xA0"), /*#__PURE__*/React.createElement("div", null, /*#__PURE__*/React.createElement("span", {
    style: {
      color: '#6cc090'
    }
  }, "\u2713"), " 12 checks passed \xB7 1.2s"))));
}
Object.assign(__ds_scope, { CodeShowcase });
})(); } catch (e) { __ds_ns.__errors.push({ path: "ui_kits/marketing-site/CodeShowcase.jsx", error: String((e && e.message) || e) }); }

// ui_kits/marketing-site/Convictions.jsx
try { (() => {
function Convictions() {
  const items = [{
    n: '01',
    t: 'Do more with more',
    b: 'Augment people\u2019s strengths so the same or more people produce more, higher quality, for broader markets.'
  }, {
    n: '02',
    t: 'Specialized beats generic now',
    b: 'Agents collapse the cost of bespoke software. Lot-of-one systems \u2014 built for one team\u2019s exact workflow \u2014 are now economically rational. Underserved companies stop bending themselves to fit generic SaaS or Excel.'
  }, {
    n: '03',
    t: 'Provider independence',
    b: 'Agnostic, self-hosted where it matters. No single AI vendor becomes a single point of failure.'
  }, {
    n: '04',
    t: 'We run what we recommend',
    b: 'Same convictions, same software, applied to ourselves first.'
  }];
  return /*#__PURE__*/React.createElement("section", {
    style: {
      padding: '96px 32px',
      background: 'var(--bg)'
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      maxWidth: 1100,
      margin: '0 auto'
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      fontFamily: 'var(--font-body)',
      fontSize: 12,
      fontWeight: 600,
      letterSpacing: '0.12em',
      textTransform: 'uppercase',
      color: 'var(--orange-11)',
      marginBottom: 16
    }
  }, "Convictions"), /*#__PURE__*/React.createElement("h2", {
    style: {
      fontFamily: 'var(--font-heading)',
      fontWeight: 700,
      fontSize: 40,
      lineHeight: 1.15,
      letterSpacing: '-0.6px',
      color: 'var(--fg-1)',
      margin: 0,
      maxWidth: 720
    }
  }, "What we believe, on the record."), /*#__PURE__*/React.createElement("div", {
    style: {
      marginTop: 40
    }
  }, items.map(it => /*#__PURE__*/React.createElement("div", {
    key: it.n,
    style: {
      display: 'grid',
      gridTemplateColumns: '90px 1fr',
      gap: 32,
      padding: '24px 0',
      borderTop: '1px solid #cdd0d5'
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      fontFamily: 'var(--font-code)',
      fontSize: 14,
      color: 'var(--orange-11)',
      fontWeight: 500
    }
  }, it.n), /*#__PURE__*/React.createElement("div", null, /*#__PURE__*/React.createElement("div", {
    style: {
      fontFamily: 'var(--font-heading)',
      fontWeight: 700,
      fontSize: 22,
      color: 'var(--fg-1)',
      letterSpacing: '-0.2px'
    }
  }, it.t), /*#__PURE__*/React.createElement("div", {
    style: {
      fontFamily: 'var(--font-body)',
      fontSize: 15,
      lineHeight: 1.6,
      color: 'var(--fg-3)',
      marginTop: 6,
      maxWidth: 760
    }
  }, it.b)))))));
}
Object.assign(__ds_scope, { Convictions });
})(); } catch (e) { __ds_ns.__errors.push({ path: "ui_kits/marketing-site/Convictions.jsx", error: String((e && e.message) || e) }); }

// ui_kits/marketing-site/Footer.jsx
try { (() => {
function Footer() {
  return /*#__PURE__*/React.createElement("footer", {
    style: {
      padding: '36px 32px',
      background: 'var(--bg)',
      borderTop: '1px solid #cdd0d5'
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      maxWidth: 1100,
      margin: '0 auto',
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'space-between'
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      alignItems: 'center',
      gap: 10
    }
  }, /*#__PURE__*/React.createElement("img", {
    src: "../../assets/logo/icon-light.svg",
    width: "22",
    height: "22",
    alt: ""
  }), /*#__PURE__*/React.createElement("span", {
    style: {
      fontFamily: 'var(--font-heading)',
      fontWeight: 700,
      fontSize: 14,
      color: 'var(--fg-1)'
    }
  }, "projectious", /*#__PURE__*/React.createElement("span", {
    style: {
      color: '#7490b2'
    }
  }, "\xB7"), /*#__PURE__*/React.createElement("span", {
    style: {
      color: '#E05232'
    }
  }, "work"))), /*#__PURE__*/React.createElement("div", {
    style: {
      fontFamily: 'var(--font-body)',
      fontSize: 12,
      color: 'var(--fg-3)'
    }
  }, "Cloud \xB7 Agile \xB7 Agentic AI \xA0\xB7\xA0 \xA9 2026")));
}
Object.assign(__ds_scope, { Footer });
})(); } catch (e) { __ds_ns.__errors.push({ path: "ui_kits/marketing-site/Footer.jsx", error: String((e && e.message) || e) }); }

// ui_kits/marketing-site/Header.jsx
try { (() => {
function Header() {
  const fg = 'var(--fg-1)';
  const sub = 'var(--fg-2)';
  const border = 'var(--border)';
  const bg = 'var(--header-bg)';
  return /*#__PURE__*/React.createElement("header", {
    style: {
      position: 'sticky',
      top: 0,
      zIndex: 10,
      background: bg,
      backdropFilter: 'blur(8px)',
      borderBottom: `1px solid ${border}`,
      padding: '14px 32px',
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'space-between'
    }
  }, /*#__PURE__*/React.createElement("a", {
    href: "#",
    style: {
      display: 'flex',
      alignItems: 'center',
      gap: 10,
      textDecoration: 'none'
    }
  }, /*#__PURE__*/React.createElement("img", {
    src: "../../assets/logo/icon-light.svg",
    width: "28",
    height: "28",
    style: {
      display: dark ? 'none' : 'block'
    },
    alt: ""
  }), /*#__PURE__*/React.createElement("img", {
    src: "../../assets/logo/icon-dark.svg",
    width: "28",
    height: "28",
    style: {
      display: dark ? 'block' : 'none'
    },
    alt: ""
  }), /*#__PURE__*/React.createElement("span", {
    style: {
      fontFamily: 'var(--font-heading)',
      fontWeight: 700,
      fontSize: 17,
      color: fg,
      letterSpacing: '-0.3px'
    }
  }, "projectious", /*#__PURE__*/React.createElement("span", {
    style: {
      color: 'var(--midnight-8)'
    }
  }, "\xB7"), /*#__PURE__*/React.createElement("span", {
    style: {
      color: 'var(--logo-accent)'
    }
  }, "work"))), /*#__PURE__*/React.createElement("nav", {
    style: {
      display: 'flex',
      alignItems: 'center',
      gap: 28
    }
  }, ['Practice areas', 'How we work', 'Cases', 'Writing'].map(label => /*#__PURE__*/React.createElement("a", {
    key: label,
    href: "#",
    style: {
      fontFamily: 'var(--font-body)',
      fontSize: 14,
      color: sub,
      textDecoration: 'none',
      whiteSpace: 'nowrap'
    }
  }, label)), /*#__PURE__*/React.createElement("button", {
    style: {
      fontFamily: 'var(--font-heading)',
      fontWeight: 500,
      fontSize: 13,
      padding: '8px 18px',
      border: 0,
      borderRadius: 6,
      whiteSpace: 'nowrap',
      color: 'var(--fixed-control-text)',
      background: 'var(--color-accent-solid)',
      cursor: 'pointer'
    }
  }, "Start a project")));
}
Object.assign(__ds_scope, { Header });
})(); } catch (e) { __ds_ns.__errors.push({ path: "ui_kits/marketing-site/Header.jsx", error: String((e && e.message) || e) }); }

// ui_kits/marketing-site/Hero.jsx
try { (() => {
function Hero() {
  return /*#__PURE__*/React.createElement("section", {
    style: {
      background: 'var(--hero-bg)',
      color: 'var(--hero-fg)',
      padding: '110px 32px 100px',
      position: 'relative',
      overflow: 'hidden'
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      position: 'absolute',
      top: -200,
      right: -200,
      width: 560,
      height: 560,
      background: 'radial-gradient(circle, rgba(224,82,50,0.12) 0%, rgba(224,82,50,0) 65%)',
      pointerEvents: 'none'
    }
  }), /*#__PURE__*/React.createElement("div", {
    style: {
      maxWidth: 1100,
      margin: '0 auto',
      position: 'relative'
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      fontFamily: 'var(--font-body)',
      fontSize: 12,
      fontWeight: 600,
      letterSpacing: '0.12em',
      textTransform: 'uppercase',
      color: 'var(--hero-eyebrow)',
      marginBottom: 24
    }
  }, "Cloud \xB7 Agile \xB7 Agentic AI"), /*#__PURE__*/React.createElement("h1", {
    style: {
      fontFamily: 'var(--font-heading)',
      fontWeight: 800,
      fontSize: 72,
      lineHeight: 1.05,
      letterSpacing: '-1.5px',
      color: 'var(--hero-title)',
      margin: 0,
      maxWidth: 820
    }
  }, "Redesigning work."), /*#__PURE__*/React.createElement("p", {
    style: {
      fontFamily: 'var(--font-body)',
      fontSize: 19,
      lineHeight: 1.6,
      color: 'var(--hero-body)',
      marginTop: 28,
      maxWidth: 620
    }
  }, "Agent-first consulting for organizations adopting AI-native workflows. We run what we recommend \u2014 same convictions, same software, applied to your stack."), /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      gap: 12,
      marginTop: 36
    }
  }, /*#__PURE__*/React.createElement("button", {
    style: {
      fontFamily: 'var(--font-heading)',
      fontWeight: 500,
      fontSize: 15,
      padding: '13px 28px',
      border: 0,
      borderRadius: 6,
      color: 'var(--fixed-control-text)',
      background: 'var(--color-accent-solid)',
      cursor: 'pointer'
    }
  }, "Start a project"), /*#__PURE__*/React.createElement("button", {
    style: {
      fontFamily: 'var(--font-heading)',
      fontWeight: 500,
      fontSize: 15,
      padding: '13px 28px',
      borderRadius: 6,
      background: 'transparent',
      color: 'var(--hero-fg)',
      border: '1.5px solid rgba(255,255,255,0.18)',
      cursor: 'pointer'
    }
  }, "How we work"))));
}
Object.assign(__ds_scope, { Hero });
})(); } catch (e) { __ds_ns.__errors.push({ path: "ui_kits/marketing-site/Hero.jsx", error: String((e && e.message) || e) }); }

// ui_kits/marketing-site/Pillars.jsx
try { (() => {
function Pillars() {
  const items = [{
    title: 'Cloud',
    body: 'Composable, provider-independent infrastructure. We design platforms that survive vendor decisions.'
  }, {
    title: 'Agile',
    body: 'Lean delivery with continuous policy and audit baked in — quality is not a phase, it is a posture.'
  }, {
    title: 'Agentic AI',
    body: 'Agents in the loop, not on the side. We integrate them into the work, not the demo.'
  }];
  return /*#__PURE__*/React.createElement("section", {
    style: {
      padding: '96px 32px',
      background: 'var(--bg)'
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      maxWidth: 1100,
      margin: '0 auto'
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      fontFamily: 'var(--font-body)',
      fontSize: 12,
      fontWeight: 600,
      letterSpacing: '0.12em',
      textTransform: 'uppercase',
      color: 'var(--orange-11)',
      marginBottom: 16
    }
  }, "Practice areas"), /*#__PURE__*/React.createElement("h2", {
    style: {
      fontFamily: 'var(--font-heading)',
      fontWeight: 700,
      fontSize: 40,
      lineHeight: 1.15,
      letterSpacing: '-0.6px',
      color: 'var(--fg-1)',
      margin: 0,
      maxWidth: 720
    }
  }, "Three practices. One discipline."), /*#__PURE__*/React.createElement("p", {
    style: {
      fontFamily: 'var(--font-body)',
      fontSize: 16,
      lineHeight: 1.6,
      color: 'var(--fg-3)',
      marginTop: 16,
      maxWidth: 620
    }
  }, "We deliver as a single team. The labels exist for clarity, not for org charts."), /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'grid',
      gridTemplateColumns: 'repeat(3, 1fr)',
      gap: 16,
      marginTop: 48
    }
  }, items.map(it => /*#__PURE__*/React.createElement("div", {
    key: it.title,
    style: {
      background: 'var(--surface)',
      border: '1px solid #cdd0d5',
      borderRadius: 9,
      padding: '28px 26px'
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      width: 36,
      height: 36,
      borderRadius: 6,
      background: '#dae2ec',
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'center',
      fontFamily: 'var(--font-heading)',
      fontWeight: 700,
      color: 'var(--fg-1)',
      marginBottom: 18
    }
  }, it.title[0]), /*#__PURE__*/React.createElement("div", {
    style: {
      fontFamily: 'var(--font-heading)',
      fontWeight: 700,
      fontSize: 20,
      color: 'var(--fg-1)',
      marginBottom: 8
    }
  }, it.title), /*#__PURE__*/React.createElement("div", {
    style: {
      fontFamily: 'var(--font-body)',
      fontSize: 14,
      lineHeight: 1.6,
      color: 'var(--fg-3)'
    }
  }, it.body))))));
}
Object.assign(__ds_scope, { Pillars });
})(); } catch (e) { __ds_ns.__errors.push({ path: "ui_kits/marketing-site/Pillars.jsx", error: String((e && e.message) || e) }); }

__ds_ns.StatusPill = __ds_scope.StatusPill;

__ds_ns.PipelineList = __ds_scope.PipelineList;

__ds_ns.RunDetail = __ds_scope.RunDetail;

__ds_ns.Sidebar = __ds_scope.Sidebar;

__ds_ns.StatusBar = __ds_scope.StatusBar;

__ds_ns.Topbar = __ds_scope.Topbar;

__ds_ns.THEMES = __ds_scope.THEMES;

__ds_ns.CTA = __ds_scope.CTA;

__ds_ns.CodeShowcase = __ds_scope.CodeShowcase;

__ds_ns.Convictions = __ds_scope.Convictions;

__ds_ns.Footer = __ds_scope.Footer;

__ds_ns.Header = __ds_scope.Header;

__ds_ns.Hero = __ds_scope.Hero;

__ds_ns.Pillars = __ds_scope.Pillars;

})();

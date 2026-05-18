import React, {useState, useMemo} from 'react';
import Layout from '@theme/Layout';
import indexJson from '../../static/asciinema/themes/index.json';

// The full sweep of all 61 themes is generated under
// /static/asciinema/themes/<slug>.cast by scripts/test-screencasts.sh.
const THEME_INDEX = indexJson;

function ThemeCard({theme, baseUrl}) {
  const castSrc = `${baseUrl}asciinema/themes/${theme.slug}.cast`;
  const hasCast = theme.cast != null;

  // Lazy-init asciinema player on mount via the global AsciinemaPlayer loaded
  // in docusaurus.config.js scripts.
  const ref = React.useRef(null);
  React.useEffect(() => {
    if (!hasCast) return;
    const el = ref.current;
    if (!el) return;
    // Only initialise once (guard repeated effect calls during dev HMR)
    if (el.dataset.initialized === 'true') return;
    const tryInit = () => {
      if (typeof AsciinemaPlayer === 'undefined') {
        // Player script not yet ready; retry after a short delay
        setTimeout(tryInit, 200);
        return;
      }
      AsciinemaPlayer.create(castSrc, el, {
        poster: 'npt:0:5',
        autoPlay: false,
        loop: false,
        fit: 'width',
        terminalFontSize: 'small',
        cols: 160,
        rows: 45,
      });
      el.dataset.initialized = 'true';
    };
    tryInit();
  }, [castSrc, hasCast]);

  const modeLabel = theme.mode.charAt(0).toUpperCase() + theme.mode.slice(1);
  const metaLine = [
    theme.family,
    modeLabel,
    theme.variant ? `${theme.variant}` : null,
  ].filter(Boolean).join(' / ');

  return (
    <div
      style={{
        border: '1px solid var(--ifm-color-emphasis-300)',
        borderRadius: 'var(--ifm-border-radius)',
        overflow: 'hidden',
        display: 'flex',
        flexDirection: 'column',
        background: 'var(--ifm-card-background-color)',
      }}
    >
      {/* Card header */}
      <div
        style={{
          padding: '10px 14px 8px',
          borderBottom: '1px solid var(--ifm-color-emphasis-200)',
        }}
      >
        <div style={{fontWeight: 600, fontSize: '0.95rem', fontFamily: 'var(--ifm-font-family-monospace)'}}>
          {theme.slug}
        </div>
        <div style={{fontSize: '0.78rem', color: 'var(--ifm-color-emphasis-600)', marginTop: 2}}>
          {metaLine}
        </div>
      </div>

      {/* Player area */}
      <div style={{flex: 1, background: '#111', minHeight: 100}}>
        {hasCast ? (
          <div ref={ref} style={{width: '100%'}} />
        ) : (
          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              height: 100,
              color: '#888',
              fontSize: '0.8rem',
            }}
          >
            cast not yet recorded
          </div>
        )}
      </div>
    </div>
  );
}

export default function ThemesPage() {
  const themes = THEME_INDEX.themes || [];
  const totalCount = themes.length;

  // Collect unique families and modes for the filter bar
  const families = useMemo(
    () => [...new Set(themes.map((t) => t.family))].sort(),
    [themes]
  );
  const modes = useMemo(
    () => [...new Set(themes.map((t) => t.mode))].sort(),
    [themes]
  );

  const [selectedFamily, setSelectedFamily] = useState('all');
  const [selectedMode, setSelectedMode] = useState('all');

  const filtered = useMemo(
    () =>
      themes.filter((t) => {
        if (selectedFamily !== 'all' && t.family !== selectedFamily) return false;
        if (selectedMode !== 'all' && t.mode !== selectedMode) return false;
        return true;
      }),
    [themes, selectedFamily, selectedMode]
  );

  // Docusaurus base URL (needed to build static asset paths)
  // We use a relative path that works from any page depth because the player
  // is initialized with an absolute /aibox/ prefix matching baseUrl.
  const baseUrl = '/aibox/';

  const pillStyle = (active) => ({
    display: 'inline-block',
    padding: '4px 12px',
    borderRadius: 999,
    fontSize: '0.82rem',
    fontWeight: active ? 600 : 400,
    cursor: 'pointer',
    border: '1px solid var(--ifm-color-emphasis-300)',
    background: active ? 'var(--ifm-color-primary)' : 'var(--ifm-background-color)',
    color: active ? '#fff' : 'var(--ifm-font-color-base)',
    marginRight: 6,
    marginBottom: 6,
    userSelect: 'none',
  });

  return (
    <Layout
      title="Theme Gallery"
      description={`aibox supports ${totalCount} themes across multiple families and modes. Each tile shows a real terminal recording.`}
    >
      <div className="container margin-vert--lg">
        {/* Page header */}
        <h1>Theme Gallery</h1>
        <p style={{marginBottom: 24}}>
          aibox supports <strong>{totalCount} theme variants</strong> across{' '}
          <strong>{families.length} families</strong>. Each tile embeds a real asciinema
          recording of an aibox terminal session under that theme — powerkit status bar
          and all.
          {/* TODO: replace stub description once all casts are recorded */}
        </p>

        {/* Filter bar */}
        <div style={{marginBottom: 20}}>
          <div style={{marginBottom: 8, fontSize: '0.82rem', fontWeight: 600, color: 'var(--ifm-color-emphasis-600)'}}>
            Family
          </div>
          <div>
            <span
              style={pillStyle(selectedFamily === 'all')}
              onClick={() => setSelectedFamily('all')}
            >
              All
            </span>
            {families.map((f) => (
              <span
                key={f}
                style={pillStyle(selectedFamily === f)}
                onClick={() => setSelectedFamily(f)}
              >
                {f}
              </span>
            ))}
          </div>

          <div style={{marginTop: 12, marginBottom: 8, fontSize: '0.82rem', fontWeight: 600, color: 'var(--ifm-color-emphasis-600)'}}>
            Mode
          </div>
          <div>
            <span
              style={pillStyle(selectedMode === 'all')}
              onClick={() => setSelectedMode('all')}
            >
              All
            </span>
            {modes.map((m) => (
              <span
                key={m}
                style={pillStyle(selectedMode === m)}
                onClick={() => setSelectedMode(m)}
              >
                {m.charAt(0).toUpperCase() + m.slice(1)}
              </span>
            ))}
          </div>
        </div>

        {/* Result count */}
        <div style={{fontSize: '0.85rem', color: 'var(--ifm-color-emphasis-600)', marginBottom: 16}}>
          Showing {filtered.length} of {totalCount} themes
        </div>

        {/* Grid */}
        <div
          style={{
            display: 'grid',
            gridTemplateColumns: 'repeat(auto-fill, minmax(340px, 1fr))',
            gap: 20,
          }}
        >
          {filtered.map((theme) => (
            <ThemeCard key={theme.slug} theme={theme} baseUrl={baseUrl} />
          ))}
        </div>
      </div>
    </Layout>
  );
}

import { render } from "solid-js/web";
import "./styles.css";

function App() {
  return (
    <main class="shell">
      <header class="topbar">
        <div class="brand">Utagoe</div>
        <label class="global-search">
          <span>Search</span>
          <input type="search" placeholder="Track, artist, album, URL" />
        </label>
        <nav class="top-actions" aria-label="Global actions">
          <button class="icon-button" aria-label="Inbox">Inbox</button>
          <button class="icon-button" aria-label="Settings">Settings</button>
        </nav>
      </header>
      <section class="workspace">
        <section class="workbench" aria-label="Workbench">
          <div class="surface-tabs" role="tablist" aria-label="Workbench views">
            <button class="tab active" role="tab" aria-selected="true">
              Discover
            </button>
            <button class="tab" role="tab" aria-selected="false">
              Library
            </button>
            <button class="tab" role="tab" aria-selected="false">
              Organize
            </button>
          </div>
          <section class="surface">
            <header class="surface-header">
              <div>
                <h1>Discover</h1>
                <p>Mixed local and provider results will land here.</p>
              </div>
              <button class="primary-action">Open Track</button>
            </header>
            <div class="alert-bar">Provider search degraded. Showing partial results.</div>
            <div class="result-list">
              <article class="result-row">
                <div>
                  <h2>Plastic Love</h2>
                  <p>Mariya Takeuchi</p>
                </div>
                <span>Track</span>
              </article>
              <article class="result-row muted">
                <div>
                  <h2>YouTube Music</h2>
                  <p>Provider attention appears in Inbox.</p>
                </div>
                <span>Provider</span>
              </article>
            </div>
            <section class="track-screen" aria-label="Track screen">
              <header>
                <div>
                  <h2>Plastic Love</h2>
                  <p>Mariya Takeuchi</p>
                </div>
                <div class="track-actions">
                  <button>Play</button>
                  <button>Add to Queue</button>
                  <button>Organize</button>
                </div>
              </header>
            </section>
            <section class="settings-surface" aria-label="Settings">
              <nav class="settings-nav" aria-label="Settings sections">
                <button class="active">General</button>
                <button>Providers</button>
                <button>Storage</button>
              </nav>
              <div class="settings-grid">
                <label>
                  <span>Audio Device</span>
                  <select>
                    <option>Default</option>
                  </select>
                </label>
                <section>
                  <h2>YouTube Music</h2>
                  <p>Signed out</p>
                  <button>Sign in</button>
                </section>
                <section>
                  <h2>Managed Storage</h2>
                  <p>Prepared audio, artwork cache, metadata cache</p>
                </section>
              </div>
            </section>
          </section>
        </section>
        <aside class="playback-panel" aria-label="Playback panel">
          <header>
            <h2>Playback</h2>
            <span>Local</span>
          </header>
          <div class="now-playing">
            <strong>No track selected</strong>
            <p>saisei owns direct local audio output.</p>
          </div>
          <div class="transport">
            <button aria-label="Previous">Prev</button>
            <button aria-label="Play">Play</button>
            <button aria-label="Next">Next</button>
          </div>
        </aside>
      </section>
      <footer class="mini-player" aria-label="Mini player">
        <button class="management-entry">Playback</button>
        <div>
          <strong>No track selected</strong>
          <span>Queue idle</span>
        </div>
        <button aria-label="Play">Play</button>
      </footer>
    </main>
  );
}

render(() => <App />, document.getElementById("root")!);

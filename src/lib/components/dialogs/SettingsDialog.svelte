<script lang="ts">
  import { onMount } from 'svelte';
  import { getState, closeSettingsDialog, updateReaderSettings } from "$lib/stores/app.svelte";
  import { setThemeMode, getThemeMode } from "$lib/utils/theme";
  import { exportOpml, importOpml, setSetting, getSetting, getSyncAccounts, addSyncAccount, removeSyncAccount, syncNow } from "$lib/utils/tauri";
  import type { ThemeMode, SyncAccount } from "$lib/types";

  const appState = getState();

  let themeMode = $state<ThemeMode>(getThemeMode());
  let refreshInterval = $state(30); // minutes; 0 = never
  let opmlStatus = $state<string | null>(null);
  let isExporting = $state(false);
  let isImporting = $state(false);

  // Cache settings
  let cacheMaxDays = $state(60);
  let cacheMaxPerFeed = $state(500);

  // Sync accounts
  let syncAccounts = $state<SyncAccount[]>([]);
  let showAddSync = $state(false);
  let syncProvider = $state('freshrss');
  let syncServerUrl = $state('');
  let syncUsername = $state('');
  let syncPassword = $state('');
  let syncLoading = $state(false);
  let syncError = $state('');
  let syncingId = $state<number | null>(null);

  const REFRESH_OPTIONS = [
    { label: "15 minutes", value: 15 },
    { label: "30 minutes", value: 30 },
    { label: "1 hour", value: 60 },
    { label: "2 hours", value: 120 },
    { label: "4 hours", value: 240 },
    { label: "Never", value: 0 },
  ];

  const FONT_OPTIONS = [
    { value: "'Inter', sans-serif", label: 'Inter', type: 'sans' },
    { value: "'Plus Jakarta Sans', sans-serif", label: 'Plus Jakarta Sans', type: 'sans' },
    { value: "'DM Sans', sans-serif", label: 'DM Sans', type: 'sans' },
    { value: "'Literata', Georgia, serif", label: 'Literata', type: 'serif' },
    { value: "'Source Serif 4', Georgia, serif", label: 'Source Serif 4', type: 'serif' },
    { value: "'IBM Plex Serif', Georgia, serif", label: 'IBM Plex Serif', type: 'serif' },
    { value: "system-ui, sans-serif", label: 'System Default', type: 'sans' },
  ];

  const WIDTH_OPTIONS = [
    { label: "Narrow (600px)", value: 600 },
    { label: "Normal (720px)", value: 720 },
    { label: "Wide (900px)", value: 900 },
    { label: "Full width", value: 9999 },
  ];

  const SHORTCUTS = [
    { key: '↑ / ↓', action: 'Navigate articles' },
    { key: 'Enter', action: 'Open article' },
    { key: 'Space', action: 'Page down in reader' },
    { key: 'W', action: 'Toggle full-text extraction' },
    { key: 'R', action: 'Toggle read/unread' },
    { key: 'S', action: 'Star/unstar' },
    { key: 'B', action: 'Bookmark/read later' },
    { key: 'O', action: 'Open in browser' },
    { key: 'Delete', action: 'Mark read, next article' },
    { key: 'F5', action: 'Refresh feeds' },
    { key: 'Ctrl+Shift+A', action: 'Mark all as read' },
    { key: 'Ctrl+F', action: 'Search' },
    { key: 'Ctrl+,', action: 'Settings' },
    { key: 'Esc', action: 'Close / clear search' },
  ];

  onMount(async () => {
    const days = await getSetting('cache_max_days');
    const perFeed = await getSetting('cache_max_per_feed');
    if (days) cacheMaxDays = parseInt(days);
    if (perFeed) cacheMaxPerFeed = parseInt(perFeed);
    syncAccounts = await getSyncAccounts();
  });

  async function saveCacheSettings() {
    await setSetting('cache_max_days', String(cacheMaxDays));
    await setSetting('cache_max_per_feed', String(cacheMaxPerFeed));
  }

  async function handleAddSync() {
    syncLoading = true;
    syncError = '';
    try {
      const account = await addSyncAccount(syncProvider, syncServerUrl, syncUsername, syncPassword);
      syncAccounts = [...syncAccounts, account];
      showAddSync = false;
      syncServerUrl = ''; syncUsername = ''; syncPassword = '';
    } catch (e: any) {
      syncError = e?.toString() ?? 'Connection failed';
    } finally {
      syncLoading = false;
    }
  }

  async function handleRemoveSync(id: number) {
    await removeSyncAccount(id);
    syncAccounts = syncAccounts.filter(a => a.id !== id);
  }

  async function handleSyncNow(id: number) {
    syncingId = id;
    try { await syncNow(id); } catch {}
    syncingId = null;
    syncAccounts = await getSyncAccounts();
  }

  async function setTheme(mode: ThemeMode) {
    themeMode = mode;
    await setThemeMode(mode);
    await setSetting("theme_mode", mode);
  }

  async function handleExportOpml() {
    isExporting = true;
    opmlStatus = null;
    try {
      const xml = await exportOpml();
      const blob = new Blob([xml], { type: "text/xml" });
      const a = document.createElement("a");
      a.href = URL.createObjectURL(blob);
      a.download = `kublai-feeds-${new Date().toISOString().slice(0,10)}.opml`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(a.href);
      opmlStatus = "Exported successfully!";
    } catch (e) {
      opmlStatus = "Export failed. Please try again.";
      console.error("OPML export failed:", e);
    } finally {
      isExporting = false;
    }
  }

  async function handleImportOpml() {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = ".opml,.xml";
    input.onchange = async (e) => {
      const file = (e.target as HTMLInputElement).files?.[0];
      if (!file) return;
      isImporting = true;
      opmlStatus = null;
      try {
        const xml = await file.text();
        const feeds = await importOpml(xml);
        opmlStatus = `Imported ${feeds.length} feed${feeds.length !== 1 ? "s" : ""}!`;
      } catch (err) {
        opmlStatus = "Import failed. Make sure the file is a valid OPML file.";
        console.error("OPML import failed:", err);
      } finally {
        isImporting = false;
      }
    };
    input.click();
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) closeSettingsDialog();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") closeSettingsDialog();
  }
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  class="dialog-backdrop"
  role="dialog"
  aria-modal="true"
  aria-label="Settings"
  onclick={handleBackdropClick}
  onkeydown={handleKeydown}
>
  <div class="dialog">
    <div class="dialog-header">
      <h2 class="dialog-title">Settings</h2>
      <button class="dialog-close" onclick={closeSettingsDialog} aria-label="Close">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <line x1="18" y1="6" x2="6" y2="18"/>
          <line x1="6" y1="6" x2="18" y2="18"/>
        </svg>
      </button>
    </div>

    <div class="dialog-body">
      <!-- Appearance -->
      <section class="settings-section">
        <h3 class="section-title">Appearance</h3>

        <div class="setting-row">
          <div class="setting-info">
            <span class="setting-name">Theme</span>
            <span class="setting-desc">Choose your preferred color scheme</span>
          </div>
          <div class="theme-toggle">
            {#each (["light", "dark", "auto"] as ThemeMode[]) as mode}
              <button
                class="theme-btn"
                class:active={themeMode === mode}
                onclick={() => setTheme(mode)}
              >
                {#if mode === "light"}
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <circle cx="12" cy="12" r="5"/>
                    <line x1="12" y1="1" x2="12" y2="3"/><line x1="12" y1="21" x2="12" y2="23"/>
                    <line x1="4.22" y1="4.22" x2="5.64" y2="5.64"/><line x1="18.36" y1="18.36" x2="19.78" y2="19.78"/>
                    <line x1="1" y1="12" x2="3" y2="12"/><line x1="21" y1="12" x2="23" y2="12"/>
                    <line x1="4.22" y1="19.78" x2="5.64" y2="18.36"/><line x1="18.36" y1="5.64" x2="19.78" y2="4.22"/>
                  </svg>
                {:else if mode === "dark"}
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/>
                  </svg>
                {:else}
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <rect x="2" y="3" width="20" height="14" rx="2"/>
                    <line x1="8" y1="21" x2="16" y2="21"/>
                    <line x1="12" y1="17" x2="12" y2="21"/>
                  </svg>
                {/if}
                {mode.charAt(0).toUpperCase() + mode.slice(1)}
              </button>
            {/each}
          </div>
        </div>
      </section>

      <!-- Feed Updates -->
      <section class="settings-section">
        <h3 class="section-title">Feed Updates</h3>
        <div class="setting-row">
          <div class="setting-info">
            <span class="setting-name">Auto-refresh interval</span>
            <span class="setting-desc">How often to check for new articles</span>
          </div>
          <select
            class="form-select"
            bind:value={refreshInterval}
          >
            {#each REFRESH_OPTIONS as opt}
              <option value={opt.value}>{opt.label}</option>
            {/each}
          </select>
        </div>
      </section>

      <!-- Default Reader Settings -->
      <section class="settings-section">
        <h3 class="section-title">Reader Defaults</h3>

        <div class="setting-row vertical">
          <div class="setting-info">
            <span class="setting-name">Font Family</span>
          </div>
          <div class="option-list">
            {#each FONT_OPTIONS as opt}
              <button
                class="option-row"
                class:active={appState.readerSettings.font_family === opt.value}
                onclick={() => updateReaderSettings({ font_family: opt.value })}
                style="font-family: {opt.value}"
              >
                <div class="option-check">
                  {#if appState.readerSettings.font_family === opt.value}
                    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>
                  {/if}
                </div>
                {opt.label}
              </button>
            {/each}
          </div>
        </div>

        <div class="setting-row">
          <div class="setting-info">
            <span class="setting-name">Font Size</span>
          </div>
          <div class="stepper">
            <button class="stepper-btn" onclick={() => updateReaderSettings({ font_size: Math.max(12, appState.readerSettings.font_size - 1) })}>−</button>
            <span class="stepper-val">{appState.readerSettings.font_size}px</span>
            <button class="stepper-btn" onclick={() => updateReaderSettings({ font_size: Math.min(28, appState.readerSettings.font_size + 1) })}>+</button>
          </div>
        </div>

        <div class="setting-row">
          <div class="setting-info">
            <span class="setting-name">Line Height</span>
          </div>
          <div class="stepper">
            <button class="stepper-btn" onclick={() => updateReaderSettings({ line_height: Math.max(1.2, +(appState.readerSettings.line_height - 0.1).toFixed(1)) })}>−</button>
            <span class="stepper-val">{appState.readerSettings.line_height.toFixed(1)}</span>
            <button class="stepper-btn" onclick={() => updateReaderSettings({ line_height: Math.min(2.5, +(appState.readerSettings.line_height + 0.1).toFixed(1)) })}>+</button>
          </div>
        </div>

        <div class="setting-row vertical">
          <div class="setting-info">
            <span class="setting-name">Content Width</span>
          </div>
          <div class="option-list horizontal">
            {#each WIDTH_OPTIONS as opt}
              <button
                class="option-pill"
                class:active={appState.readerSettings.content_width === opt.value}
                onclick={() => updateReaderSettings({ content_width: opt.value })}
              >
                {opt.label}
              </button>
            {/each}
          </div>
        </div>
      </section>

      <!-- Cache -->
      <section class="settings-section">
        <h3 class="section-title">Article Cache</h3>
        <div class="setting-row">
          <label class="setting-label">Keep articles for</label>
          <div class="setting-control range-row">
            <input type="range" min="7" max="365" step="1" bind:value={cacheMaxDays} onchange={saveCacheSettings} />
            <span class="range-val">{cacheMaxDays} days</span>
          </div>
        </div>
        <div class="setting-row">
          <label class="setting-label">Max per feed</label>
          <div class="setting-control range-row">
            <input type="range" min="100" max="2000" step="100" bind:value={cacheMaxPerFeed} onchange={saveCacheSettings} />
            <span class="range-val">{cacheMaxPerFeed}</span>
          </div>
        </div>
      </section>

      <!-- Sync Accounts -->
      <section class="settings-section">
        <h3 class="section-title">Sync Accounts</h3>

        {#each syncAccounts as account}
          <div class="sync-account-row">
            <div class="sync-account-info">
              <span class="sync-provider">{account.provider === 'freshrss' ? 'FreshRSS' : 'Miniflux'}</span>
              <span class="sync-username">{account.username} @ {account.server_url.replace(/^https?:\/\//, '').split('/')[0]}</span>
              {#if account.last_synced}
                <span class="sync-last">Last synced: {new Date(account.last_synced).toLocaleString()}</span>
              {/if}
            </div>
            <div class="sync-account-actions">
              <button class="btn-icon" onclick={() => handleSyncNow(account.id)} disabled={syncingId === account.id} title="Sync now">
                <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class:spinning={syncingId === account.id}>
                  <polyline points="1 4 1 10 7 10"/><path d="M3.51 15a9 9 0 1 0 .49-3.42"/>
                </svg>
              </button>
              <button class="btn-icon danger" onclick={() => handleRemoveSync(account.id)} title="Remove">
                <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <polyline points="3 6 5 6 21 6"/><path d="M19 6l-1 14H6L5 6"/><path d="M10 11v6"/><path d="M14 11v6"/><path d="M9 6V4h6v2"/>
                </svg>
              </button>
            </div>
          </div>
        {/each}

        {#if syncAccounts.length === 0}
          <p class="sync-empty">No sync accounts configured.</p>
        {/if}

        {#if showAddSync}
          <div class="sync-add-form">
            <select bind:value={syncProvider} class="setting-select">
              <option value="freshrss">FreshRSS</option>
              <option value="miniflux">Miniflux</option>
            </select>
            <input type="url" bind:value={syncServerUrl} placeholder="Server URL (e.g. https://freshrss.example.com)" class="setting-input" />
            <input type="text" bind:value={syncUsername} placeholder="Username" class="setting-input" />
            <input type="password" bind:value={syncPassword} placeholder="Password" class="setting-input" />
            {#if syncError}<p class="sync-error">{syncError}</p>{/if}
            <div class="sync-form-actions">
              <button class="btn-secondary" onclick={() => { showAddSync = false; syncError = ''; }}>Cancel</button>
              <button class="btn-primary" onclick={handleAddSync} disabled={syncLoading || !syncServerUrl || !syncUsername}>
                {syncLoading ? 'Connecting…' : 'Connect'}
              </button>
            </div>
          </div>
        {:else}
          <button class="btn-secondary add-sync-btn" onclick={() => showAddSync = true}>+ Add Sync Account</button>
        {/if}
      </section>

      <!-- OPML -->
      <section class="settings-section">
        <h3 class="section-title">Import / Export</h3>
        <div class="opml-row">
          <button class="opml-btn" onclick={handleImportOpml} disabled={isImporting}>
            {#if isImporting}
              <div class="btn-spinner"></div>
            {:else}
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17 8 12 3 7 8"/><line x1="12" y1="3" x2="12" y2="15"/></svg>
            {/if}
            Import OPML
          </button>
          <button class="opml-btn" onclick={handleExportOpml} disabled={isExporting}>
            {#if isExporting}
              <div class="btn-spinner"></div>
            {:else}
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
            {/if}
            Export OPML
          </button>
        </div>
        {#if opmlStatus}
          <p class="opml-status" class:error={opmlStatus.includes("failed")}>{opmlStatus}</p>
        {/if}
      </section>

      <!-- Keyboard Shortcuts -->
      <section class="settings-section">
        <h3 class="section-title">Keyboard Shortcuts</h3>
        <div class="shortcuts-grid">
          {#each SHORTCUTS as shortcut}
            <div class="shortcut-row">
              <kbd class="shortcut-key">{shortcut.key}</kbd>
              <span class="shortcut-action">{shortcut.action}</span>
            </div>
          {/each}
        </div>
      </section>
    </div>
  </div>
</div>

<style>
  .dialog-backdrop {
    position: fixed;
    inset: 0;
    z-index: 1000;
    background: rgba(0, 0, 0, 0.3);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: flex-start;
    justify-content: flex-end;
    animation: fadein 0.15s ease;
  }

  @keyframes fadein {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  .dialog {
    width: 440px;
    height: 100%;
    background: var(--color-bg-elevated);
    border-left: 1px solid var(--color-border);
    box-shadow: -8px 0 32px rgba(0,0,0,0.15);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    animation: slidein 0.2s ease;
  }

  @keyframes slidein {
    from { transform: translateX(24px); opacity: 0; }
    to { transform: none; opacity: 1; }
  }

  .dialog-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 20px;
    border-bottom: 1px solid var(--color-divider);
    flex-shrink: 0;
  }

  .dialog-title {
    font-size: 15px;
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0;
  }

  .dialog-close {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 26px;
    height: 26px;
    background: none;
    border: none;
    border-radius: var(--radius-sm);
    color: var(--color-text-tertiary);
    cursor: pointer;
    transition: background 0.1s;
  }

  .dialog-close:hover {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  .dialog-body {
    flex: 1;
    overflow-y: auto;
    padding: 0 0 32px;
  }

  /* ===== Sections ===== */
  .settings-section {
    padding: 18px 20px 14px;
    border-bottom: 1px solid var(--color-divider);
  }

  .settings-section:last-child {
    border-bottom: none;
  }

  .section-title {
    font-size: 11px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--color-text-tertiary);
    margin: 0 0 14px;
  }

  .setting-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 8px 0;
  }

  .setting-row.vertical {
    flex-direction: column;
    align-items: flex-start;
  }

  .setting-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
    flex: 1;
    min-width: 0;
  }

  .setting-name {
    font-size: 13px;
    font-weight: 500;
    color: var(--color-text-primary);
  }

  .setting-desc {
    font-size: 11px;
    color: var(--color-text-tertiary);
  }

  .setting-label {
    font-size: 13px;
    font-weight: 500;
    color: var(--color-text-primary);
    flex-shrink: 0;
    min-width: 120px;
  }

  .setting-control {
    flex: 1;
  }

  /* ===== Theme Toggle ===== */
  .theme-toggle {
    display: flex;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    overflow: hidden;
    flex-shrink: 0;
  }

  .theme-btn {
    display: flex;
    align-items: center;
    gap: 5px;
    padding: 6px 10px;
    background: none;
    border: none;
    border-right: 1px solid var(--color-border);
    color: var(--color-text-secondary);
    font-size: 12px;
    cursor: pointer;
    transition: background 0.12s, color 0.12s;
    white-space: nowrap;
  }

  .theme-btn:last-child {
    border-right: none;
  }

  .theme-btn:hover {
    background: var(--color-bg-hover);
  }

  .theme-btn.active {
    background: var(--color-accent-soft);
    color: var(--color-accent);
    font-weight: 500;
  }

  /* ===== Form Select ===== */
  .form-select {
    height: 32px;
    padding: 0 8px;
    background: var(--color-bg-base);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    color: var(--color-text-primary);
    font-size: 13px;
    outline: none;
    cursor: pointer;
    flex-shrink: 0;
  }

  .form-select:focus {
    border-color: var(--color-accent);
  }

  /* ===== Option List ===== */
  .option-list {
    display: flex;
    flex-direction: column;
    gap: 2px;
    width: 100%;
  }

  .option-list.horizontal {
    flex-direction: row;
    flex-wrap: wrap;
    gap: 6px;
  }

  .option-row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 7px 10px;
    background: none;
    border: 1px solid transparent;
    border-radius: var(--radius-sm);
    color: var(--color-text-secondary);
    font-size: 13px;
    cursor: pointer;
    text-align: left;
    transition: all 0.12s;
    width: 100%;
  }

  .option-row:hover {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  .option-row.active {
    background: var(--color-accent-soft);
    border-color: var(--color-accent);
    color: var(--color-accent);
    font-weight: 500;
  }

  .option-check {
    width: 16px;
    height: 16px;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .option-pill {
    padding: 5px 12px;
    background: var(--color-bg-hover);
    border: 1px solid var(--color-border);
    border-radius: 20px;
    font-size: 12px;
    color: var(--color-text-secondary);
    cursor: pointer;
    transition: all 0.12s;
  }

  .option-pill:hover {
    border-color: var(--color-accent);
    color: var(--color-accent);
  }

  .option-pill.active {
    background: var(--color-accent-soft);
    border-color: var(--color-accent);
    color: var(--color-accent);
    font-weight: 500;
  }

  /* ===== Stepper ===== */
  .stepper {
    display: flex;
    align-items: center;
    gap: 6px;
    background: var(--color-bg-hover);
    border: 1px solid var(--color-border);
    border-radius: 20px;
    padding: 3px 8px;
    flex-shrink: 0;
  }

  .stepper-btn {
    width: 22px;
    height: 22px;
    background: none;
    border: none;
    font-size: 16px;
    line-height: 1;
    color: var(--color-text-secondary);
    cursor: pointer;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: background 0.1s;
  }

  .stepper-btn:hover {
    background: var(--color-bg-active);
    color: var(--color-text-primary);
  }

  .stepper-val {
    font-size: 13px;
    color: var(--color-text-primary);
    min-width: 42px;
    text-align: center;
    font-weight: 500;
  }

  /* ===== Range ===== */
  .range-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .range-row input[type="range"] {
    flex: 1;
    accent-color: var(--color-accent);
  }

  .range-val {
    font-size: 12px;
    color: var(--color-text-secondary);
    min-width: 60px;
    text-align: right;
  }

  /* ===== Sync Accounts ===== */
  .sync-account-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px;
    background: var(--color-bg-hover);
    border-radius: var(--radius-sm);
    margin-bottom: 6px;
  }

  .sync-account-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
    overflow: hidden;
  }

  .sync-provider {
    font-size: 11px;
    font-weight: 600;
    color: var(--color-accent);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .sync-username {
    font-size: 12px;
    color: var(--color-text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .sync-last {
    font-size: 10px;
    color: var(--color-text-tertiary);
  }

  .sync-account-actions {
    display: flex;
    gap: 4px;
    flex-shrink: 0;
  }

  .btn-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 26px;
    height: 26px;
    background: none;
    border: none;
    border-radius: var(--radius-sm);
    color: var(--color-text-secondary);
    cursor: pointer;
  }

  .btn-icon:hover { background: var(--color-bg-active); }
  .btn-icon.danger:hover { color: var(--color-danger); }
  .btn-icon:disabled { opacity: 0.4; cursor: default; }

  .sync-empty {
    font-size: 12px;
    color: var(--color-text-tertiary);
    margin: 4px 0;
  }

  .sync-error {
    font-size: 11px;
    color: var(--color-danger);
    margin: 4px 0;
  }

  .sync-add-form {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-top: 8px;
  }

  .setting-input {
    width: 100%;
    padding: 6px 8px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-bg-elevated);
    color: var(--color-text-primary);
    font-size: 12px;
    box-sizing: border-box;
    outline: none;
  }

  .setting-input:focus { border-color: var(--color-accent); }

  .setting-select {
    width: 100%;
    padding: 6px 8px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-bg-elevated);
    color: var(--color-text-primary);
    font-size: 12px;
    outline: none;
    cursor: pointer;
  }

  .sync-form-actions {
    display: flex;
    gap: 6px;
    justify-content: flex-end;
  }

  .btn-primary {
    display: inline-flex;
    align-items: center;
    height: 30px;
    padding: 0 14px;
    background: var(--color-accent);
    border: none;
    border-radius: var(--radius-sm);
    color: #fff;
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
    transition: opacity 0.12s;
  }

  .btn-primary:disabled { opacity: 0.5; cursor: default; }
  .btn-primary:hover:not(:disabled) { opacity: 0.85; }

  .btn-secondary {
    display: inline-flex;
    align-items: center;
    height: 30px;
    padding: 0 14px;
    background: var(--color-bg-hover);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    color: var(--color-text-secondary);
    font-size: 12px;
    cursor: pointer;
    transition: all 0.12s;
  }

  .btn-secondary:hover { background: var(--color-bg-active); color: var(--color-text-primary); }

  .add-sync-btn {
    width: 100%;
    margin-top: 4px;
    justify-content: center;
  }

  .spinning {
    animation: spin 1s linear infinite;
  }

  /* ===== OPML ===== */
  .opml-row {
    display: flex;
    gap: 8px;
    margin-bottom: 8px;
  }

  .opml-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    height: 34px;
    padding: 0 14px;
    background: var(--color-bg-hover);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    color: var(--color-text-secondary);
    font-size: 13px;
    cursor: pointer;
    transition: all 0.12s;
  }

  .opml-btn:hover:not(:disabled) {
    background: var(--color-bg-active);
    color: var(--color-text-primary);
    border-color: var(--color-border-strong);
  }

  .opml-btn:disabled {
    opacity: 0.5;
    cursor: default;
  }

  .opml-status {
    font-size: 12px;
    color: var(--color-success);
    margin: 4px 0 0;
  }

  .opml-status.error {
    color: var(--color-danger);
  }

  /* ===== Shortcuts ===== */
  .shortcuts-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px 16px;
    padding-bottom: 12px;
  }

  .shortcut-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .shortcut-key {
    font-size: 11px;
    padding: 2px 6px;
    background: var(--color-bg-surface);
    border: 1px solid var(--color-border-strong);
    border-radius: 4px;
    color: var(--color-text-secondary);
    font-family: "Cascadia Code", "Consolas", monospace;
    white-space: nowrap;
    flex-shrink: 0;
  }

  .shortcut-action {
    font-size: 12px;
    color: var(--color-text-secondary);
  }

  /* ===== Spinner ===== */
  .btn-spinner {
    width: 12px;
    height: 12px;
    border: 2px solid var(--color-border);
    border-top-color: var(--color-accent);
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
</style>

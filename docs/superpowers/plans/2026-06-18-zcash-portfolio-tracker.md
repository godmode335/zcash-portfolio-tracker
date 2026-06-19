# Zcash Portfolio Tracker (Project 2) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship a Zcash-only (ZEC) local-first Windows portfolio tracker app plus a distinctly-designed SEO marketing site, forked from Project 1 and simplified to a single coin.

**Architecture:** Fork Project 1's source (`C:\Users\user\privacy-coin-tracker`) into the new repo, then (a) reduce the Rust core to a single coin (ZEC), (b) simplify the frontend, (c) rebrand the app, (d) strip the site's multi-coin pages, (e) give the site a *unique* visual identity (NOT a recolor of Project 1), and (f) write Zcash-specific content. Reuses Project 1's tested Rust/TS logic and build pipeline.

**Tech Stack:** Tauri 2 (Rust + web + SQLite), Astro 5 + @astrojs/sitemap + @fontsource fonts, Vitest, Chart.js. Deploy: Cloudflare Pages; releases: GitHub Releases.

## Global Constraints

- App name (productName / display): **"Zcash Portfolio Tracker"**. App identifier: **`com.zcashportfoliotracker.app`**. Cargo package: `zcash-portfolio-tracker`.
- Single coin only: **Zcash (ZEC)**, CoinGecko id `zcash`. No coin selector anywhere.
- Money/quantities use `rust_decimal::Decimal`; SQLite stores them as TEXT; timestamps as INTEGER Unix seconds.
- All user data local (SQLite in app-data dir); only public ZEC price requests leave the machine.
- Site: zero client JS on content pages; unique title/description/canonical/OG per page.
- Stable download link: `https://github.com/godmode335/zcash-portfolio-tracker/releases/latest/download/ZcashPortfolioTracker-Setup.exe`.
- **Design must be visually distinct from Project 1** (which is dark, minimal, green, centered, system-font). Project 2 identity: **Zcash gold `#F4B728`** accent on **warm graphite** background, **left-aligned editorial hero**, **bento-grid** features, **display+sans font pairing**, **pill buttons**.
- Default placeholder domain `https://zcashportfoliotracker.com` (single source: `website/src/config.ts` `SITE_URL` and `astro.config.mjs` `site`, kept identical).
- Privacy brand: no third-party trackers/cookies (Cloudflare Web Analytics only, added at deploy).

---

## File Structure (after fork)

```
zcash-portfolio-tracker/
  src-tauri/              # forked Rust app — coins.rs reduced to ZEC; rebranded
    Cargo.toml            # package name, identifier
    tauri.conf.json       # productName, identifier, window title, icons
    src/{models,coins,portfolio,db,prices,commands,scheduler,lib,main}.rs
    icons/                # regenerated gold icon
  website/                # forked Astro site — multi-coin removed, redesigned
    src/config.ts         # ZEC branding + domain + download asset
    src/lib/seo.ts(.test) # inherited (coinPageJsonLd dropped)
    src/layouts/BaseLayout.astro   # adds @fontsource fonts
    src/styles/global.css # NEW unique design system (gold/graphite/bento)
    src/components/{Header,Footer,DownloadButton}.astro
    src/pages/{index,download,about,privacy}.astro + blog/
    src/content/blog/*.md # 5 Zcash articles
    public/screenshots/   # ZEC app screenshot
  docs/superpowers/...    # this plan + spec (already present)
```

---

### Task 1: Bootstrap the repo by forking Project 1

**Files:** copies Project 1's source into this repo (excluding build artifacts, `.git`, and Project 1's `docs/`).

**Interfaces:**
- Consumes: Project 1 at `C:\Users\user\privacy-coin-tracker`.
- Produces: a building, testing clone in `C:\Users\user\zcash-portfolio-tracker` (still branded as Project 1 until later tasks).

- [ ] **Step 1: Copy source (exclude artifacts and Project 1 docs)**

Run (PowerShell; `robocopy` exit codes 0–7 mean success):
```powershell
robocopy "C:\Users\user\privacy-coin-tracker" "C:\Users\user\zcash-portfolio-tracker" /E `
  /XD ".git" "node_modules" "dist" "target" "gen" ".astro" "docs" `
  /XF "*.db" "appicon.png"
if ($LASTEXITCODE -lt 8) { "copy OK ($LASTEXITCODE)" } else { "copy FAILED ($LASTEXITCODE)"; exit 1 }
```

- [ ] **Step 2: Install website deps and verify the clone builds + tests**

```powershell
$env:PATH = "$env:ProgramFiles\nodejs;$env:ProgramFiles\Git\cmd;$env:PATH"
cd C:\Users\user\zcash-portfolio-tracker\website; npm install; npm test; npm run build
cd C:\Users\user\zcash-portfolio-tracker\src-tauri; cargo test
```
Expected: website 9 tests pass + build OK; cargo 17 tests pass.

- [ ] **Step 3: Commit the fork baseline**

```powershell
$env:PATH = "$env:ProgramFiles\Git\cmd;$env:PATH"; cd C:\Users\user\zcash-portfolio-tracker
git add -A; git commit -m "chore: fork Project 1 source as Zcash Portfolio Tracker baseline"
```

---

### Task 2: Reduce the Rust core to a single coin (ZEC)

**Files:**
- Modify: `src-tauri/src/coins.rs` (replace coin list + tests)

**Interfaces:**
- Consumes: nothing new.
- Produces: `coins::all()` returns exactly one `Coin` (ZEC); `by_id("zcash")` works; `coingecko_ids()` == `["zcash"]`. Same `Coin` struct and function signatures as Project 1 (so `commands.rs`/`scheduler.rs` are unchanged).

- [ ] **Step 1: Replace the coin table and tests**

Overwrite `src-tauri/src/coins.rs`:
```rust
#[derive(Debug, Clone, Copy)]
pub struct Coin {
    pub id: &'static str,
    pub symbol: &'static str,
    pub name: &'static str,
    pub coingecko_id: &'static str,
}

static COINS: &[Coin] = &[
    Coin { id: "zcash", symbol: "ZEC", name: "Zcash", coingecko_id: "zcash" },
];

pub fn all() -> &'static [Coin] { COINS }

pub fn by_id(id: &str) -> Option<&'static Coin> {
    COINS.iter().find(|c| c.id == id)
}

pub fn coingecko_ids() -> Vec<&'static str> {
    COINS.iter().map(|c| c.coingecko_id).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_only_zcash() {
        assert_eq!(all().len(), 1);
        assert_eq!(all()[0].id, "zcash");
    }

    #[test]
    fn lookup_zcash_works() {
        assert_eq!(by_id("zcash").unwrap().symbol, "ZEC");
        assert!(by_id("monero").is_none());
    }

    #[test]
    fn coingecko_ids_is_zcash() {
        assert_eq!(coingecko_ids(), vec!["zcash"]);
    }
}
```

- [ ] **Step 2: Run tests**

```powershell
$env:PATH = "$env:ProgramFiles\nodejs;$env:ProgramFiles\Git\cmd;$env:PATH"
cd C:\Users\user\zcash-portfolio-tracker\src-tauri; cargo test
```
Expected: all pass (the 3 coins tests replaced; others unchanged still green).

- [ ] **Step 3: Commit**

```powershell
git add src-tauri/src/coins.rs; git commit -m "feat: reduce coin set to Zcash only"
```

---

### Task 3: Simplify the frontend to single-coin

**Files:**
- Modify: `website` is the site — NOT this. This task edits the **app** frontend at `src/` (Tauri webview): `src/main.ts`, `src/forms.ts`.

Wait — the Tauri app frontend lives at repo-root `src/` (forked from Project 1). Edit those.

**Interfaces:**
- Consumes: `api.ts` (`listCoins`, `addTransaction`, `getCoinHistory`).
- Produces: a dashboard with no coin chooser. The add-transaction form has no coin dropdown (ZEC implied); the price chart always shows ZEC.

- [ ] **Step 1: Remove the coin dropdown from the transaction form**

In `src/forms.ts`, replace the coin `<select>` usage. Change the form markup so there is no `#f-coin` select, and set the coin to `"zcash"` directly. Replace the `panel.innerHTML` "Add transaction" fieldset with:
```html
    <fieldset>
      <legend>Add ZEC transaction</legend>
      <select id="f-type"><option value="buy">Buy</option><option value="sell">Sell</option></select>
      <input id="f-qty" type="number" step="any" placeholder="ZEC amount" />
      <input id="f-price" type="number" step="any" placeholder="Price USD" />
      <input id="f-date" type="date" />
      <button id="f-add">Add</button>
      <span id="f-err" class="neg"></span>
    </fieldset>
```
Remove the `listCoins().then(...)` block that filled `#f-coin`. In the add handler, replace the line that read the coin select with a constant:
```ts
    const coin = "zcash";
```
(Keep the rest of the handler — type, qty, price, date, addTransaction — unchanged.)

- [ ] **Step 2: Lock the price chart to ZEC (remove its selector)**

In `src/main.ts`, the charts markup currently includes a `#coin-chart-select`. Replace the `<section class="charts">` block with:
```html
  <section class="charts">
    <canvas id="pf-chart" height="120"></canvas>
    <div><h3>ZEC price (30d)</h3><canvas id="coin-chart" height="120"></canvas></div>
  </section>
```
Then in `show(id)`, replace the coin-select logic with a fixed ZEC chart. Remove the block that reads/fills `coin-chart-select` and replace `drawCoin` with:
```ts
async function drawCoin() {
  const cc = document.getElementById("coin-chart") as HTMLCanvasElement;
  renderCoinChart(cc, "zcash", await getCoinHistory("zcash", 30));
}
```
Remove the now-unused `coinSel` references and the `listCoins` import if unused.

- [ ] **Step 3: Verify the app frontend builds (type-check via vite build)**

```powershell
$env:PATH = "$env:ProgramFiles\nodejs;$env:ProgramFiles\Git\cmd;$env:PATH"
cd C:\Users\user\zcash-portfolio-tracker; npx tsc --noEmit
```
Expected: no type errors.

- [ ] **Step 4: Commit**

```powershell
git add src; git commit -m "feat: single-coin (ZEC) app frontend"
```

---

### Task 4: Rebrand the app (name, identifier, gold theme, icon)

**Files:**
- Modify: `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `src/styles.css` (app theme), `index.html` (title)
- Create: `appicon.png` (gold) and regenerate `src-tauri/icons/`

**Interfaces:**
- Consumes: nothing.
- Produces: app titled "Zcash Portfolio Tracker", identifier `com.zcashportfoliotracker.app`, gold-accent theme, gold icon. (New identifier ⇒ separate local DB from Project 1.)

- [ ] **Step 1: Cargo package name**

In `src-tauri/Cargo.toml`, change the `[package]` name:
```toml
name = "zcash-portfolio-tracker"
```
(Leave `[lib] name = "app_lib"` as-is.)

- [ ] **Step 2: tauri.conf.json branding**

In `src-tauri/tauri.conf.json` set:
```json
  "productName": "Zcash Portfolio Tracker",
  "identifier": "com.zcashportfoliotracker.app",
```
and in `app.windows[0].title`: `"Zcash Portfolio Tracker"`.

- [ ] **Step 3: index.html + app theme to gold/graphite**

In `index.html` set `<title>Zcash Portfolio Tracker</title>`.
Overwrite `src/styles.css` with a warm-graphite + gold theme (distinct from Project 1's green):
```css
:root { color-scheme: dark; --bg:#17130c; --fg:#f3ecdd; --muted:#b6a98f; --accent:#f4b728; --card:#211b11; --line:#33291a; }
* { box-sizing: border-box; }
body { font-family: system-ui, "Segoe UI", sans-serif; margin: 0; background: var(--bg); color: var(--fg); }
#app { padding: 16px; }
header { display: flex; gap: 12px; align-items: center; }
h1 { letter-spacing: -.02em; }
table { width: 100%; border-collapse: collapse; margin-top: 12px; }
th, td { text-align: left; padding: 6px 10px; border-bottom: 1px solid var(--line); }
.totals { display: flex; gap: 18px; margin: 12px 0; font-size: 15px; }
.pos { color: #6cc070; } .neg { color: #e6603c; } .muted { color: var(--muted); }
.charts { display: grid; grid-template-columns: 1fr 1fr; gap: 16px; margin-top: 16px; }
.forms { display: flex; gap: 16px; margin-top: 16px; flex-wrap: wrap; }
fieldset { border: 1px solid var(--line); border-radius: 8px; }
button, select, input { background: var(--card); color: var(--fg); border: 1px solid var(--line); padding: 6px 10px; border-radius: 6px; margin: 2px; }
button { background: var(--accent); color: #1a1305; font-weight: 700; border-color: var(--accent); cursor: pointer; }
```
Also update the chart line color: in `src/charts.ts` change `borderColor: "#4caf50"` to `borderColor: "#f4b728"`.

- [ ] **Step 4: Generate a gold icon set**

```powershell
Add-Type -AssemblyName System.Drawing
$bmp = New-Object System.Drawing.Bitmap 1024,1024
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
$g.Clear([System.Drawing.Color]::FromArgb(23,19,12))
$brush = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(244,183,40))
$g.FillEllipse($brush, 112,112,800,800)
$font = New-Object System.Drawing.Font('Segoe UI',430,[System.Drawing.FontStyle]::Bold)
$sf = New-Object System.Drawing.StringFormat; $sf.Alignment=1; $sf.LineAlignment=1
$g.DrawString('Z', $font, [System.Drawing.Brushes]::Black, (New-Object System.Drawing.RectangleF(0,0,1024,1024)), $sf)
$g.Dispose(); $bmp.Save('C:\Users\user\zcash-portfolio-tracker\appicon.png',[System.Drawing.Imaging.ImageFormat]::Png); $bmp.Dispose()
$env:PATH = "$env:ProgramFiles\nodejs;$env:ProgramFiles\Git\cmd;$env:PATH"
cd C:\Users\user\zcash-portfolio-tracker; npm run tauri icon appicon.png
```
Expected: `src-tauri/icons/` regenerated (icon.ico present).

- [ ] **Step 5: Verify app builds**

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:ProgramFiles\nodejs;$env:ProgramFiles\Git\cmd;$env:PATH"
cd C:\Users\user\zcash-portfolio-tracker\src-tauri; cargo build
```
Expected: compiles.

- [ ] **Step 6: Commit**

```powershell
git add -A; git commit -m "feat: rebrand app to Zcash Portfolio Tracker (gold theme, icon, identifier)"
```

---

### Task 5: Strip multi-coin from the site + ZEC copy

**Files:**
- Delete: `website/src/pages/coins/[coin].astro`, `website/src/pages/coins/index.astro`, `website/src/data/coins.ts`
- Modify: `website/src/config.ts`, `website/astro.config.mjs`, `website/src/components/Header.astro`, `website/src/pages/index.astro`, `website/src/lib/seo.ts` + `seo.test.ts` (drop `coinPageJsonLd`)

**Interfaces:**
- Consumes: inherited `seo.ts` builders.
- Produces: a single-coin site (no `/coins`), ZEC-branded config, homepage without the coin grid.

- [ ] **Step 1: Delete the coin pages and data**

```powershell
cd C:\Users\user\zcash-portfolio-tracker\website
Remove-Item src\pages\coins -Recurse -Force
Remove-Item src\data\coins.ts -Force
```

- [ ] **Step 2: Update config + astro site**

Overwrite `website/src/config.ts`:
```ts
export const PRODUCT_NAME = "Zcash Portfolio Tracker";
export const SITE_URL = "https://zcashportfoliotracker.com";
export const GITHUB_OWNER = "godmode335";
export const GITHUB_REPO = "zcash-portfolio-tracker";
export const DOWNLOAD_ASSET = "ZcashPortfolioTracker-Setup.exe";
```
In `website/astro.config.mjs` set `site: "https://zcashportfoliotracker.com"`.

- [ ] **Step 3: Remove `coinPageJsonLd` and its test**

In `website/src/lib/seo.ts` delete the `coinPageJsonLd` function. In `website/src/lib/seo.test.ts` delete its import name and the test `"coinPageJsonLd is a per-coin SoftwareApplication with coin url"`.

- [ ] **Step 4: Header without Coins; homepage without coin grid**

In `website/src/components/Header.astro` remove the `<a href="/coins">Coins</a>` line.
In `website/src/pages/index.astro`: remove the `import { COINS } from "../data/coins";` line and delete the entire `<section>` that maps `COINS` (the "Track your privacy coins" block). Update the page `title`/`description`/hero copy to Zcash, e.g. title `"Zcash Portfolio Tracker — Private ZEC Tracker for Windows"`, hero H1 `"Track your Zcash privately"`, and feature/FAQ copy referencing ZEC specifically (replace "privacy coins"/"Zcash, Monero…" with Zcash/ZEC).

- [ ] **Step 5: Verify**

```powershell
$env:PATH = "$env:ProgramFiles\nodejs;$env:ProgramFiles\Git\cmd;$env:PATH"
cd C:\Users\user\zcash-portfolio-tracker\website; npm test; npm run build
```
Expected: tests pass (8 now), build OK, no `/coins` output.

- [ ] **Step 6: Commit**

```powershell
git add -A; git commit -m "feat(web): strip multi-coin, brand site to Zcash"
```

---

### Task 6: Unique site design system (the differentiator)

**Files:**
- Modify: `website/src/styles/global.css` (full rewrite), `website/src/layouts/BaseLayout.astro` (fonts), `website/package.json` (font deps), `website/src/components/DownloadButton.astro` (pill), `website/src/pages/index.astro` (left hero + bento)

**Interfaces:**
- Consumes: nothing new.
- Produces: a visual identity clearly distinct from Project 1 — gold `#F4B728` on warm graphite, **left-aligned two-column hero**, **bento-grid** features, **pill buttons**, **Space Grotesk (display) + Inter (body)** self-hosted fonts.

- [ ] **Step 1: Add self-hosted fonts (privacy-friendly, no Google request)**

```powershell
$env:PATH = "$env:ProgramFiles\nodejs;$env:ProgramFiles\Git\cmd;$env:PATH"
cd C:\Users\user\zcash-portfolio-tracker\website
npm install @fontsource/space-grotesk @fontsource/inter
```

- [ ] **Step 2: Import fonts in BaseLayout**

In `website/src/layouts/BaseLayout.astro` frontmatter (top script), add imports below the existing `import "../styles/global.css";`:
```ts
import "@fontsource/space-grotesk/600.css";
import "@fontsource/space-grotesk/700.css";
import "@fontsource/inter/400.css";
import "@fontsource/inter/500.css";
```

- [ ] **Step 3: Rewrite `global.css` with the unique system**

Overwrite `website/src/styles/global.css`:
```css
/* Zcash Portfolio Tracker — unique design system (warm graphite + gold, editorial/bento) */
:root {
  --bg: #17130c;
  --bg-2: #1d1810;
  --surface: #221b10;
  --surface-2: #2b2214;
  --line: #382d18;
  --fg: #f4ecdc;
  --muted: #bcab8c;
  --gold: #f4b728;
  --gold-2: #ffcb4d;
  --ink: #1a1305;
  --radius: 18px;
  --maxw: 1080px;
  --shadow: 0 14px 40px rgba(0,0,0,.45);
  color-scheme: dark;
}
* { box-sizing: border-box; }
html { scroll-behavior: smooth; }
body {
  margin: 0; color: var(--fg);
  font-family: "Inter", system-ui, sans-serif; line-height: 1.65;
  background:
    radial-gradient(900px 480px at 85% -5%, rgba(244,183,40,.12), transparent 60%),
    var(--bg);
  -webkit-font-smoothing: antialiased;
}
a { color: var(--gold-2); }
.container { max-width: var(--maxw); margin: 0 auto; padding: 0 22px; }

h1, h2, h3, .brand { font-family: "Space Grotesk", system-ui, sans-serif; }
h1 { font-size: clamp(2.1rem, 5vw, 3.6rem); line-height: 1.05; letter-spacing: -.03em; margin: .1em 0 .25em; }
h2 { font-size: clamp(1.5rem, 2.6vw, 2.1rem); letter-spacing: -.02em; }
.lead { font-size: clamp(1.05rem, 1.5vw, 1.28rem); color: #d8cdb6; max-width: 56ch; }
.muted { color: var(--muted); }
.eyebrow { display:inline-block; font-family:"Space Grotesk"; font-weight:700; font-size:.78rem; letter-spacing:.16em; text-transform:uppercase; color: var(--gold); }
section { margin: 64px 0; }

/* Header / Footer */
.site-header { border-bottom: 1px solid var(--line); position: sticky; top: 0; z-index: 20; background: rgba(23,19,12,.72); backdrop-filter: blur(8px); }
.site-footer { border-top: 1px solid var(--line); color: var(--muted); margin-top: 72px; }
.nav { display: flex; gap: 22px; align-items: center; padding: 15px 0; flex-wrap: wrap; }
.nav a { color: var(--fg); text-decoration: none; font-size: .95rem; }
.nav a:hover { color: var(--gold); }
.nav .brand { font-weight: 700; margin-right: auto; letter-spacing: -.02em; }

/* Pill buttons */
.download-btn {
  display: inline-flex; flex-direction: column; align-items: center;
  background: linear-gradient(180deg, var(--gold-2), var(--gold));
  color: var(--ink); font-weight: 700; text-decoration: none;
  padding: 15px 34px; border-radius: 999px; box-shadow: var(--shadow);
  transition: transform .15s ease, box-shadow .15s ease, filter .15s ease;
}
.download-btn:hover { transform: translateY(-2px); filter: brightness(1.05); }
.download-btn:active { transform: translateY(0) scale(.99); }
.download-btn .dl-sub { font-weight: 600; font-size: .8rem; opacity: .8; margin-top: 3px; }
a:focus-visible, .download-btn:focus-visible { outline: 3px solid var(--gold); outline-offset: 3px; }

/* Left-aligned two-column hero */
.hero { display: grid; grid-template-columns: 1.05fr .95fr; gap: 40px; align-items: center; padding: 56px 0 24px; }
.hero .cta-row { display: flex; gap: 14px; align-items: center; margin-top: 22px; flex-wrap: wrap; }
.hero .shot { margin: 0; border: 1px solid var(--line); border-radius: var(--radius); overflow: hidden; box-shadow: var(--shadow); background: var(--surface); }
.hero .shot img { display: block; width: 100%; height: auto; }
.trust { display: flex; flex-wrap: wrap; gap: 8px; margin-top: 18px; }
.trust span { font-size: .82rem; color: var(--muted); border: 1px solid var(--line); border-radius: 999px; padding: 5px 13px; background: var(--surface); }

/* Bento grid */
.bento { display: grid; grid-template-columns: repeat(6, 1fr); gap: 16px; }
.bento .tile { background: var(--surface); border: 1px solid var(--line); border-radius: var(--radius); padding: 22px; grid-column: span 2; transition: transform .15s ease, border-color .15s ease; }
.bento .tile:hover { transform: translateY(-3px); border-color: rgba(244,183,40,.45); }
.bento .tile.wide { grid-column: span 3; }
.bento .tile.tall { grid-column: span 3; }
.bento .tile h3 { margin: 0 0 .4em; }

/* FAQ + checks + article */
.checks { list-style: none; padding: 0; display: grid; gap: 10px; max-width: 56ch; }
.checks li { position: relative; padding-left: 28px; }
.checks li::before { content: "◆"; position: absolute; left: 0; color: var(--gold); }
.faq dt { font-family: "Space Grotesk"; font-weight: 700; margin-top: 18px; }
.faq dd { margin: 4px 0 0; color: var(--muted); }
article p, article li { max-width: 70ch; }
article :is(h2,h3) { margin-top: 1.6em; }
article hr { border: 0; border-top: 1px solid var(--line); margin: 32px 0; }

@media (max-width: 820px) { .hero { grid-template-columns: 1fr; } .bento .tile, .bento .tile.wide, .bento .tile.tall { grid-column: span 6; } }
@media (prefers-reduced-motion: reduce) { html { scroll-behavior: auto; } * { transition: none !important; } }
```

- [ ] **Step 4: Pill download button copy**

In `website/src/components/DownloadButton.astro` change the sub-label text to ZEC tone: `Free · Windows 10/11 · private`. (Markup/classes stay; CSS makes it a pill.)

- [ ] **Step 5: Rebuild the landing with left hero + bento**

Overwrite `website/src/pages/index.astro`:
```astro
---
import BaseLayout from "../layouts/BaseLayout.astro";
import DownloadButton from "../components/DownloadButton.astro";
import { softwareAppJsonLd, faqJsonLd, organizationJsonLd, webSiteJsonLd } from "../lib/seo";

const title = "Zcash Portfolio Tracker — Private ZEC Tracker for Windows";
const description = "Track your Zcash (ZEC) holdings, cost basis and P&L privately on Windows. Local-first, no account, no cloud. Free download.";

const tiles = [
  { h: "Built for ZEC", p: "A focused Zcash tracker — not a 10,000-coin app. Everything is about your ZEC.", cls: "wide" },
  { h: "Private by design", p: "Holdings stay in a local database on your PC. Only the public ZEC price is fetched.", cls: "tall" },
  { h: "Real P&L", p: "Transaction-based cost basis with realized and unrealized profit/loss." },
  { h: "Charts", p: "Portfolio value history and the ZEC price chart." },
  { h: "Multiple portfolios", p: "Split ZEC into separate portfolios (e.g. hold vs trade)." },
  { h: "No account", p: "No sign-up, no email, no KYC. ~3 MB, Windows 10/11." },
];

const faqs = [
  { q: "Is Zcash Portfolio Tracker free?", a: "Yes — free to download and use." },
  { q: "Does it upload my ZEC holdings?", a: "No. Data stays in a local database on your PC; only the public ZEC price is fetched." },
  { q: "Why not read my Zcash balance from the chain?", a: "Shielded (z-address) balances are encrypted and unreadable on-chain by design — so you record transactions instead." },
  { q: "Do I need an account?", a: "No account, email or KYC." },
  { q: "Which OS?", a: "Windows 10 and 11 (64-bit)." },
];
---
<BaseLayout title={title} description={description} path="/" jsonLd={[organizationJsonLd(), webSiteJsonLd(), softwareAppJsonLd(), faqJsonLd(faqs)]}>
  <section class="hero">
    <div>
      <span class="eyebrow">Local-first · ZEC only</span>
      <h1>Track your Zcash privately</h1>
      <p class="lead">A focused, local-first Windows tracker for Zcash (ZEC). Your holdings never leave your PC.</p>
      <div class="cta-row"><DownloadButton /></div>
      <div class="trust"><span>No account</span><span>100% local</span><span>Open source</span><span>~3 MB · Win 10/11</span></div>
    </div>
    <figure class="shot">
      <img src="/screenshots/dashboard.png" width="1393" height="720"
        alt="Zcash Portfolio Tracker dashboard showing ZEC holdings, value and profit/loss" loading="eager" decoding="async" />
    </figure>
  </section>

  <section>
    <h2>Everything for your ZEC</h2>
    <div class="bento">
      {tiles.map((t) => (<div class={`tile ${t.cls ?? ""}`}><h3>{t.h}</h3><p class="muted">{t.p}</p></div>))}
    </div>
  </section>

  <section>
    <h2>Frequently asked questions</h2>
    <dl class="faq">{faqs.map((f) => (<><dt>{f.q}</dt><dd>{f.a}</dd></>))}</dl>
  </section>

  <section class="hero" style="grid-template-columns:1fr; text-align:center;">
    <div><h2>Start tracking ZEC privately</h2><p><DownloadButton label="Download free for Windows" /></p></div>
  </section>
</BaseLayout>
```

- [ ] **Step 6: Verify build + visual sanity**

```powershell
$env:PATH = "$env:ProgramFiles\nodejs;$env:ProgramFiles\Git\cmd;$env:PATH"
cd C:\Users\user\zcash-portfolio-tracker\website; npm run check; npm run build
```
Expected: 0 errors; build OK. (`/screenshots/dashboard.png` is added in Task 9; until then it 404s in preview — fine for build.)

- [ ] **Step 7: Commit**

```powershell
git add -A; git commit -m "feat(web): unique gold/graphite design system (bento, left hero, pill, fonts)"
```

---

### Task 7: Rewrite the five blog articles for Zcash

**Files:**
- Replace contents of the five files in `website/src/content/blog/` with Zcash-specific articles (reuse filenames or rename). Final set:
  - `track-zcash-holdings.md`, `zcash-wallet-portfolio-tracker.md`, `zcash-tax-tracking.md`, `shielded-vs-transparent-zcash.md`, `zcash-price-history-guide.md`

**Interfaces:**
- Consumes: the inherited blog schema (`title`, `description`, `keywords`, `date`, `draft`).
- Produces: 5 published ZEC articles, each with a `/download` CTA.

- [ ] **Step 1: Remove Project 1 articles, add ZEC articles**

```powershell
cd C:\Users\user\zcash-portfolio-tracker\website\src\content\blog
Remove-Item *.md -Force
```
Create `track-zcash-holdings.md`:
```markdown
---
title: "How to Track Your Zcash (ZEC) Holdings Privately"
description: "A step-by-step guide to tracking your Zcash holdings, cost basis and profit/loss without handing your data to a cloud service."
keywords: ["track zcash", "zcash portfolio tracker", "zec holdings"]
date: 2026-06-18
---

If you hold Zcash for its privacy, using a cloud tracker that wants your email and wallet addresses defeats the purpose. Here is the private way to track ZEC.

## Why address-based tracking fails for Zcash
Shielded Zcash (z-addresses) can't be read from the public chain — that's the whole point. Only transparent t-addresses expose a balance. So address trackers either see nothing or leak your transparent activity.

## The private approach
1. Record each ZEC buy and sell — amount, price, date.
2. Use a local tracker that computes holdings, average cost and P&L.
3. Refresh the price from a public source — that request reveals only "zcash", not your balance.

## Do it with Zcash Portfolio Tracker
Zcash Portfolio Tracker is a free, local-first Windows app built for exactly this. Your transactions live in a local database; only the public ZEC price leaves your machine.

[Download Zcash Portfolio Tracker](/download) and set up your ZEC portfolio in minutes.
```
Create `zcash-wallet-portfolio-tracker.md`:
```markdown
---
title: "The Best Way to Track a Zcash Wallet's Portfolio"
description: "Why a local-first tracker beats connecting your Zcash wallet to a cloud service, and how to set one up on Windows."
keywords: ["zcash wallet tracker", "zcash portfolio", "zec tracker"]
date: 2026-06-18
---

Connecting a Zcash wallet to an online portfolio service means sharing addresses and trusting a server with your financial profile. For a privacy coin, that's the wrong trade.

## Local-first beats wallet-connect
- No account to breach, no server to trust.
- Your ZEC transactions and history stay on your PC.
- Only the public ZEC price is fetched.

## What to track
Record buys and sells; the app derives your ZEC quantity, average entry price, current value and realized/unrealized P&L, with a portfolio value chart.

[Download Zcash Portfolio Tracker](/download) — free, Windows 10/11, no account.
```
Create `zcash-tax-tracking.md`:
```markdown
---
title: "Tracking Zcash for Taxes: Cost Basis and P&L"
description: "How to keep clean Zcash records — cost basis, realized gains and losses — with a private, local-first tracker."
keywords: ["zcash tax", "zec cost basis", "zcash gains tracking"]
date: 2026-06-18
---

Good tax records start with good transaction records. For Zcash, that means logging every buy and sell with its price and date.

## What you need
- **Cost basis:** the average price you paid for your ZEC.
- **Realized P&L:** gains/losses from sells.
- **Unrealized P&L:** paper gains on what you still hold.

A transaction-based tracker computes all three automatically as you enter trades.

## Keep it private
Zcash Portfolio Tracker stores everything locally — no account, no cloud — so your tax records aren't sitting on someone else's server.

[Download Zcash Portfolio Tracker](/download) to keep clean, private ZEC records.
```
Create `shielded-vs-transparent-zcash.md`:
```markdown
---
title: "Shielded vs Transparent Zcash: What's the Difference?"
description: "Zcash z-addresses vs t-addresses explained — privacy, balances, and what it means for tracking your ZEC."
keywords: ["shielded vs transparent zcash", "z-address", "t-address", "zcash privacy"]
date: 2026-06-18
---

Zcash supports two kinds of addresses, and the difference matters for both privacy and tracking.

## Transparent (t-addresses)
Work like Bitcoin: sender, receiver and amount are public. A balance can be read from the chain.

## Shielded (z-addresses)
Use zero-knowledge proofs (zk-SNARKs) to encrypt sender, receiver and amount. Balances **cannot** be read publicly — that's the privacy guarantee.

## What this means for tracking
Because shielded balances are private, the reliable way to track your ZEC is to record your own transactions in a local tool, not to scrape an address.

Zcash Portfolio Tracker does exactly that, privately on your PC. [Download it free](/download).
```
Create `zcash-price-history-guide.md`:
```markdown
---
title: "Following Zcash (ZEC) Price History the Private Way"
description: "How to watch ZEC price history and your portfolio value over time without an account or cloud sync."
keywords: ["zcash price history", "zec price chart", "zcash portfolio value"]
date: 2026-06-18
---

Watching the ZEC price is one thing; seeing how your own portfolio tracks against it is what actually matters.

## Price vs portfolio
- **ZEC price chart:** the market over the last 30 days.
- **Portfolio value chart:** your holdings' worth over time, built from local snapshots.

Together they show whether your position is growing, independent of daily noise.

## Private by default
Zcash Portfolio Tracker pulls only the public ZEC price; your holdings and history never leave your PC.

[Download Zcash Portfolio Tracker](/download) — free for Windows.
```

- [ ] **Step 2: Verify build + sitemap**

```powershell
$env:PATH = "$env:ProgramFiles\nodejs;$env:ProgramFiles\Git\cmd;$env:PATH"
cd C:\Users\user\zcash-portfolio-tracker\website; npm run build
```
Expected: 5 article pages under `dist/blog/`; each `dist/blog/<slug>/index.html` contains `BlogPosting`.

- [ ] **Step 3: Commit**

```powershell
git add -A; git commit -m "content(web): five Zcash blog articles"
```

---

### Task 8: SEO finalize (download page, about/privacy, robots)

**Files:**
- Modify: `website/src/pages/download.astro`, `website/src/pages/about.astro`, `website/src/pages/privacy.astro`, `website/public/robots.txt`

**Interfaces:**
- Consumes: inherited components.
- Produces: ZEC-branded supporting pages; robots referencing the new domain sitemap.

- [ ] **Step 1: ZEC copy on download/about/privacy**

In `website/src/pages/download.astro`, `about.astro`, `privacy.astro` replace product references "Privacy Coin Tracker" → "Zcash Portfolio Tracker" and "privacy coins (Zcash, Monero…)" → "Zcash (ZEC)". (PRODUCT_NAME-driven strings already update via config; fix only the hardcoded prose/titles.)

- [ ] **Step 2: robots.txt sitemap host**

Overwrite `website/public/robots.txt`:
```
User-agent: *
Allow: /

Sitemap: https://zcashportfoliotracker.com/sitemap-index.xml
```

- [ ] **Step 3: Verify**

```powershell
$env:PATH = "$env:ProgramFiles\nodejs;$env:ProgramFiles\Git\cmd;$env:PATH"
cd C:\Users\user\zcash-portfolio-tracker\website; npm test; npm run check; npm run build
```
Expected: tests pass (8), 0 check errors, build OK; `dist/sitemap-index.xml` present; `dist/index.html` has `Organization` + `WebSite`.

- [ ] **Step 4: Commit**

```powershell
git add -A; git commit -m "feat(web): finalize ZEC SEO pages and robots"
```

---

### Task 9: Real ZEC app screenshot for the site

**Files:**
- Create: `website/public/screenshots/dashboard.png`

**Interfaces:**
- Consumes: the built app (Task 4) and its local DB.
- Produces: a populated ZEC dashboard screenshot in the hero.

- [ ] **Step 1: Build the app (release) if not already built**

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:ProgramFiles\nodejs;$env:ProgramFiles\Git\cmd;$env:PATH"
cd C:\Users\user\zcash-portfolio-tracker; npm run tauri build
```
Expected: `src-tauri/target/release/zcash-portfolio-tracker.exe` and an installer under `src-tauri/target/release/bundle/`.

- [ ] **Step 2: Seed ZEC demo data**

Create `C:\Users\user\seed-zec.mjs`:
```js
import { DatabaseSync } from "node:sqlite";
import { homedir } from "node:os";
import { join } from "node:path";
const dbPath = join(homedir(), "AppData", "Roaming", "com.zcashportfoliotracker.app", "tracker.db");
const db = new DatabaseSync(dbPath);
db.exec(`
CREATE TABLE IF NOT EXISTS portfolios (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL, created_at INTEGER NOT NULL);
CREATE TABLE IF NOT EXISTS transactions (id INTEGER PRIMARY KEY AUTOINCREMENT, portfolio_id INTEGER NOT NULL, coin_id TEXT NOT NULL, tx_type TEXT NOT NULL, quantity TEXT NOT NULL, price_usd TEXT NOT NULL, fee_usd TEXT NOT NULL, ts INTEGER NOT NULL, note TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS price_cache (coin_id TEXT PRIMARY KEY, price_usd TEXT NOT NULL, price_btc TEXT NOT NULL, updated_at INTEGER NOT NULL);
CREATE TABLE IF NOT EXISTS value_snapshots (id INTEGER PRIMARY KEY AUTOINCREMENT, portfolio_id INTEGER NOT NULL, total_usd TEXT NOT NULL, ts INTEGER NOT NULL);`);
for (const t of ["transactions","price_cache","value_snapshots","portfolios"]) db.exec(`DELETE FROM ${t};`);
const now = Math.floor(Date.now()/1000), day = 86400;
db.prepare("INSERT INTO portfolios (id,name,created_at) VALUES (1,'Main',?)").run(now-120*day);
const tx = db.prepare("INSERT INTO transactions (portfolio_id,coin_id,tx_type,quantity,price_usd,fee_usd,ts,note) VALUES (1,'zcash','buy',?,?,'0',?,'')");
tx.run("8","34.50",now-110*day); tx.run("5","41.00",now-70*day); tx.run("4","52.00",now-30*day);
db.prepare("INSERT INTO price_cache (coin_id,price_usd,price_btc,updated_at) VALUES ('zcash',?,?,?)").run("58.40","0.00094",now);
const snap = db.prepare("INSERT INTO value_snapshots (portfolio_id,total_usd,ts) VALUES (1,?,?)");
[560,610,700,680,790,910,870,980,1010].forEach((v,i,a)=>snap.run(String(v),now-(a.length-1-i)*7*day));
console.log("seeded ZEC demo at", dbPath); db.close();
```
Run:
```powershell
$env:PATH = "$env:ProgramFiles\nodejs;$env:PATH"; node --experimental-sqlite C:\Users\user\seed-zec.mjs
```

- [ ] **Step 3: Launch + capture the window (top region, DPI-aware)**

Use the same capture approach as Project 1: launch `src-tauri/target/release/zcash-portfolio-tracker.exe`, poll `$p.MainWindowHandle`, `SetProcessDPIAware`, `ShowWindow`/`SetForegroundWindow`, send `^{HOME}` then `{HOME}`, then `GetWindowRect` and `CopyFromScreen` the top `min(720, height)` region into `website/public/screenshots/dashboard.png`. Then `Stop-Process`. (Full script: adapt Project 1's capture command, changing only the exe path and output dir to this repo.)
Expected: `website/public/screenshots/dashboard.png` ~1393×720 showing populated ZEC dashboard (gold theme).

- [ ] **Step 4: Clean up seed script, rebuild site, commit**

```powershell
Remove-Item C:\Users\user\seed-zec.mjs -Force
$env:PATH = "$env:ProgramFiles\nodejs;$env:ProgramFiles\Git\cmd;$env:PATH"
cd C:\Users\user\zcash-portfolio-tracker\website; npm run build
cd C:\Users\user\zcash-portfolio-tracker; git add website/public/screenshots; git commit -m "feat(web): add real ZEC dashboard screenshot"
```

---

### Task 10: Verify, publish repo, wire release

**Files:** `website/DEPLOY.md` (update names); no code.

**Interfaces:**
- Consumes: gh (authenticated), the built installer.
- Produces: GitHub repo + a release whose asset matches `DOWNLOAD_ASSET`, making the site's Download button live.

- [ ] **Step 1: Full verification**

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:ProgramFiles\nodejs;$env:ProgramFiles\Git\cmd;$env:PATH"
cd C:\Users\user\zcash-portfolio-tracker\src-tauri; cargo test
cd C:\Users\user\zcash-portfolio-tracker\website; npm test; npm run check; npm run build
```
Expected: cargo all pass; website 8 tests pass; 0 check errors; build OK.

- [ ] **Step 2: Update DEPLOY.md names**

In `website/DEPLOY.md` replace repo/name references with `zcash-portfolio-tracker` / "Zcash Portfolio Tracker" and the asset `ZcashPortfolioTracker-Setup.exe`. (Same Cloudflare Pages steps: root `website`, build `npm run build`, output `dist`.)

- [ ] **Step 3: Create the GitHub repo and push**

```powershell
$env:PATH = "$env:ProgramFiles\GitHub CLI;$env:ProgramFiles\Git\cmd;$env:PATH"
cd C:\Users\user\zcash-portfolio-tracker
git add -A; git commit -m "docs: update deploy runbook for Zcash Portfolio Tracker"
gh repo create zcash-portfolio-tracker --public --source=. --remote=origin
git push -u origin master
```

- [ ] **Step 4: Publish release with stable asset (makes Download live)**

```powershell
$env:PATH = "$env:ProgramFiles\GitHub CLI;$env:ProgramFiles\Git\cmd;$env:PATH"
cd C:\Users\user\zcash-portfolio-tracker
Copy-Item "src-tauri\target\release\bundle\nsis\Zcash Portfolio Tracker_0.1.0_x64-setup.exe" "ZcashPortfolioTracker-Setup.exe"
gh release create v0.1.0 "ZcashPortfolioTracker-Setup.exe" --title "Zcash Portfolio Tracker v0.1.0" --notes "First public release. Local-first Zcash (ZEC) portfolio tracker for Windows."
Remove-Item "ZcashPortfolioTracker-Setup.exe"
```
Verify:
```powershell
curl.exe -s -o NUL -w "%{http_code}\n" -L "https://github.com/godmode335/zcash-portfolio-tracker/releases/latest/download/ZcashPortfolioTracker-Setup.exe"
```
Expected: `200`.

---

## Self-Review

**1. Spec coverage**
- Single-coin ZEC app (fork + simplify) → Tasks 1, 2, 3. ✅
- Rebrand (name/identifier/icon/theme) → Task 4. ✅
- Site stripped of multi-coin + ZEC copy → Task 5. ✅
- **Unique design** (gold/graphite, left hero, bento, pill, display+sans fonts) → Task 6. ✅
- Blog with 5 Zcash articles → Task 7. ✅
- SEO (Org/WebSite/BlogPosting/FAQ JSON-LD, sitemap/robots, per-page meta) → inherited + Tasks 5, 8. ✅
- Multiple portfolios → inherited (unchanged from Project 1). ✅
- P&L, charts, manual entry, CoinGecko ZEC price, local SQLite, scheduler → inherited (Tasks 1–3). ✅
- Real ZEC screenshot → Task 9. ✅
- Separate repo + working Download (release) → Tasks 1, 10. ✅
- Domain-ready single source → Task 5 config + astro site. ✅
- Out-of-MVP (alerts, education, address watch, distribution) → excluded. ✅

**2. Placeholder scan:** No TBD/TODO. Article bodies and the full design CSS/landing are concrete. Task 9 Step 3 references "adapt Project 1's capture command" — Project 1's exact script exists in this session's history and in Project 1's git; the only changes are the exe path and output dir, both given. Default domain is an explicit documented constant.

**3. Type consistency:** `coins.rs` keeps the Project 1 `Coin` struct and `all()/by_id()/coingecko_ids()` signatures, so `commands.rs`/`scheduler.rs` compile unchanged. `seo.ts` builders used on the landing (`organizationJsonLd`, `webSiteJsonLd`, `softwareAppJsonLd`, `faqJsonLd`) match their definitions; `coinPageJsonLd` removed in Task 5 along with its only consumers (deleted coin pages) and its test. Config constant names (`PRODUCT_NAME`, `SITE_URL`, `GITHUB_OWNER`, `GITHUB_REPO`, `DOWNLOAD_ASSET`) match `seo.ts` usage. `DOWNLOAD_ASSET` = `ZcashPortfolioTracker-Setup.exe` matches the release asset in Task 10.

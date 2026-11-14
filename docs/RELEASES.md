# Releasing a version

The installers are built on GitHub Actions (`.github/workflows/instaladores.yml`) and published on [Releases](https://github.com/joseMChavez/selah-app/releases). See D-013 in [DECISIONS.md](DECISIONS.md).

| System | Files |
|---|---|
| Linux | `.deb` (Debian/Ubuntu), `.rpm` (Fedora/openSUSE), `.AppImage` (any distribution) |
| Windows | `.msi` and `-setup.exe` (NSIS) |
| macOS | universal `.dmg` (Intel and Apple Silicon) |

## Steps

1. Bump the version on `main` through a pull request: `app/package.json`, `app/package-lock.json` (two places), `app/src-tauri/Cargo.toml`, `app/src-tauri/Cargo.lock` (the `app` package), `app/src-tauri/tauri.conf.json`, and add a row to the history below.
2. Once it is merged, tag the merge commit with the form `vX.Y.Z` and push the tag:
   ```sh
   git switch main && git pull
   git tag v0.9.0
   git push origin v0.9.0
   ```
   The installers take their version from the tag (`v0.9.0` → `0.9.0`), so a mismatch only affects local builds — but tag the commit that has the bumped version (see `v0.3.1` below).
3. Actions builds the three systems (about 15–25 minutes) and leaves a **draft** on Releases. A tag that isn't `vX.Y.Z` fails right away.
4. Review the draft, test at least one installer and press **Publish release**.

A tag builds the code of the commit it points to. If a release fails and something needs fixing, push the next tag (`v0.9.1`) from the fixed `main`.

## Building without publishing

On GitHub: **Actions → Instaladores → Run workflow**. The installers are kept as artifacts of that run for 90 days.

Locally (Linux only; includes the Bibles in `app/src-tauri/resources/bibles/`):
```sh
cd app
npm run tauri build
```
The files end up in `app/src-tauri/target/release/bundle/`.

## Icon

The source is `app/src-tauri/icons/icon.svg` (also `app/static/favicon.svg`). After changing it, regenerate every size:
```sh
cd app
rsvg-convert -w 1024 -h 1024 src-tauri/icons/icon.svg -o /tmp/emetpro-icon.png
npx tauri icon /tmp/emetpro-icon.png -o src-tauri/icons
rm -rf src-tauri/icons/android src-tauri/icons/ios src-tauri/icons/64x64.png   # not used
cp src-tauri/icons/icon.svg static/favicon.svg
rsvg-convert -w 128 -h 128 static/favicon.svg -o static/favicon.png
rsvg-convert -w 192 -h 192 static/favicon.svg -o static/remote/icon-192.png
rsvg-convert -w 512 -h 512 static/favicon.svg -o static/remote/icon-512.png
rsvg-convert -w 180 -h 180 -b white static/favicon.svg -o static/remote/apple-touch-icon.png
```
The README logo (`docs/assets/logo-light.png` and `logo-dark.png`) is the same icon next to the name, in Noto Sans Bold.

## Warnings when installing

The installers are not signed:
- **Windows:** SmartScreen says "Windows protected your PC" → "More info" → "Run anyway".
- **macOS:** right-click EmetPro in Applications → "Open" → "Open". If it says the app is damaged: `xattr -cr /Applications/EmetPro.app`.
- **Linux:** the AppImage needs execute permission (`chmod +x EmetPro_*.AppImage`).

## History

Up to `v0.8.2` the app was called **Selah**.

| Tag | Result | What happened |
|---|---|---|
| `v0.2.0` | ❌ Failed | The code was still at `0.1.0` and the workflow required the tag to match the version in the files. Nothing was built. |
| `v0.2.1` | ❌ Failed | Same cause. Lesson: the workflow now takes the version from the tag (PR #5). |
| `v0.2.2` | ⚠️ Partial | Windows and macOS built; Linux failed installing dependencies: `libgstreamer1.0-dev` clashes with libunwind on ubuntu-22.04. Lesson: install only the packages that are needed (PR #6). A draft without a Linux installer remains. |
| `v0.2.3` | Pending | First version with installers for the three systems. |
| `v0.3.0` | Pending | Lyrics (library, service list, image or GIF backgrounds, fonts), Bible in two versions, clock and timer, PowerPoint presentations (with LibreOffice), logo as a watermark, "Send to screen" opens the output by itself, and settings and lyrics stored in SQLite with a reset option (D-014). |
| `v0.3.1` | ❌ Discarded | The tag was created on `06dcd44` (the 0.3.0 code) before merging #9 and #10: the installers say 0.3.1 but lack PowerPoint and Keynote. Lesson: bump the version on `main` and tag that commit. Its draft is not published. |
| `v0.3.2` | Pending | Presentations also with PowerPoint (Windows) and Keynote or PowerPoint (macOS), not only LibreOffice. Actions on Node 24. |
| `v0.3.3` | Pending | Keynote and PowerPoint on macOS: the file is passed as an alias so the sandboxed app can open it (Keynote used to say it couldn't open the file). |
| `v0.4.0` | Pending | Presentations with LibreOffice first; if it's missing, the app offers to download its own copy (26.2.6, SHA-256 checked) without installing it system-wide, removable from Settings (D-015). |
| `v0.5.0` | Pending | Import and export lyrics: own `.selah` format (with service list and backgrounds) and Holyrics `.hpls` playlists, with a preview and a choice for duplicates (D-017). Ask before sending to the church screen. OBS no longer starts in Safe Mode after a forced quit, and if it does, the app explains it. A line with `**` separates slides. Code organized by feature, each one an internal plugin (D-016). |
| `v0.6.0` | Pending | Remote control from a phone (D-018): turned on in Settings and paired with a QR code on the same Wi-Fi. Searches and shows verses, changes the Bible version, moves through verses and slides, blacks out the screen and returns to recent content (a presentation on the page where it was left). Asks for confirmation if the console does. Full screen, no zoom. Ports 7300–7309 may need to be opened in the firewall. |
| `v0.7.0` | Pending | Service plans (D-019): a Plan tab to prepare what will be shown, with name, date and notes, and sections, lyrics, verses (paste the preacher's list), media and pauses. Click prepares, double-click sends and "Next" advances, skipping missing items. Media are stored as a copy in the app's folder. The plan can be seen and used from the phone. The Texts list becomes a plan. The timer has a title, its own background and a red border when time is up. **A database upgraded to this version can't be opened by 0.6.0:** to go back, restore a copy of `selah.db`. |
| `v0.8.0` | Pending | Remote control (D-020): the phone shows the page of the presentation or PDF on screen and a strip to jump to any page, lyric slide or verse in a range. Jumping doesn't ask: "Back to …" stays for a few seconds to undo. Lyrics show on the phone with their line breaks. |
| `v0.8.1` | Pending | Whiteboard from the remote control (D-021): draw with a finger on the phone over what's on screen (underline or highlight a verse, mark a slide or a PDF page) or on blank sheets, shown on the projector while drawing. Pen, highlighter, eraser, colors, widths and undo; it lasts while the remote control is on. On macOS the output now goes to the chosen monitor instead of staying on the laptop screen. |
| `v0.8.2` | Pending | Remote control: returning to a verse shown in another version switches the console and the phone to that version and opens that chapter on the phone; the verse list is folded around the verse on screen ("Whole chapter"), and the version is chosen with buttons. Console: "Preview" and "On screen" in tabs (the right one opens when preparing or sending), the arrow keys move the visible tab, and the menu fits short or narrow windows. |
| `v0.9.0` | Pending | **Selah is renamed EmetPro** (D-022), with a new icon. On first launch it copies the Selah data (database, Bibles, backgrounds, plan media, LibreOffice copy, remote-control key) into its own folder and fixes the stored paths; the Selah folder is kept as a backup. Lyrics are exported as `.emetpro`; `.selah` files still import. OBS scenes and sources "Selah · …" are renamed "EmetPro · …". Paired phones keep working without scanning again. README and release notes in English. |

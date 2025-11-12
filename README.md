# L4D2 Addon Manager

A cross-platform app that lets you manage your l4d2 addons with ease, allowing you to quickly add, delete, disable, and search your long list of addons.
In addition, let's you quickly offload workshop addons to be manually installed, allowing them to be loaded instantly on game startup.

Currently in development and only tested running on linux

### Current Features:

* Speedy addon list
* Scanning system that extracts data from addon files
  * Addon types (campaign, scripts, skins, weapons, sounds, etc)
  * Coop campaign chapter ids

### Future Work

* User tagging of addons
* Downloading new addons from workshop
* Updating manual addons
* Profiles (sets of addons)
* Testing on Windows

### Preview

![preview.png](images/preview.png)

## Building

1. Set DATABASE_URL (either via `src-tauri/.env`) or other means to where sqlite:// should be for testing
2. Install dependencies `pnpm i`
3. Build `cargo tauri build`

## License

MIT



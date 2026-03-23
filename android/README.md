# Iron Insights Android

Iron Insights Android is the native Kotlin + Jetpack Compose client for the published Iron Insights dataset.

It consumes the same versioned payload tree as the website instead of running a separate mobile backend:

- `data/latest.json`
- `data/<version>/index.json`
- `data/<version>/index_shards/<sex>/<equip>/index.json`
- referenced `hist/*.bin` and `heat/*.bin`
- `data/<version>/trends.json`

Current app surfaces:

- `Lookup` for percentile checks
- `Compare` for shard-summary comparisons
- `Trends` for yearly cohort and threshold views
- `Calculators` for 1RM and plate-loading tools

## Project Layout

- `app/` - Android application module
- `RELEASING.md` - GitHub Actions and Play upload notes
- `TODO.md` - Android-specific roadmap

## Prerequisites

Best local workflow:

- Android Studio
- JDK 17
- Android SDK Platform 35

If you want to build from the command line instead of Android Studio:

- Gradle 8.9 on your `PATH`

Important:

- there is currently no checked-in Gradle wrapper under `android/`
- Android Studio is the simplest Windows workflow today

## Open In Android Studio

1. Open the `android/` directory as a project.
2. Let Android Studio install any missing SDK or JDK pieces.
3. Wait for Gradle sync to finish.
4. Run the `app` configuration on an emulator or physical device.

## Command-Line Build

From the repo root:

```bash
gradle -p android testDebugUnitTest
gradle -p android :app:assembleDebug
```

Debug APK output:

- `android/app/build/outputs/apk/debug/app-debug.apk`

If you prefer to run the commands from inside `android/`:

```bash
gradle testDebugUnitTest
gradle :app:assembleDebug
```

On Windows PowerShell the commands are the same if `gradle` is installed and on `PATH`.

## Release Build

Release metadata and signing are configured from Gradle properties or environment variables.

Supported environment variables:

- `IRON_INSIGHTS_VERSION_CODE`
- `IRON_INSIGHTS_VERSION_NAME`
- `IRON_INSIGHTS_REQUIRE_RELEASE_SIGNING`
- `IRON_INSIGHTS_KEYSTORE_PATH`
- `IRON_INSIGHTS_KEYSTORE_PASSWORD`
- `IRON_INSIGHTS_KEY_ALIAS`
- `IRON_INSIGHTS_KEY_PASSWORD`

Example PowerShell flow:

```powershell
$env:IRON_INSIGHTS_VERSION_CODE="1"
$env:IRON_INSIGHTS_VERSION_NAME="0.1.0"
$env:IRON_INSIGHTS_REQUIRE_RELEASE_SIGNING="true"
$env:IRON_INSIGHTS_KEYSTORE_PATH="C:\Users\grego\keys\iron-insights-release.keystore"
$env:IRON_INSIGHTS_KEYSTORE_PASSWORD="..."
$env:IRON_INSIGHTS_KEY_ALIAS="ironinsights"
$env:IRON_INSIGHTS_KEY_PASSWORD="..."

gradle -p android validateReleaseConfig :app:bundleRelease
```

Release bundle output:

- `android/app/build/outputs/bundle/release/app-release.aab`

For GitHub Actions release automation and optional Google Play upload, see [RELEASING.md](./RELEASING.md).

## Notes

- The app currently fetches live data from `https://gregorycarnegie.github.io/iron_insights/`.
- The Android client caches downloaded payloads and can fall back to cached versions when the network or latest pointer is unavailable.
- The launcher icon and theme now track the website branding, but additional per-screen polish is still planned.

# Iron Insights Android

Iron Insights Android is a native Kotlin + Jetpack Compose powerlifting companion app. It combines population-level analytics from the published Iron Insights dataset with a full training log, programme builder, personal progress tracking, and equipment management.

## Features

### Analytics (population data)

Consumes the same versioned payload tree as the website instead of running a separate mobile backend:

- `data/latest.json`
- `data/<version>/index.json`
- `data/<version>/index_shards/<sex>/<equip>/index.json`
- referenced `hist/*.bin` and `heat/*.bin`
- `data/<version>/trends.json`

Surfaces:

- **Lookup** -- percentile checks against the full population dataset
- **Compare** -- shard-summary comparisons across cohorts
- **Trends** -- yearly cohort and threshold views
- **Calculators** -- 1RM estimation (Brzycki/Epley blend) and plate-loading tools

### Training log

- Start/finish workout sessions with timestamped tracking
- Add exercises from a built-in library of 28 exercises (competition lifts + accessories)
- Log sets with weight, reps, and RPE; automatic e1RM computation on every set
- Duplicate-previous-set fast logging
- Rest timer with countdown display
- Session volume summary (total sets, reps, volume load, average intensity)
- Personal record detection (e1RM, weight, and rep PRs)

### Programme templates and calendar

- Create training programmes with periodised blocks (hypertrophy, strength, peak, deload)
- Plan sessions with prescribed sets using percentage-of-1RM, RPE, or fixed-weight targets
- Automatic deload prescription generation by block type
- Prescription resolver converts planned sets into concrete weights using current e1RM

### Personal progress dashboards

- e1RM trend charts for competition lifts (squat, bench, deadlift)
- Personal record board with all-time bests
- Population percentile rank integration -- see where your e1RM sits in the dataset

### Export and import

- CSV export in Strong-compatible format (`Date,Workout Name,Exercise Name,Set Order,Weight,Reps,RPE,Notes`)
- Full-fidelity JSON export of all training data
- CSV import from Strong app exports
- Background processing via WorkManager

### Equipment presets

- Create and manage plate inventories with custom weights, colours, and pair counts
- Bar presets (name + weight) for different barbells
- Set active inventory/bar to use in the plate calculator
- Plate calculator respects available pair counts from custom inventories

### Health Connect integration

- Opt-in sync of completed workouts to Health Connect as exercise sessions
- Read latest body weight from Health Connect
- Background sync via WorkManager after each finished workout

### Settings

- Weight unit toggle (kg/lb)
- Default bar weight and rounding increment
- Active plate inventory and bar preset selection
- Health Connect enable/disable

## Architecture

- **Navigation**: Jetpack Navigation Compose with `NavHost`/`NavController` and Material3 `NavigationBar`
- **Database**: Room with KSP annotation processing (11 entities, 7 DAOs)
- **Preferences**: Preferences DataStore
- **Background work**: WorkManager `CoroutineWorker` for export/import and Health Connect sync
- **ViewModels**: Compose `mutableStateOf` state with companion `factory()` methods
- **Threading**: New ViewModels use `viewModelScope` + coroutines; original analytics VM uses `ExecutorService`

## Project Layout

- `app/` -- Android application module
- `RELEASING.md` -- GitHub Actions and Play upload notes
- `TODO.md` -- Android-specific roadmap

## Prerequisites

Best local workflow:

- Android Studio (Meerkat or later)
- JDK 21 (Android Studio bundled JBR recommended)
- Android SDK Platform 36 (compile SDK) with BAKLAVA target
- Gradle 9.4.1 (checked-in wrapper)
- Kotlin 2.3.20, AGP 9.1.0

If you want to build from the command line instead of Android Studio:

- use the checked-in Gradle wrapper in `android/`
- set `JAVA_HOME` to the Android Studio bundled JBR (e.g. `C:/Program Files/Android/Android Studio/jbr`)

Important:

- Android Studio is the simplest Windows workflow today

## Open In Android Studio

1. Open the `android/` directory as a project.
2. Let Android Studio install any missing SDK or JDK pieces.
3. Wait for Gradle sync to finish.
4. Run the `app` configuration on an emulator or physical device.

## Command-Line Build

From the repo root:

```bash
./android/gradlew -p android testDebugUnitTest
./android/gradlew -p android :app:assembleDebug
```

Debug APK output:

- `android/app/build/outputs/apk/debug/app-debug.apk`

If you prefer to run the commands from inside `android/`:

```bash
./gradlew testDebugUnitTest
./gradlew :app:assembleDebug
```

On Windows PowerShell, use:

```powershell
.\android\gradlew.bat -p android testDebugUnitTest
.\android\gradlew.bat -p android :app:assembleDebug
```

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

.\android\gradlew.bat -p android validateReleaseConfig :app:bundleRelease
```

Release bundle output:

- `android/app/build/outputs/bundle/release/app-release.aab`

For GitHub Actions release automation and optional Google Play upload, see [RELEASING.md](./RELEASING.md).

## Key Dependencies

| Library | Purpose |
| ------- | ------- |
| Jetpack Compose + Material3 | UI framework |
| Navigation Compose | Screen routing |
| Room + KSP | Local SQLite database |
| Preferences DataStore | User settings |
| WorkManager | Background export/import and Health Connect sync |
| Health Connect SDK | Exercise session and body weight integration |
| Kotlinx Coroutines | Async operations |

## Notes

- The app fetches live population data from `https://gregorycarnegie.github.io/iron_insights/`.
- The Android client caches downloaded payloads and can fall back to cached versions when the network or latest pointer is unavailable.
- All training data is stored locally in a Room database (`iron_insights_training.db`).
- Health Connect integration is opt-in and the app works fully without it.
- The launcher icon and theme track the website branding (NightVoid dark theme with AccentLime primary).

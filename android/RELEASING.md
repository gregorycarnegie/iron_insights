# Android Releasing

## GitHub Environment

Create a GitHub environment named `android-release`.

Required secrets for signed bundle builds:

- `ANDROID_KEYSTORE_BASE64`: base64-encoded release keystore file
- `ANDROID_KEYSTORE_PASSWORD`: keystore password
- `ANDROID_KEY_ALIAS`: release key alias
- `ANDROID_KEY_PASSWORD`: release key password

Optional secret for Google Play upload:

- `PLAY_SERVICE_ACCOUNT_JSON`: raw JSON for a Google service account with Google Play Developer API access

## Workflow Behavior

The release workflow lives at `.github/workflows/android-release.yml`.

- Tag push `android-v*`: builds a signed `.aab` and uploads it as a GitHub Actions artifact.
- Manual dispatch: builds a signed `.aab`, and can optionally upload it to Google Play.
- Manual releases must be run from `master` or a vetted `android-v*` tag.

Manual-dispatch inputs:

- `version_name`: optional override for the Android `versionName`
- `version_code`: optional override for the Android `versionCode`
- `upload_to_play`: when `true`, publish the built bundle to Google Play
- `play_track`: Play track to target, default `internal`
- `play_status`: Play release status, default `draft`

Safety gates:

- `production` or `completed` Play uploads must be run from an `android-v*` tag.
- Tag pushes only build and upload the signed bundle artifact; Play upload stays manual.

## Google Play Prerequisites

- The package name must already exist in Play Console: `com.gregorycarnegie.ironinsights`
- The Google Play Developer API must be enabled for the Google Cloud project tied to the service account
- The service account must be invited in Play Console and granted app-level release permissions

## First Safe Release Flow

1. Configure the `android-release` environment secrets.
2. Run `Android Release` from `master` with `upload_to_play=false` and confirm the signed `.aab` artifact.
3. Run `Android Release` from `master` with `upload_to_play=true`, `play_track=internal`, and `play_status=draft`.
4. Verify the draft/internal release in Play Console before creating an `android-v*` tag for broader rollout.

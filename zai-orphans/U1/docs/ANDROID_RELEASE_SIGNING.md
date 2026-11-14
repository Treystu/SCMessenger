# Android Release Signing Setup

Status: Ready to configure (release.yml already wired, needs one-time keystore setup)

## Why This Matters -- Read Before Generating Anything

Whatever key signs the **first** release uploaded to a given Play Store app
listing is the **only** key that can ever sign updates to that listing.
There is no recovery path from Google if it's lost -- you cannot re-verify
ownership and get a new key issued for an existing listing. Losing this
keystore means the app can never be updated again under this package name
(`com.scmessenger.android`); the only fallback is publishing as a brand-new
listing, which throws away all reviews/install history/ratings.

**Before generating the keystore, decide your backup plan.** At minimum:
- The `.jks` file backed up in two places you control (e.g. a password
  manager's file-attachment feature, plus an encrypted drive/backup)
- The store password, key alias, and key password saved in a password
  manager -- NOT in a plain text file, NOT in this repo, NOT pasted into
  any chat (including this one)

## Step 1: Generate the Keystore (run this yourself, not through an agent)

```bash
keytool -genkeypair -v \
  -keystore scmessenger-release.jks \
  -alias scmessenger \
  -keyalg RSA \
  -keysize 2048 \
  -validity 10000 \
  -storetype JKS
```

`keytool` will interactively prompt for:
- A keystore password (this becomes `SCMESSENGER_KEYSTORE_PASSWORD`)
- Your name/org details for the certificate (cosmetic, shown in the cert, not security-sensitive)
- A key password (press Enter to reuse the keystore password, or set a
  separate one -- this becomes `SCMESSENGER_KEY_PASSWORD`)

`-validity 10000` is roughly 27 years -- Play Store requires the signing
cert to remain valid through the year 2033 minimum; this comfortably clears
that with margin.

The alias `scmessenger` becomes `SCMESSENGER_KEY_ALIAS` below (change it if
you prefer, just keep it consistent with what you set as the secret).

**Immediately back up `scmessenger-release.jks`** (see the note above)
before doing anything else with it.

## Step 2: Base64-Encode the Keystore

```bash
# Windows (PowerShell):
[Convert]::ToBase64String([IO.File]::ReadAllBytes("scmessenger-release.jks")) | Set-Content -NoNewline scmessenger-release.b64

# macOS/Linux:
base64 -w0 scmessenger-release.jks > scmessenger-release.b64
```

## Step 3: Set the 4 GitHub Repo Secrets

Using `gh` (run these yourself from a terminal where you can see the
values momentarily -- avoid pasting the actual password strings into any
chat, including this one):

```bash
gh secret set SCMESSENGER_KEYSTORE_BASE64 < scmessenger-release.b64
gh secret set SCMESSENGER_KEYSTORE_PASSWORD   # paste the keystore password when prompted
gh secret set SCMESSENGER_KEY_ALIAS           # paste "scmessenger" (or your chosen alias)
gh secret set SCMESSENGER_KEY_PASSWORD        # paste the key password
```

`gh secret set NAME` (no `<`) prompts interactively and reads from stdin
without echoing it to the terminal history -- safer than putting the value
directly on the command line where it could land in shell history.

Or via the GitHub web UI: repo -> Settings -> Secrets and variables ->
Actions -> New repository secret, same 4 names.

## Step 4: Delete the Local Plaintext Copies

Once the secrets are set:

```bash
rm scmessenger-release.b64
# Keep scmessenger-release.jks itself -- that's your backup copy, just
# make sure it's also saved somewhere OTHER than this working directory
# (password manager attachment, encrypted external backup, etc.)
```

## What Happens Next

Once all 4 secrets exist, the next `v*` tag push triggers `release.yml`'s
`build-android` job to also produce:
- `android/app/build/outputs/bundle/release/*.aab` -- upload this to Play
  Console (internal testing track, then production when ready)
- `android/app/build/outputs/apk/release/*.apk` -- a signed APK, useful for
  direct/sideload distribution (e.g. handing a build straight to an alpha
  tester without waiting on Play Store review)

Both get attached to the resulting GitHub Release automatically (the
`create-release` job's file glob already includes `**/*.aab` and
`**/*.apk`).

If the secrets are NOT set, `build-android` still succeeds -- it just
produces the debug APK only, same as it does today. The signed-build steps
are conditional (`if: secrets.SCMESSENGER_KEYSTORE_BASE64 != ''`), so
there's no way to accidentally break CI by not having gotten to this setup
yet.

## Play Store Upload (Manual, By Design)

Per the "CI builds a signed AAB, you upload manually" choice: this repo
does NOT auto-publish to Play Store. You download the `.aab` from the
GitHub Release and upload it to Play Console yourself each time. This
keeps Play Store publishing credentials (a service account JSON with
publish rights) out of GitHub Secrets entirely, and keeps a manual human
gate before anything reaches real users via the Play Store.

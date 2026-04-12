#!/bin/bash
set -euo pipefail

# ─────────────────────────────────────────────────────────
# Monochrome Android Build Script
# Pulls latest from GitHub, applies Android patches, builds APK
#
# Patches applied temporarily during build:
#   - index.html: script tag, viewport-fit, brand name, YTM UI
#   - js/*.js: YTM API and provider integration
#   - package.json: Capacitor dependencies
# All reverted after build. Git stays clean.
# ─────────────────────────────────────────────────────────

PROJECT_DIR="$(cd "$(dirname "$0")" && pwd)"
APK_OUTPUT="$PROJECT_DIR/android/app/build/outputs/apk/debug/app-debug.apk"
APK_COPY="$PROJECT_DIR/Monochrome-debug.apk"

# These are usually set in the environment, but provided here as fallbacks for Mac
export JAVA_HOME=${JAVA_HOME:-/opt/homebrew/opt/openjdk@21/libexec/openjdk.jdk/Contents/Home}
export ANDROID_HOME=${ANDROID_HOME:-/opt/homebrew/share/android-commandlinetools}
export ANDROID_SDK_ROOT=${ANDROID_SDK_ROOT:-$ANDROID_HOME}
export PATH=$JAVA_HOME/bin:$ANDROID_HOME/platform-tools:$ANDROID_HOME/cmdline-tools/latest/bin:$PATH

cd "$PROJECT_DIR"

cleanup() {
    echo ""
    echo "▶ Cleaning up patched files..."
    git checkout -- index.html package.json package-lock.json js/app.js js/settings.js js/music-api.js js/storage.js 2>/dev/null || true
    rm -f js/android-service.js js/ytm-api.js
    echo "  ✓ Source restored to upstream."
}
trap cleanup EXIT

echo "══════════════════════════════════════════"
echo "  Monochrome Android Build"
echo "══════════════════════════════════════════"

# ── 1. Pull latest ──
echo ""
echo "▶ Pulling latest from upstream/main..."
cleanup 2>/dev/null || true
git fetch upstream
LOCAL=$(git rev-parse HEAD)
REMOTE=$(git rev-parse upstream/main)

if [ "$LOCAL" = "$REMOTE" ]; then
    echo "  Already up to date. Building anyway."
else
    echo "  $(git rev-list --count HEAD..upstream/main) new commits."
    git pull upstream main
    echo "  ✓ Updated."
fi

# ── 2. Install deps + add Capacitor ──
echo ""
echo "▶ Installing dependencies..."
npm install --silent 2>/dev/null
npm install --save @capacitor/core @capacitor/cli @capacitor/android @capacitor/status-bar 2>/dev/null
echo "  ✓ Done."

# ── 3. Patches ──
echo ""
echo "▶ Patching for Android build..."

# 3a. Copy overlay files from storage
OVERLAY_STORAGE="$PROJECT_DIR/android/overlay"

# Copy android-service.js (main bridge)
cp "$PROJECT_DIR/android/android-service.js" js/android-service.js

# Copy specialized JS files (YTM integration, etc.)
if [ -d "$OVERLAY_STORAGE/js" ]; then
    cp "$OVERLAY_STORAGE/js/"*.js js/
fi

# Replace index.html with modified version if available
if [ -f "$OVERLAY_STORAGE/index.html.modified" ]; then
    cp "$OVERLAY_STORAGE/index.html.modified" index.html
fi

# 3b. Add viewport-fit=cover if not already present
if ! grep -q "viewport-fit=cover" index.html; then
    sed -i 's|initial-scale=1.0"|initial-scale=1.0, viewport-fit=cover, maximum-scale=1.0, user-scalable=no"|' index.html
fi

# 3c. Brand: "Monochrome" → "Monochrome Music" in sidebar logo if not already present
sed -i 's|<span>Monochrome</span>|<span>Monochrome Music</span>|' index.html 2>/dev/null || true

echo "  ✓ Source patched with Android-specific UI and features."

# ── 5. Init Capacitor Android if needed ──
if [ ! -d "$PROJECT_DIR/android/app" ]; then
    npx cap add android 2>/dev/null
    echo "  ✓ Android platform added."
fi

# ── 6. Build web ──
echo ""
echo "▶ Building web app..."
npx vite build 2>&1 | tail -3
echo "  ✓ Web build complete."

# ── 7. Sync to Android ──
echo ""
echo "▶ Syncing to Android..."
npx cap sync android 2>&1 | tail -2
echo "  ✓ Synced."

# ── 7b. Fix duplicate splash resources ──
if [ -f "$PROJECT_DIR/android/app/src/main/res/drawable/splash.png" ] && \
   [ -f "$PROJECT_DIR/android/app/src/main/res/drawable/splash.xml" ]; then
    rm "$PROJECT_DIR/android/app/src/main/res/drawable/splash.png"
    echo "  ✓ Removed duplicate splash.png (keeping splash.xml)."
fi

# ── 8. Build APK ──
echo ""
echo "▶ Building APK..."
cd "$PROJECT_DIR/android"
./gradlew assembleDebug -q
cd "$PROJECT_DIR"

if [ -f "$APK_OUTPUT" ]; then
    cp "$APK_OUTPUT" "$APK_COPY"
    SIZE=$(du -h "$APK_COPY" | cut -f1)
    echo "  ✓ APK built ($SIZE)"
    echo ""
    echo "══════════════════════════════════════════"
    echo "  APK: $APK_COPY"
    echo "══════════════════════════════════════════"
else
    echo "  ✗ Build failed!"
    exit 1
fi

# cleanup runs automatically via trap EXIT

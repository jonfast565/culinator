# OCR and AI recipe import

The Vue app accepts multiple recipe-page images using a file input with `capture="environment"`, which opens the rear camera on supported mobile platforms and the image picker elsewhere. Images travel only over the authenticated loopback WebSocket.

The backend first attempts local OCR through the configured Tesseract executable. If OCR is unavailable or disabled, the OpenAI interpreter receives the images directly. The interpreter returns Culinator 0.3 DSL, which is parsed and validated before being offered to the user.

Settings are stored in the Tauri application-data directory as `settings.json`. The API key is never returned to the WebView; the UI only receives `apiKeyConfigured`. For production, migrating the key to the operating-system keychain remains recommended.

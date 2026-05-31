.PHONY: build release app clean run

BINARY = target/release/kimi-code-status
APP_DIR = "Kimi Code Status.app"
APP_CONTENTS = $(APP_DIR)/Contents
APP_MACOS = $(APP_CONTENTS)/MacOS
APP_RESOURCES = $(APP_CONTENTS)/Resources
ICON_SRC = icons/statusbar_template.png

build:
	cargo build

release:
	cargo build --release

app: release
	rm -rf $(APP_DIR)
	mkdir -p $(APP_MACOS) $(APP_RESOURCES)
	cp $(BINARY) $(APP_MACOS)/
	cp $(ICON_SRC) $(APP_RESOURCES)/statusbar_template.png
	cat > $(APP_CONTENTS)/Info.plist << 'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key>
    <string>Kimi Code Status</string>
    <key>CFBundleDisplayName</key>
    <string>Kimi Code Status</string>
    <key>CFBundleIdentifier</key>
    <string>io.ccswitch.kimi-code-status</string>
    <key>CFBundleVersion</key>
    <string>0.1.0</string>
    <key>CFBundleExecutable</key>
    <string>kimi-code-status</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>LSMinimumSystemVersion</key>
    <string>13.0</string>
    <key>LSUIElement</key>
    <true/>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
PLIST
	@echo "App bundle created: $(APP_DIR)"

run: release
	$(BINARY)

clean:
	cargo clean
	rm -rf "$(APP_DIR)"

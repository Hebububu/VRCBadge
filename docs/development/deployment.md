# Deployment

## Deploy Script

```bash
#!/bin/bash
# deploy.sh
set -e

TARGET="aarch64-unknown-linux-gnu"
BADGE_HOST="pi@badge.local"
BINARY_NAME="digital-badge"

echo "Building..."
cargo zigbuild --target $TARGET --release

echo "Deploying..."
scp target/$TARGET/release/$BINARY_NAME $BADGE_HOST:~/

echo "Running..."
ssh $BADGE_HOST "sudo ./digital-badge"
```

## Systemd Service

Create `/etc/systemd/system/badge.service`:

```ini
[Unit]
Description=Digital Badge
After=network.target

[Service]
ExecStart=/home/pi/digital-badge
Restart=always
User=root
Environment=DISPLAY=:0
WorkingDirectory=/home/pi

[Install]
WantedBy=multi-user.target
```

Enable and start:

```bash
sudo systemctl enable badge
sudo systemctl start badge
```

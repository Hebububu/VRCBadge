# API Endpoints

All endpoints are served by the esp-idf-svc HTTP server on port 80 over the badge's WiFi AP.

## Implemented Endpoints

| Method | Endpoint | Description |
| ------ | -------- | ----------- |
| GET | /api/health | Simple healthcheck (returns `OK`) |
| GET | /api/profile | Get current profile as JSON |
| POST | /api/profile | Update profile from JSON |
| POST | /api/avatar | Upload avatar image |
| POST | /api/background | Upload background image |
| DELETE | /api/background | Clear background (revert to solid color) |

### GET /api/health

Returns `OK` with status 200. Used to verify the server is running.

### GET /api/profile

Returns the current profile as JSON:

```json
{
  "display_name": "Hebu",
  "tagline": "Hello from VRCBadge!",
  "twitter_handle": "@Hebu_VRC",
  "discord_handle": "hebu",
  "background_color": "#1a1a2e",
  "tagline_color": "#e0e8f0",
  "tagline_background_color": "#1b4f72"
}
```

### POST /api/profile

Update the profile. Request body is JSON (max 4KB), same schema as the GET response. The main loop picks up the update and saves to NVS within ~2 seconds.

### POST /api/avatar

Upload a new avatar image. The request body must be **exactly 67,500 bytes** of raw RGB888 pixel data (150 x 150 x 3 bytes). No headers or encoding — just raw bytes with `Content-Length: 67500`.

The browser SPA handles resizing and RGBA-to-RGB conversion client-side before upload.

### POST /api/background

Upload a new background image. The request body must be **exactly 460,800 bytes** of raw RGB888 pixel data (480 x 320 x 3 bytes). Same raw format as avatar.

### DELETE /api/background

Clears the background image and reverts the badge display to its solid background color. Deletes the saved image from SPIFFS.

## Planned Endpoints

| Method | Endpoint | Description |
| ------ | -------- | ----------- |
| GET | /api/status | System status (battery %, uptime, WiFi info) |

## Future Endpoints (RFID)

| Method | Endpoint | Description |
| ------ | -------- | ----------- |
| GET | /api/rfid/slots | List stored cards |
| POST | /api/rfid/slot/:id | Activate card slot |

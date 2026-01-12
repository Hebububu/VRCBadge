# API Endpoints

## Current Endpoints

| Method | Endpoint       | Description                          |
| ------ | -------------- | ------------------------------------ |
| POST   | /api/avatar    | Upload new avatar image              |
| POST   | /api/profile   | Update name, tagline, socials        |
| GET    | /api/status    | Battery %, WiFi strength, uptime     |
| GET    | /api/health    | Simple healthcheck                   |

## Future Endpoints (RFID)

| Method | Endpoint            | Description                |
| ------ | ------------------- | -------------------------- |
| GET    | /api/rfid/slots     | List stored cards          |
| POST   | /api/rfid/slot/:id  | Activate card slot         |

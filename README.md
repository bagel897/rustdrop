# Rustdrop

Reimplementation of Google's Quick Share (originally Nearby Sharing) protocol.

## Demo

[](https://private-user-images.githubusercontent.com/57874654/302790560-3109c779-7cdd-42e1-86b2-7510815be182.webm?jwt=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJnaXRodWIuY29tIiwiYXVkIjoicmF3LmdpdGh1YnVzZXJjb250ZW50LmNvbSIsImtleSI6ImtleTUiLCJleHAiOjE3MDcyNTc1NjksIm5iZiI6MTcwNzI1NzI2OSwicGF0aCI6Ii81Nzg3NDY1NC8zMDI3OTA1NjAtMzEwOWM3NzktN2NkZC00MmUxLTg2YjItNzUxMDgxNWJlMTgyLndlYm0_WC1BbXotQWxnb3JpdGhtPUFXUzQtSE1BQy1TSEEyNTYmWC1BbXotQ3JlZGVudGlhbD1BS0lBVkNPRFlMU0E1M1BRSzRaQSUyRjIwMjQwMjA2JTJGdXMtZWFzdC0xJTJGczMlMkZhd3M0X3JlcXVlc3QmWC1BbXotRGF0ZT0yMDI0MDIwNlQyMjA3NDlaJlgtQW16LUV4cGlyZXM9MzAwJlgtQW16LVNpZ25hdHVyZT0zM2Y0OTljYzEzNzAwZTVlYWQ4ODliODUyY2I1OTg1ZGQ3ZTdiOWRmNmY2ODYxZDg2MzY5MjM0NWRhYjZhOGI1JlgtQW16LVNpZ25lZEhlYWRlcnM9aG9zdCZhY3Rvcl9pZD0wJmtleV9pZD0wJnJlcG9faWQ9MCJ9.06b8g_Q8rtO7EdwkDPbK9mgftH5r60u4u9KNqDlqMec)

## Building

- Needs to be built with tokio unstable. The provided cargo config should do this, but be sure to unset `RUSTFLAGS`

## Features

### Mediums

- WLAN
- BT (very WIP)

### Discovery

- Mdns
- BLE (partial)

## Troubleshooting

### Wlan/Mdns

- Disable firewalls/non-wifi networks (esp vpns)

## Credits

- [NearDrop](https://github.com/grishka/NearDrop) - Protocol documentation
- [QNearbyShare](https://github.com/vicr123/QNearbyShare) - Debugging
- [Nearby](https://github.com/google/nearby) - Original protocol.
- [Simple Nearby](https://github.com/alex9099/SimpleNearby) - Bluetooth.

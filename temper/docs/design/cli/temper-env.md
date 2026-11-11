# temper: environment

Generated from temper's table (`skein/docs/design/runtime/shell.md`, 14.3): a test checks it byte for byte, and the tree's rewrite switch rewrites it.

## Public

## Internal

### `TEMPER_INTERNAL_SITE`

- **Set:** internal
- **Reads**
- **Form:** ORIGIN
- **Absent:** Use https://fabrig.ai (port 443).
- Loopback HTTPS origin for a test's temper-box download.
- **Notes:** Only https://127.0.0.1:PORT, https://[::1]:PORT or https://localhost:PORT, with nothing after the port.

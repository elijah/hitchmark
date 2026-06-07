# DBus Interface

On Linux, `hookmarks-daemon` exposes a session bus service.

## Service details

| Property | Value |
|----------|-------|
| Bus | Session bus |
| Service name | `org.not_hookmarks.Daemon` |
| Object path | `/org/not_hookmarks/Daemon` |
| Interface | `org.not_hookmarks.Daemon1` |

## Methods

### `OpenUri(uri: String) → String`

Resolve a `hook://` URI and open the target with `xdg-open`.

```bash
gdbus call --session \
  --dest org.not_hookmarks.Daemon \
  --object-path /org/not_hookmarks/Daemon \
  --method org.not_hookmarks.Daemon1.OpenUri \
  'hook://file/L1VzZXJzL2Vsd...'
```

Returns: `"Opened: /path/to/file"` on success.

Errors:
- `org.freedesktop.DBus.Error.InvalidArgs` — invalid URI
- `org.freedesktop.DBus.Error.Failed` — xdg-open failed

### `CreateLink(uri_a: String, uri_b: String, note: String) → String`

Create a bidirectional link. Pass an empty string for `note` if not needed.

```bash
gdbus call --session \
  --dest org.not_hookmarks.Daemon \
  --object-path /org/not_hookmarks/Daemon \
  --method org.not_hookmarks.Daemon1.CreateLink \
  'hook://file/L1Zvby9...' 'hook://file/L3Jhc...' 'See section 4'
```

Returns: `"Link created: <uri_a> <-> <uri_b>"`

### `ListLinks(uri: String) → Array<String>`

Return all bidirectional links for a URI as an array of tab-separated strings.

Each element: `source\ttarget\tnote`

```bash
gdbus call --session \
  --dest org.not_hookmarks.Daemon \
  --object-path /org/not_hookmarks/Daemon \
  --method org.not_hookmarks.Daemon1.ListLinks \
  'hook://file/L1Zvby9...'
```

### `FileToUri(path: String) → String`

Convert an absolute file path to a `hook://` URI.

```bash
gdbus call --session \
  --dest org.not_hookmarks.Daemon \
  --object-path /org/not_hookmarks/Daemon \
  --method org.not_hookmarks.Daemon1.FileToUri \
  '/home/you/docs/project.md'
# → ('hook://file/L2hvbWUveW91L2RvY3MvcHJvamVjdC5tZA==',)
```

## Introspect

```bash
gdbus introspect --session \
  --dest org.not_hookmarks.Daemon \
  --object-path /org/not_hookmarks/Daemon
```

## Python example

```python
import dbus

session = dbus.SessionBus()
proxy = session.get_object(
    "org.not_hookmarks.Daemon",
    "/org/not_hookmarks/Daemon"
)
iface = dbus.Interface(proxy, "org.not_hookmarks.Daemon1")

# Get URI for a file
uri = iface.FileToUri("/home/you/docs/project.md")
print(uri)  # hook://file/...

# Create a link
iface.CreateLink(uri, "hook://file/L3Jhc...", "")

# List links
links = iface.ListLinks(uri)
for link in links:
    print(link)
```

## Checking the daemon is running

```bash
# Via systemd
systemctl --user is-active hookmarks-daemon

# Via DBus
gdbus call --session \
  --dest org.freedesktop.DBus \
  --object-path /org/freedesktop/DBus \
  --method org.freedesktop.DBus.NameHasOwner \
  'org.not_hookmarks.Daemon'
# → (true,) if running
```

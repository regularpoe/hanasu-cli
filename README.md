# hanasu

Service that binds to socket, lives in the background, and is only triggered by specific keywords.

## Prerequisites

```bash
apt install libasound2-dev
```

## Use

### Build and install

```bash
make all
```

### Enable and start service

```bash
make enable_service

systemctl enable hanasu.service
```

### Uninstall service

```bash
make uninstall
```

### Interact

```bash
echo "wake" | socat - UNIX-CONNECT:/tmp/hanasu_socket
```

### Debug

```bash
journalctl -u hanasu.service -f
```


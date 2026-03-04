# UDP Tracker Verification

**Status**: ✅ Verified (2026-03-04)

## Endpoints

| Domain        | URL                                        |
| ------------- | ------------------------------------------ |
| UDP Tracker 1 | `udp://udp1.torrust-tracker-demo.com:6969` |
| UDP Tracker 2 | `udp://udp2.torrust-tracker-demo.com:6868` |

## 1. Port Connectivity

Check that the UDP ports are open and reachable.

> **Note**: `nc -u -z` is unreliable for UDP — there is no handshake to confirm
> the port is open. Use the BEP 15 script below as the real connectivity test.

```bash
# Test port 6969
nc -u -z -w 3 udp1.torrust-tracker-demo.com 6969 && echo "port 6969 open" || echo "port 6969 closed"

# Test port 6868
nc -u -z -w 3 udp2.torrust-tracker-demo.com 6868 && echo "port 6868 open" || echo "port 6868 closed"
```

## 2. UDP Tracker Protocol Test (BEP 15)

The UDP tracker protocol (defined in
[BEP 15](https://www.bittorrent.org/beps/bep_0015.html)) requires a two-step
handshake: a connection request followed by an announce/scrape request.

Use this Python script to perform a full connection handshake against both
trackers:

```python
import socket
import struct
import random

for host, port, label in [
    ("udp1.torrust-tracker-demo.com", 6969, "UDP Tracker 1 (port 6969)"),
    ("udp2.torrust-tracker-demo.com", 6868, "UDP Tracker 2 (port 6868)"),
]:
    transaction_id = random.randint(0, 0xFFFFFFFF)
    packet = struct.pack(">QII", 0x41727101980, 0, transaction_id)
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    sock.settimeout(5)
    try:
        sock.sendto(packet, (host, port))
        data, _ = sock.recvfrom(16)
        action, resp_tid, connection_id = struct.unpack(">IIQ", data)
        assert action == 0, f"unexpected action: {action}"
        assert resp_tid == transaction_id, "transaction ID mismatch"
        print(f"✅ {label}: Connected! connection_id = {connection_id:#018x}")
    except socket.timeout:
        print(f"❌ {label}: Timeout — no response")
    except Exception as e:
        print(f"❌ {label}: Error — {e}")
    finally:
        sock.close()
```

Expected output:

```text
✅ UDP Tracker 1 (port 6969): Connected! connection_id = 0x<16-digit hex value>
✅ UDP Tracker 2 (port 6868): Connected! connection_id = 0x<16-digit hex value>
```

## 3. Using a BitTorrent Client

The most realistic verification is to add a torrent to a BitTorrent client
(e.g. qBittorrent, Transmission, Deluge) and configure it to use one of the
tracker URLs. The client will send a UDP announce and the tracker will respond
with a peer list.

Example magnet link using both UDP trackers:

```text
magnet:?xt=urn:btih:0000000000000000000000000000000000000000
  &tr=udp://udp1.torrust-tracker-demo.com:6969/announce
  &tr=udp://udp2.torrust-tracker-demo.com:6868/announce
```

Note: a real info hash is needed for a meaningful announce. The zero hash above
will return an empty peer list but confirms the tracker is reachable.

## Results

| Check                        | Result | Notes                                |
| ---------------------------- | ------ | ------------------------------------ |
| UDP port 6969 open           | ✅     | BEP 15 handshake succeeded           |
| UDP port 6868 open           | ✅     | BEP 15 handshake succeeded           |
| BEP 15 handshake (tracker 1) | ✅     | `connection_id = 0x927bc33b3260b795` |
| BEP 15 handshake (tracker 2) | ✅     | `connection_id = 0x59c13493038e3be3` |

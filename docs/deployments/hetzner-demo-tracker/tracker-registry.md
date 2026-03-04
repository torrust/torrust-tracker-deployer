# Tracker Registry

Public tracker registries list open BitTorrent trackers so clients can
discover and use them. Submitting the demo tracker improves community
visibility and provides a passive uptime signal.

## newTrackon

[newTrackon](https://newtrackon.com/) continuously monitors the health of
submitted trackers and publishes them in public lists.

The previous Torrust demo tracker (`udp://tracker.torrust-demo.com:6969/announce`)
was already listed there. The new Hetzner demo tracker should be submitted as
well.

### Which tracker to submit

Only **UDP Tracker 1** is submitted to public registries:

```text
udp://udp1.torrust-tracker-demo.com:6969/announce
```

**UDP Tracker 2** (`udp://udp2.torrust-tracker-demo.com:6868/announce`) is
intentionally kept off all public tracker lists. Once a tracker appears in
public lists it receives a continuous stream of announces from BitTorrent
clients worldwide. That background noise makes it very hard to read logs
and debug issues when testing something in production. Keeping `udp2` quiet
reserves it as a low-traffic endpoint for manual testing and investigation.

### How to submit

1. Go to <https://newtrackon.com/>
2. Paste `udp://udp1.torrust-tracker-demo.com:6969/announce` into the submission box
3. Click **Submit**
4. Wait a few minutes while newTrackon gathers data
5. Verify it appears in the [Submitted](https://newtrackon.com/submitted) section

### Status

✅ Submitted (2026-03-04) — pending appearance in the public list.

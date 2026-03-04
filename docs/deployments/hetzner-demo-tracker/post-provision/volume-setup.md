# Volume Setup

> **Status**: ⏳ Pending

Create a Hetzner volume and mount it at `/opt/torrust/storage` on the server before running
`configure`.

## Why a Separate Volume?

The server's root disk contains the OS and application binaries. Persistent tracker data
(database, logs, Grafana state, Prometheus data) lives under `/opt/torrust/storage/`.

Putting that data on a separate Hetzner volume means:

- **Targeted backups**: back up only the volume, not the entire server.
- **Easy migration**: detach the volume and reattach to a new server if the VM is recreated.
- **Independent lifecycle**: you can snapshot or resize the volume without touching the server.

## Volume Specification

| Property    | Value                                                 |
| ----------- | ----------------------------------------------------- |
| Name        | `torrust-tracker-demo-storage`                        |
| Size        | 50 GB                                                 |
| Location    | `nbg1` (same as the server — required for attachment) |
| Format      | `ext4`                                                |
| Mount point | `/opt/torrust/storage`                                |

## Step 1: Create the Volume in Hetzner Console

In the [Hetzner Console](https://console.hetzner.cloud/):

1. Open the project `torrust-tracker-demo.com`.
2. Go to **Storage → Volumes → Create Volume**.
3. Set:
   - **Name**: `torrust-tracker-demo-storage`
   - **Size**: `50 GB`
   - **Location**: `nbg1`
   - **Format**: leave unformatted (we will format manually below)
   - **Server**: select `torrust-tracker-vm-torrust-tracker-demo` to attach immediately
4. Click **Create & Attach**.

## Step 2: Find the Device Path on the Server

SSH into the server:

```bash
ssh -i ~/.ssh/torrust_tracker_deployer_ed25519 torrust@46.225.234.201
```

List block devices to find the new volume:

```bash
lsblk
```

The Hetzner volume appears as a new disk (not partitioned). You will see something like:

```text
NAME    MAJ:MIN RM  SIZE RO TYPE MOUNTPOINTS
sda     252:0    0   40G  0 disk
├─sda1  252:1    0 39.9G  0 part /
...
sdb     252:16   0   50G  0 disk        ← this is the new volume
```

Hetzner also provides a stable device symlink:

```bash
ls /dev/disk/by-id/ | grep HC_Volume
# Example output: scsi-0HC_Volume_12345678
```

Use the stable path for all subsequent commands (replace with your actual volume ID):

```bash
VOLUME_DEVICE="/dev/disk/by-id/scsi-0HC_Volume_XXXXXXXX"
```

## Step 3: Format the Volume

> **Warning**: This destroys any existing data on the device. Only do this on a brand-new volume.

```bash
sudo mkfs.ext4 -L torrust-storage "$VOLUME_DEVICE"
```

The `-L` flag sets a filesystem label (`torrust-storage`) which can be used in `fstab` as an
alternative to UUID.

Verify:

```bash
sudo blkid "$VOLUME_DEVICE"
# Expected: ... TYPE="ext4" LABEL="torrust-storage" ...
```

Note the UUID from the output — you will need it for `fstab`.

## Step 4: Create the Mount Point

```bash
sudo mkdir -p /opt/torrust/storage
```

## Step 5: Mount the Volume

Mount once manually to verify it works:

```bash
sudo mount "$VOLUME_DEVICE" /opt/torrust/storage
```

Confirm it is mounted:

```bash
df -h /opt/torrust/storage
# Expected:
# Filesystem      Size  Used Avail Use% Mounted on
# /dev/sdb         49G   24K   47G   1% /opt/torrust/storage
```

## Step 6: Make the Mount Persistent (fstab)

Get the UUID of the volume:

```bash
sudo blkid -s UUID -o value "$VOLUME_DEVICE"
# Example output: a1b2c3d4-e5f6-7890-abcd-ef1234567890
```

Add an entry to `/etc/fstab` using the UUID (replace with your actual UUID):

```bash
echo "UUID=<your-uuid>  /opt/torrust/storage  ext4  defaults,nofail  0  2" \
  | sudo tee -a /etc/fstab
```

The `nofail` option prevents the server from failing to boot if the volume is temporarily
unavailable. The `0 2` means: no dump, fsck on boot after root filesystem.

Verify the fstab entry is correct by simulating a remount:

```bash
sudo mount -a
df -h /opt/torrust/storage
```

## Step 7: Set Correct Ownership

The `configure` command will create subdirectories under `/opt/torrust/storage/` via Ansible
(running as the `torrust` user). Set ownership in advance:

```bash
sudo chown -R torrust:torrust /opt/torrust/storage
```

## Step 8: Verify (Full Check)

```bash
# Volume is mounted
mountpoint -q /opt/torrust/storage && echo "Mounted ✓" || echo "NOT mounted"

# Correct ownership
ls -la /opt/torrust/ | grep storage

# Write test
touch /opt/torrust/storage/.volume-test && echo "Write OK ✓" && rm /opt/torrust/storage/.volume-test
```

## Outcome

Once the volume is mounted at `/opt/torrust/storage` and owned by `torrust`, you can proceed
to the `configure` command.

## Problems

<!-- Document any issues encountered during volume setup here. -->

## Improvements

<!-- Document any recommended improvements here. -->

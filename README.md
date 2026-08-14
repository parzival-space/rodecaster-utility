## RODECaster Utility

A configuration utility for the RODECaster Pro II.
This repository is an attempt at reverse engineering the RODECaster Pro II's USB protocol, and creating a
Linux compatible tool to manage the device's configuration.

### Linux udev rules setup

Install the bundled udev rules so non-root users can access the device:

```bash
sudo cp 50-rodecaster-pro-ii.rules /etc/udev/rules.d/50-rodecaster-pro-ii.rules
sudo udevadm control --reload-rules
sudo udevadm trigger --subsystem-match=hidraw --action=add
```

The final `udevadm trigger` reapplies rules to currently connected `hidraw` devices, so you usually do not
need to unplug/replug the RODECaster after installing or updating the rules.

### Virtual Devices

If you want to use the virtual devices, this is the wrong repository.  
You can set up virtual devices by either using
my [Virtual Devices Configuration for PipeWire](https://github.com/parzival-space/rodecaster-pro-2-virtual-devices-pipewire)
or by manually adding
the [Alsa UCM Configuration](https://github.com/parzival-space/alsa-ucm-conf/tree/rode/rodecaster-pro-ii) on your local
system.

This shouldn't be necessary in the future once [this PR](https://github.com/alsa-project/alsa-ucm-conf/pull/656) has
been merged.

TODO: Update README

### License

The entire repository, except the

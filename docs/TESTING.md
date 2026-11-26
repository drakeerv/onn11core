# Testing

## Current tablet, already unlocked and imaged

These tests are read-only except where noted. Use the current tablet to validate
the wizard plumbing before the new tablet run. Hardware result so far: the stock
fastbootd rejects physical partition writes, so physical images must use LK
fastboot. fastbootd is for the system image and the final wipe.

1. State probe from the running GSI.
   - Run `onn11core probe`.
   - Record model, slot, lock state, fingerprint, and LineageOS version.

2. Recovery adb probe.
   - Power off, hold Volume Up, plug in USB.
   - Run `onn11core probe`.
   - Confirm recovery adb is detected.
   - Run `adb reboot fastboot` and `onn11core probe`.
   - Confirm fastbootd is detected with slot and unlock state.

3. LK physical writes.
   - Reboot to LK fastboot and start the first flash immediately. The command
     waits for the tablet. No unplug is needed unless it was started late.
   - Flash the same stock vbmeta with verity disabled, stock init_boot, stock
     dtbo, and the GKI boot.
   - Reboot and verify `uname -r`, Wi-Fi, Bluetooth, touch, and display.
   - Revert the stock boot if the GKI fails.
   - In fastbootd, only system, product, resize, delete, wipe, and reboot are
     expected to work. Record any physical write that succeeds there too.

4. Optional root test.
   - Use the stock init_boot and the current GSI.
   - Run the Magisk flow in fastbootd.
   - Verify the Magisk app, `su`, and a reboot.

5. Do not run a full wizard install on the current tablet unless a wipe is
   acceptable. The full flash and final factory reset are for the new tablet.

## New tablet, full camera run

1. Power off and unplug the new tablet.
2. Run `onn11core`.
3. When it waits for LK fastboot, plug the tablet in while holding Volume Up +
   Volume Down. The tool reads and validates LK before unlocking.
4. Choose microG.
5. Root: no for the first run, then test separately.
6. Backup: no for speed. Note the stock dump is model-level.
7. Confirm the active slot and the stock other slot.
8. At the unlock prompt, unplug when asked, wait for the unlocker, then plug the
   tablet back in.
9. Watch the unlock, recovery adb return, LK physical writes, fastbootd system
   write, GKI, and final factory reset. On the Try again / Factory data reset
   screen choose Factory data reset with the volume keys and Power, then
   confirm. Do not choose Try again.
10. Verify boot, slot, lock state, LineageOS version, GKI `uname -r`, Wi-Fi,
    Bluetooth, touch, display, cameras, charging, and sleep.
11. If the GKI fails, report the fallback to the stock boot.
12. Record any LK first-command failures and any fastbootd physical write results.

## Pass criteria

- Recovery adb is detected and stable.
- Raw unlock returns to recovery adb.
- LK writes vbmeta, boot, init_boot, and dtbo.
- fastbootd writes the system image.
- The GSI boots with the GKI and stock vendor modules.
- The other slot remains stock.
- Optional root works through the LK init_boot flash.

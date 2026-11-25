# Testing

## Current tablet, already unlocked and imaged

These tests are read-only except where noted. Use the current tablet to validate
the wizard plumbing and the fastbootd assumption before the new tablet run.

1. State probe from the running GSI.
   - Run `onn11core probe`.
   - Record model, slot, lock state, fingerprint, and LineageOS version.

2. Recovery adb probe.
   - Power off, hold Volume Up, plug in USB.
   - Run `onn11core probe`.
   - Confirm recovery adb is detected.
   - Run `adb reboot fastboot` and `onn11core probe`.
   - Confirm fastbootd is detected with slot and unlock state.

3. Fastbootd physical writes.
   - From fastbootd, record `fastboot getvar is-userspace` and `current-slot`.
   - Flash the same stock vbmeta with verity disabled:
     `fastboot --disable-verity --disable-verification flash vbmeta_<slot> stock/vbmeta_<slot>.img`
   - Flash the same stock init_boot and dtbo:
     `fastboot flash init_boot_<slot> stock/init_boot_<slot>.img`
     `fastboot flash dtbo_<slot> stock/dtbo_<slot>.img`
   - Flash the GKI boot:
     `fastboot flash boot_<slot> boot_b_gki.img`
   - Reboot and verify `uname -r`, Wi-Fi, Bluetooth, touch, and display.
   - If any physical write is rejected, stop. The fastbootd-only wizard
     assumption does not hold for that unit.
   - Revert the stock boot if the GKI fails.

4. Optional root test.
   - Use the stock init_boot and the current GSI.
   - Run the Magisk flow in fastbootd.
   - Verify the Magisk app, `su`, and a reboot.

5. Do not run a full wizard install on the current tablet unless a wipe is
   acceptable. The full flash and final factory reset are for the new tablet.

## New tablet, full camera run

1. Power off, hold Volume Up, plug in USB.
2. Run `onn11core`.
3. Choose microG.
4. Root: no for the first run, then test separately.
5. Backup: no for speed. Note the stock dump is model-level.
6. Confirm the active slot and the stock other slot.
7. Watch the unlock, recovery adb return, fastbootd install, GKI, and final
   factory reset.
8. Verify boot, slot, lock state, LineageOS version, GKI `uname -r`, Wi-Fi,
   Bluetooth, touch, display, cameras, charging, and sleep.
9. If the GKI fails, report the fallback to the stock boot.
10. Record any fastbootd physical write failures. Those decide whether the
    no-LK assumption is valid.

## Pass criteria

- Recovery adb is detected and stable.
- Raw unlock returns to recovery adb.
- fastbootd writes system, vbmeta, boot, init_boot, and dtbo.
- The GSI boots with the GKI and stock vendor modules.
- The other slot remains stock.
- Optional root works without LK fastboot.

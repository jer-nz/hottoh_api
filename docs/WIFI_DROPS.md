# Stove won't connect to Wi-Fi or keeps dropping off (firmware 10.1.0 and below)

**CMG, Edilkamin or other pellet stove with the AppFire app, and its Wi-Fi no longer works?**
These brands use the same HottoH Wi-Fi module ("Wifier 2.0", ESP32), so they share the
bug described here as long as the module runs firmware 10.1.0 or below.

If you cannot pair your stove with your Wi-Fi from AppFire, or if it joins the Wi-Fi, stays
reachable for a few seconds, then disappears and comes back a few minutes later, over and over,
and nothing changed on your side, this page is for you. Both are the same problem: the module
does connect, but loses the link after about 20 seconds, too fast for the app to finish the
setup, so most people only see a stove that never connects. This page explains what happens
inside the module, how it was found, and how to fix it.

**Short answer: update the module firmware to 10.5.0 or later with the AppFire app.** Since the
module drops off before an update can finish, first block `www.google.com` for a moment (for the
stove, or for the whole network), update, then unblock it: [step by step](#fix-update-to-firmware-1050).

## Symptoms

- **Pairing from AppFire fails**, or the stove shows up in the app and goes offline right away.
  The Wi-Fi settings are right: the module did join the network, it just did not stay long
  enough.
- The stove connects, answers for about **20 seconds**, then goes silent.
- It comes back by itself, and the cycle repeats **every 5 min 25 s**, day and night.
- The signal is good and the router hands out an address instantly: in the DHCP log of the router,
  the stove asks for an address again at every cycle.
- The router may keep listing the stove as connected for a few minutes after it went silent: trust
  a ping, not the list of Wi-Fi clients.
- Changing the Wi-Fi settings does not help: disabling 5 GHz (the module only uses 2.4 GHz) or
  switching the router to WPA/WPA2 mixed mode gives exactly the same cycle.
- hottoh_api reconnects by itself after each drop, but has no data while the module restarts.

The module firmware version is shown on the **Wi-Fi module** page of hottoh_api
(`GET /api/inf`).

## What happens

The firmware was disassembled to find the cause. Three things combine in version 10.1.0
(built in June 2021), the one that was analysed. Earlier versions, older still, are
expected to behave the same:

1. **An Internet check that downloads a whole web page.** Once connected, a background task of the
   module checks the Internet access with a plain `GET http://www.google.com`, keeps **the whole
   response in memory**, growing its buffer at every chunk, and only then looks for the word
   "google" in it. The Google home page is about **83 KiB** today, far more than what was left
   in the ESP32 heap. Memory runs out in the middle of the download and the Wi-Fi link goes down.
   A network capture shows it clearly: Google keeps retransmitting to a stove that went silent
   mid-transfer.
2. **No Wi-Fi reconnection.** When the link drops, the firmware never tries to connect again.
3. **A restart as the only way out.** The Wi-Fi task restarts the whole module after 300 seconds
   without connection (`No connection timeout -> restart!` in its code).

20 s connected + 300 s waiting + boot time = the 5 min 25 s cycle.

### Why now, after years without trouble?

Most likely because the Google home page keeps growing. Nothing changed in the stove or in your
home: the firmware is from 2021, and back then the page was lighter and still fit in the memory
the ESP32 had left once its Wi-Fi, cloud and Modbus tasks were running. Year after year Google
added scripts, styles and inline images to it, until the download no longer fit. The module did
not break: the page it depends on outgrew it. The exact size at each period was not measured, but
nothing else in the chain moves.

The design is the real problem. A connectivity check is not supposed to download a web page.

## How it was confirmed

Each test only changed the traffic of the stove, nothing else on the network:

| Setup | Result |
|---|---|
| DNS requests of the stove refused | stable |
| Only `www.google.com` blocked for the stove, cloud relay allowed | stable |
| `www.google.com` answered by a local web server with a 71-byte page containing "google" | stable, and the cloud connection goes further |

## Fix: update to firmware 10.5.0

In 10.5.0 (built in October 2024), the background task that ran the Google check and talked to
the HottoH cloud is gone: the check only runs as a test while you set up the Wi-Fi from the app.
A stove updated to 10.5.0 is stable without any workaround.

The catch: **a module stuck in this loop cannot be updated as it is.** It stays online for about
20 seconds at a time, not long enough for AppFire to reach it and for the firmware to download.
Break the loop first:

1. **Block the name `www.google.com`**, so that it no longer resolves. Two options, depending on
   what your network allows:
   - **for the stove only** (cleanest): a DNS server that supports per-client rules (Pi-hole
     groups, dnsmasq, AdGuard Home, some routers) answers nothing or `0.0.0.0` for
     `www.google.com` to the stove's address;
   - **for the whole network**, if you cannot target a single device: the same rule for everyone,
     or `www.google.com` added to the blocklist of your router. Google search stops working on
     every device of the house for a few minutes; only `www.google.com` is needed, not the other
     Google services.
2. **Wait for the next restart of the module** (at most 5 minutes). It then connects and stays
   connected: it cannot run the check, so it cannot exhaust its memory.
3. **Update the firmware from AppFire**, and wait until the module has restarted on the new
   version. hottoh_api shows it on the **Wi-Fi module** page.
4. **Remove the block.** The bug is gone from the new firmware: the module no longer runs this
   check once connected, and Google works again for everyone.

Blocking only `www.google.com` does not cut the stove from the Internet: the HottoH servers used
by the update and by the cloud relay stay reachable.

## Workaround if you cannot update

If the update is not possible, keep `www.google.com` blocked **for the stove only**, as in step 1
above: the module stays connected for good. A gentler variant answers the check instead of
blocking it: a local web server on port 80 serves a tiny page containing the word `google`, for
example `<html><head><title>Google</title></head><body>google.com</body></html>`, and your DNS
server answers `www.google.com` with the address of that server, again for the stove only. The
module then downloads a few bytes instead of 83 KiB and believes it has Internet access.

Updating the firmware remains the real fix: remove the workaround afterwards.

# esp-hello-display :crab:
Wokwi-example how the display is initialised for ESP32C3

## Instructions

### Build

```
cargo build --target riscv32imac-unknown-none-elf --release
```

### Execution with VSCode Wokwi extension  

Except of extension itself, you will need two files to execute the simulatuion inside of your VSCode IDE:
* wokwi.toml 
* diagram.json

Both of them are already provided in this repo and you can edit them as needed.

```
F1 -> Wokwi: Start simulation
```
In case you will have additional questions - feel free to open an issue :wink:

## Description
There's a short example for newcomers that shows, how the display is initialised on different Espressif boards (ESP32, ESP32S2, ESP32C3) in bare-metal <br>
Initially, there were three examples on Wokwi only (without Gitpod), but it requires local builder, what is relatively difficult to explain to newcomer (I'll do a separate repository for this one day). 
<br>
Anyway, we're already working on including [esp-hal](https://github.com/esp-rs/esp-hal) drivers for building it in Wokwi without using local builder, so you need just to wait a little :heart:

>### **Important** : below in this branch you can find pin connection for REAL hardware, not for Wokwi. Pin connection for every chip for Wokwi can be found in corresponding branches

## Demonstration
### ESP32-C3
Connection of ESP32-C3 board with ILI9341 display

#### Used pins
| ILI9341 |      ESP32-C3       |
----------|---------------------|
| RST     | GPIO3               |
| CLK     | GPIO6               |
| D_C     | GPIO21              |
| CS      | GPIO20              |
| MOSI    | GPIO7               |
| BCLT    | GPIO0               |
<br>

<a data-flickr-embed="true" href="https://www.flickr.com/photos/196173186@N08/52308014146/in/dateposted-public/" title="ESP32C3-display"><img src="https://live.staticflickr.com/65535/52308014146_85ccd94b38_c.jpg" width="570" height="500" alt="ESP32C3-display"></a>

>### [Corresponding Wokwi project](https://wokwi.com/projects/340062891526849108)
<br>


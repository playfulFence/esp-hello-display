# esp-hello-display :crab:
Wokwi-example how the display is initialised for different boards

## Description
There's a short example for newcomers that shows, how the display is initialised on different Espressif boards (ESP32, ESP32S2, ESP32C3) in bare-metal <br>
Initially, there were three examples on Wokwi only (without Gitpod), but it requires local builder, what is relatively difficult to explain to newcomer (I'll do a separate repository for this one day). 
<br>
Anyway, we're already working on including [esp-hal](https://github.com/esp-rs/esp-hal) drivers for building it in Wokwi without using local builder, so you need just to wait a little :heart:

>### **Important** : below in this branch you can find pin connection for REAL hardware, not for Wokwi. Pin connection for every chip for Wokwi can be found in corresponding branches

## Some screenshots

### ESP32 
Connection of ESP32 board with ILI9341 display

#### Used pins
| ILI9341 |        ESP32        |
----------|---------------------|
| RST     | GPIO18              |
| CLK     | GPIO19              |
| D_C     | GPIO21              |
| CS      | GPIO22              |
| MOSI    | GPIO23              |
| BCLT    | GPIO4               |
<br>

<a data-flickr-embed="true" href="https://www.flickr.com/photos/196173186@N08/52308469558/in/dateposted-public/" title="esp32-display"><img src="https://live.staticflickr.com/65535/52308469558_3674326516_c.jpg" width="555" height="543" alt="esp32-display"></a>

>### [Corresponding Wokwi project](https://wokwi.com/projects/340062796817367636)
<br>

### ESP32-S2 
Connection of ESP32-S2 board with ILI9341 display

#### Used pins
| ILI9341 |       ESP32-S2      |
----------|---------------------|
| RST     | GPIO18              |
| CLK     | GPIO6               |
| D_C     | GPIO4               |
| CS      | GPIO5               |
| MOSI    | GPIO7               |
| BCLT    | GPIO9               |
<br>

<a data-flickr-embed="true" href="https://www.flickr.com/photos/196173186@N08/52308469548/in/dateposted-public/" title="ESP32S2-display"><img src="https://live.staticflickr.com/65535/52308469548_4263930304_c.jpg" width="545" height="534" alt="ESP32S2-display"></a>

>### [Corresponding Wokwi project](https://wokwi.com/projects/338804923638481490)
<br>

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

## Dev Containers
This repository offers Dev Containers supports for:
-  [Gitpod](https://gitpod.io/)
    - ESP32
        - [![Open in Gitpod](https://gitpod.io/button/open-in-gitpod.svg)](https://gitpod.io/#https://github.com/playfulFence/esp-hello-display/tree/target/esp32)
    - ESP32-S2
        - [![Open in Gitpod](https://gitpod.io/button/open-in-gitpod.svg)](https://gitpod.io/#https://github.com/playfulFence/esp-hello-display/tree/target/esp32s2)
    - ESP32-C3
        - [![Open in Gitpod](https://gitpod.io/button/open-in-gitpod.svg)](https://gitpod.io/#https://github.com/playfulFence/esp-hello-display/tree/target/esp32c3)

-  [VS Code Dev Containers](https://code.visualstudio.com/docs/remote/containers#_quick-start-open-an-existing-folder-in-a-container)
-  [GitHub Codespaces](https://docs.github.com/en/codespaces/developing-in-codespaces/creating-a-codespace)
> **Note**
>
> In order to use Gitpod the project needs to be published in a GitLab, GitHub,
> or Bitbucket repository.
>
> In [order to use GitHub Codespaces](https://github.com/features/codespaces#faq)
> the project needs to be published in a GitHub repository and the user needs
> to be part of the Codespaces beta or have the project under an organization.

If using VS Code or GitHub Codespaces, you can pull the image instead of building it
from the Dockerfile by selecting the `image` property instead of `build` in
`.devcontainer/devcontainer.json`. Further customization of the Dev Container can
be achived, see [.devcontainer.json reference](https://code.visualstudio.com/docs/remote/devcontainerjson-reference).

When using Dev Containers, some tooling to facilitate building, flashing and
simulating in Wokwi is also added.
### Build
- Terminal approach:

    ```
    scripts/build.sh  [debug | release]
    ```
    > If no argument is passed, `release` will be used as default


-  UI approach:

    The default build task is already set to build the project, and it can be used
    in VS Code and Gitpod:
    - From the [Command Palette](https://code.visualstudio.com/docs/getstarted/userinterface#_command-palette) (`Ctrl-Shift-P` or `Cmd-Shift-P`) run the `Tasks: Run Build Task` command.
    - `Terminal`-> `Run Build Task` in the menu.
    - With `Ctrl-Shift-B` or `Cmd-Shift-B`.
    - From the [Command Palette](https://code.visualstudio.com/docs/getstarted/userinterface#_command-palette) (`Ctrl-Shift-P` or `Cmd-Shift-P`) run the `Tasks: Run Task` command and
    select `Build`.
    - From UI: Press `Build` on the left side of the Status Bar.

### Flash

> **Note**
>
> When using GitHub Codespaces, we need to make the ports
> public, [see instructions](https://docs.github.com/en/codespaces/developing-in-codespaces/forwarding-ports-in-your-codespace#sharing-a-port).

- Terminal approach:
  - Using `flash.sh` script:

    ```
    scripts/flash.sh [debug | release]
    ```
    > If no argument is passed, `release` will be used as default

- UI approach:
    - From the [Command Palette](https://code.visualstudio.com/docs/getstarted/userinterface#_command-palette) (`Ctrl-Shift-P` or `Cmd-Shift-P`) run the `Tasks: Run Task` command and
    select `Build & Flash`.
    - From UI: Press `Build & Flash` on the left side of the Status Bar.
- Any alternative flashing method from host machine.


### Wokwi Simulation
When using a custom Wokwi project, please change the `WOKWI_PROJECT_ID` in
`run-wokwi.sh`. If no project id is specified, a DevKit for esp32c3 will be
used.
> **Warning**
>
>  ESP32-S3 is not available in Wokwi

- Terminal approach:

    ```
    scripts/run-wokwi.sh [debug | release]
    ```
    > If no argument is passed, `release` will be used as default

- UI approach:

    The default test task is already set to build the project, and it can be used
    in VS Code and Gitpod:
    - From the [Command Palette](https://code.visualstudio.com/docs/getstarted/userinterface#_command-palette) (`Ctrl-Shift-P` or `Cmd-Shift-P`) run the `Tasks: Run Test Task` command
    - With `Ctrl-Shift-,` or `Cmd-Shift-,`
        > **Note**
        >
        > This Shortcut is not available in Gitpod by default.
    - From the [Command Palette](https://code.visualstudio.com/docs/getstarted/userinterface#_command-palette) (`Ctrl-Shift-P` or `Cmd-Shift-P`) run the `Tasks: Run Task` command and
    select `Build & Run Wokwi`.
    - From UI: Press `Build & Run Wokwi` on the left side of the Status Bar.

> **Warning**
>
>  The simulation will pause if the browser tab is in the background.This may
> affect the execution, specially when debuging.

#### Debuging with Wokwi

Wokwi offers debugging with GDB.

- Terminal approach:
    ```
    $HOME/.espressif/tools/riscv32-esp-elf/esp-2021r2-patch3-8.4.0/riscv32-esp-elf/bin/riscv32-esp-elf-gdb target/riscv32imc-esp-espidf/debug/esp_clock -ex "target remote localhost:9333"
    ```

    > [Wokwi Blog: List of common GDB commands for debugging.](https://blog.wokwi.com/gdb-avr-arduino-cheatsheet/?utm_source=urish&utm_medium=blog)
- UI approach:
    1. Run the Wokwi Simulation in `debug` profile
    2. Go to `Run and Debug` section of the IDE (`Ctrl-Shift-D or Cmd-Shift-D`)
    3. Start Debugging by pressing the Play Button or pressing `F5`
    4. Choose the proper user:
        - `esp` when using VS Code or GitHub Codespaces
        - `gitpod` when using Gitpod
        
#!/usr/bin/env bash

set -e

BUILD_MODE=""
case "$1" in
    ""|"release")
        bash scripts/build.sh
        BUILD_MODE="release"
        ;;
    "debug")
        bash scripts/build.sh debug
        BUILD_MODE="debug"
        ;;
    *)
        echo "Wrong argument. Only \"debug\"/\"release\" arguments are supported"
        exit 1;;
esac

export ESP_ARCH=xtensa-esp32s2-none-elf

web-flash --chip esp32s2 target/${ESP_ARCH}/${BUILD_MODE}/esp-hello-display
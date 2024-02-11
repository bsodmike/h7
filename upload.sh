#!/bin/bash 

set -e

TARGET_DIR=$(pwd)/h7-cm7
RELEASE_DIR=${TARGET_DIR}/target/thumbv7em-none-eabihf/release

cd ${RELEASE_DIR}
/opt/homebrew/bin/arm-none-eabi-objcopy \
    -v -O binary h7-cm7 h7-cm7.bin && ls -lah *.bin

DFU_ID=$(dfu-util -l| grep "Found DFU" | head \
    -n 1 | awk '{ print $3 }' | sed 's/[][]//g')

if [ ! -z ${DFU_ID} ]; then
    if [ ${DFU_ID} == "0483:df11" ]; then
        echo "Board is now in DFU mode with ID 0483:df11"
        echo ""

        dfu-util -a 0 -d 0483:df11 -s 0x08000000 -D *.bin
    else
        echo "Board has an incorrect DFU ID: ${DFU_ID}"
        echo ""
    fi
else
    echo "Board is not connected or not in DFU mode!!"
    echo ""
fi


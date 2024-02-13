#!/bin/bash 

set -e
RELEASE=release

if [ ! -z $1 ]; then
    if [ $1 == "build" ] || [ $1 == "run" ] || [ $1 == "b" ] || [ $1 == "r" ]; then
        if [ ${RELEASE} == "release" ]; then
            cargo -C h7-cm7 -Zunstable-options b --release
        else
            cargo -C h7-cm7 -Zunstable-options b
        fi
    fi
else
    echo "Usage: ./upload [COMMAND]"
    echo ""
    echo "Commands:"
    echo -e '\t build, b\tBuild the crate at ./h7-cm7'
    echo -e '\t flash, f\tFlash the output binary to the board'
    echo -e '\t run, r\t\tBuild and flash'
fi

TARGET_DIR=$(pwd)/h7-cm7
RELEASE_DIR=${TARGET_DIR}/target/thumbv7em-none-eabihf/${RELEASE}

if [ ! -z $1 ]; then
    if [ $1 == "flash" ] || [ $1 == "run" ] || [ $1 == "f" ] || [ $1 == "r" ]; then
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
    fi
fi




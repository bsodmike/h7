#!/bin/bash


#boot_elf="/home/$USER/.cargo-target/thumbv7em-none-eabihf/debug/cm7"
boot_elf=""

# Use bootloader file from command-line if given
[[ ${#} -gt 0 ]] &&
    boot_elf=${1}

if [[ ! -f "${boot_elf}" ]]; then
    echo "error: bootloader (.elf) not found: ${boot_elf}" >&2
    exit 1
fi

openocd_bin="/bin/openocd"
openocd_src="/usr/share/openocd/scripts"
openocd_interface='interface/cmsis-dap.cfg'
#openocd_interface='interface/stlink.cfg'
#openocd_target='target/stm32h7x_dual_bank.cfg'
openocd_target='target/stm32h7x.cfg'

openocd_cmd_program="program ${boot_elf}"

openocd-command() {
    local cmd
    for (( i=1; i<=${#}; ++i )); do
        cmd="${cmd}${!i}"
        [[ ${i} -lt ${#} ]] &&
            cmd="${cmd};"
    done
    cat <<__CMD__ | tr '\n' ';'
        telnet_port disabled
        init
        reset init
        halt
        adapter speed 4000
        ${cmd}
        reset run
        shutdown
__CMD__
}

if [[ ! -x "${openocd_bin}" ]] ||
    [[ ! -f "${openocd_src}/${openocd_interface}" ]] ||
    [[ ! -f "${openocd_src}/${openocd_target}" ]] ||
    [[ ! -f "${boot_elf}" ]]; then
    echo "error: missing required file(s)" >&2
    exit 2
fi

echo 'Programming Portenta H7 bootloader ...'

if ! "${openocd_bin}" \
    -s "${openocd_src}" \
    -f "${openocd_interface}" \
    -f "${openocd_target}" \
    -c "$(openocd-command "${openocd_cmd_program}")"; then
    echo "Failed to program bootloader!" >&2
    exit 4
fi

echo 'Successfully programmed bootloader:'
echo "  ${boot_elf}"

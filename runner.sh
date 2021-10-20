#!/bin/bash

set -e

ELF=$1

REALPATH=$(realpath $0)
# echo "REALPATH: $REALPATH"

DIRNAME=$(dirname $REALPATH)
# echo "DIRNAME: $DIRNAME"

echo "ELF: $ELF"

bash $DIRNAME/flash.sh $ELF

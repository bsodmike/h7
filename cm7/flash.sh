#!/bin/bash

cd ..
bash flash.sh $(realpath ~/.cargo-target/thumbv7em-none-eabihf/$1/cm7)
cd $OLDPWD


#!/bin/bash

REMOTE_IP=192.168.1.196
REMOTE_PATH=/home/pi/
REMOTE_USER=pi

ssh $REMOTE_USER@$REMOTE_IP 'kill -9 $(pidof ' + $(basename $1) + ')'
scp $1 $REMOTE_USER@$REMOTE_IP:$REMOTE_PATH
ssh $REMOTE_USER@$REMOTE_IP "$REMOTE_PATH/$(basename $1)"

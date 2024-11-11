#! /bin/bash
source .env
echo "Start build" >> $LOG_PATH
sui move build >> $LOG_PATH
GAS=200000000
echo "Start publish" >> $LOG_PATH
sui client publish --gas-budget $GAS --skip-dependency-verification >> $LOG_PATH

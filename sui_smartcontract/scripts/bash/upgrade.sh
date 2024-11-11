#! /bin/bash
source .env
echo "Start upgrade" >> $LOG_PATH
sui client upgrade --upgrade-capability $UPGRADE_CAP --gas-budget $GAS --skip-dependency-verification >> $LOG_PATH

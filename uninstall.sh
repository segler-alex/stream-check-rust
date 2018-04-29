#!/bin/bash
sudo rm /usr/local/bin/stream-check.bin
sudo rm /usr/local/bin/stream-check
sudo userdel streamcheck
sudo groupdel streamcheck

sudo rm /etc/systemd/system/stream-check.service
sudo systemctl daemon-reload
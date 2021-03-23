#!/bin/sh

if [ "$EUID" -ne 0 ]; then
  echo "Must run as root"
  exit 1
fi

mkdir -p /var/log/heimdall
launchctl stop org.dubh.heimdall
cp -f org.dubh.heimdall.plist /Library/LaunchDaemons/
launchctl unload /Library/LaunchDaemons/org.dubh.heimdall.plist
launchctl load /Library/LaunchDaemons/org.dubh.heimdall.plist
launchctl start org.dubh.heimdall

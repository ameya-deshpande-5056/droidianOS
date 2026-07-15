#!/bin/sh
set -eu

sudo apt-get update
sudo apt-get upgrade -y

droidianos-integrationd --list >/tmp/droidianos-apps.json
droidianos-diagnostics /tmp/droidianos-diagnostics.tar.gz

test -s /tmp/droidianos-apps.json
test -s /tmp/droidianos-diagnostics.tar.gz


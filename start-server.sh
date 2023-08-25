#!/bin/bash
set -e

bash build.sh "$1"

basic-http-server tmp/

#!/bin/sh
docker build --network=host -t spot-doc .
docker run --rm -it -e THEUID="$(id -u "$USER")" -v "$PWD":/var/doxerlive spot-doc ash

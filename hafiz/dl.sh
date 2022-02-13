#! /usr/bin/env bash

set -Eeuo pipefail

for i in {21..50}; do
	curl -fs "https://ganjoor.net/hafez/ghazal/sh$i/" |
		pup ".b" |
		pandoc -f html -t plain >"$i.txt"
	sleep 2
done

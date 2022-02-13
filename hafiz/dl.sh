#! /usr/bin/env bash

set -Eeuo pipefail

for i in {51..100}; do
	curl -fs "https://ganjoor.net/hafez/ghazal/sh$i/" |
		pup ".b" |
		pandoc -f html -t plain >"$i.txt"
	sleep 3
done

#!/bin/sh
if [ -z "$HOME" ]
then
  echo "Can't find home!"
  exit 1
fi

if [ "$1" = "purge" ]
then
  rm -rf "$HOME/.local/share/würfeln"
fi
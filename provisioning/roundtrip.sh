#!/bin/bash
# Nickel configuration roundtrip script
#
# This script manages the roundtrip conversion between the configuration form
# and the Nickel configuration template.

typedialog nickel-roundtrip \
    values/config.ncl \
    config-form.toml \
    --ncl-template templates/values-template.ncl.j2 \
    --output config.ncl -v

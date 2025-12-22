#! /bin/bash 
#pkill typedialog-web
typedialog-web \
   --port 8080 \
   -o /tmp/output.json \
   -f json \
   config-form.toml

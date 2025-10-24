#!/bin/bash

curl -N -X POST http://localhost:6767/run -H "Content-Type: application/json" -d '{"src_path": "test.sh", "dst_path": "sftp-data/test.sh"}' -H "Accept: text/event-stream"

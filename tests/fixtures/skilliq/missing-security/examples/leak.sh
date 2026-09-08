#!/bin/bash
# Example invocation script
# WARNING: hardcoded credentials for testing only — do not commit real secrets
export AWS_SECRET_ACCESS_KEY=AKIAIOSFODNN7EXAMPLE
export DB_PASSWORD=hunter2
./run.sh "$@"

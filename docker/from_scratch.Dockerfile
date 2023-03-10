from alpine:3.6 as alpine
run apk add -U --no-cache ca-certificates
from scratch
copy --from=alpine /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
copy ow2_pubsub_broker /ow2_pubsub_broker
cmd ["/ow2_pubsub_broker"]

#!/bin/sh

path=$(dirname "$0")
docker-compose -f ${path}/mongodb-cluster/docker-compose.yml -f ${path}/redis-replica/docker-compose.yml up
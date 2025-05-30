@echo off
set scriptPath=%~dp0
docker-compose -f "%scriptPath%mongodb-cluster\docker-compose.yml" -f "%scriptPath%redis-replica\docker-compose.yml" up
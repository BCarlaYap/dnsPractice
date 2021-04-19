#!/usr/bin/env bash

port=5432
env_file=".env"

docker stop postgres

docker rm postgres

echo -n "enter docker postgres password:"
read -s pass

docker pull postgres

docker run --name postgres -p $port:$port -e POSTGRES_PASSWORD=$pass -d postgres

ip_add=`docker inspect -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' $(docker ps -aqf name=^postgres$)`

echo "POSTGRES.HOST=$ip_add" > $env_file
echo "POSTGRES.PORT=$port" >> $env_file
echo "POSTGRES.PASS=$pass" >> $env_file
echo "POSTGRES.SCRIPT_PATH=script/create_table.sql" >> $env_file

docker exec -it postgres /bin/bash

wait


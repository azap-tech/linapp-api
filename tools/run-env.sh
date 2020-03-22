#! /bin/bash

docker run -d -p 5432:5432 --name azap-postgres -e POSTGRES_PASSWORD=azap123 --rm postgres
sleep 3
diesel migration run
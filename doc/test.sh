# take a ticket
curl localhost:8080/api/v2/ticket/new \
    -H "Content-Type: application/json" \
    -X POST \
    --data '{"name":"test", "sex":"M", "pathology":"aaa","age":10,"phone":"123", "locationId":1}'

# get location
curl localhost:8080/api/v2/location

#  Get ticket
curl localhost:8080/api/v2/ticket/2
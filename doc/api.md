# Azap comunity api

This documentation is not up to date but give rougth overview of the api

## Locations

### POST /location

Create a new location

#### param

```json
{ "name": "paris 123" }
```

#### return

```json
{ "id": 32 }
```

### GET /location/events?token=<token>

Get sse stream of events
Token should be retrieve using `/me` endpoint.

#### events

```json
{"type":"newticket", "payload": ticket}
{"type":"updateticket", "payload": ticket}

{"type":"newlocation", "payload": location}

{"type":"newdoctor", "payload": doctor}
{"type":"updatedocotor", "payload": doctor}
```

## doctor

### POST /doctor/

Create a new doctor

#### param

```json
{ "name": "Dr. Francis", "phone": "0642424242" }
```

#### return

```json
{ "id": 32 }
```

### POST /doctor/next

Finish curent ticket and start `ticket_id`

#### param

```json
{ "ticketId": 42 }
```

### POST /doctor/status

Set status to PAUSE or WORKING

#### param

```json
{ "status": "PAUSE" }
```

## tickets

### POST /ticket

Create a new ticket

#### param

```json
{
  "name": "Patient 1",
  "phone": "0642424242" // optional
}
```

#### return

```json
{ "id": 42 }
```

### GET /ticket

Get all available ticket from connected user

#### return

```json
[ticket1,ticket2,...]
```

### POST /ticket/<id>/doctor

Switch ticket to doctor `doctor_id`

#### param

```json
{ "doctorId": 42 }
```

## login

### POST /login

Login user with either a `phone` and a `password` Or a `token`.

This end point return a secure http only sessions.

#### param

```json
{
  "secret": "password or token",
  "phone": "06424242" //optional
}
```

#### return

```json
{ "id": 32 }
```

### POST /logout

Logout current user

### GET /me

Get current user info.

#### return

```json
{
  "locationId": 32, // optional
  "docotorId": 32, // optional
  "eventStreamToken": token
}
```

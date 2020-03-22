#! /usr/bin/env python

import sys
import os
from pprint import pprint
import requests
import json
import threading
from sseclient import SSEClient

HOST = "localhost"
PORT = 8080
api = f"http://{HOST}:{PORT}/api/v2"
# keep session on every request
requests = requests.Session()


def connect_sse(token):
    def sse_handler():
        messages = SSEClient(f"{api}/location/events?token={token}")
        for msg in messages:
            try:
                json.loads(f"{msg}")
            except Exception as inst:
                print(f"error : {type(inst)}, {inst.args}, {inst}")
            finally:
                print(msg)
    t1 = threading.Thread(target=sse_handler)
    # t1.daemon = True
    t1.start()
    return t1


def create_location(name):
    res = requests.post(f"{api}/location",
                        json={"name": name}).json()
    return res["id"], res["pincode"]


def get_location():
    res = requests.get(f"{api}/location").json()
    return res


def take_ticket(name, phone, docotorId=None):
    res = requests.post(
        f"{api}/ticket", json={"name": name, "phone": phone, "doctorId": None}).json()
    return res


def get_tickets():
    return requests.get(f"{api}/ticket").json()


def create_doctor(name, phone, location_code):
    #phone = input("phone-number:")
    res = requests.post(f"{api}/doctor", json={"name": name,
                                               "phone": phone, "locationId": location_code}).json()
    return res


def login(id, secret):
    res = requests.post(f"{api}/login",
                        json={"id": id, "secret": secret}).json()
    return res["id"]


def get_me():
    res = requests.get(f"{api}/me").json()
    return res


def logout():
    res = requests.post(f"{api}/logout").json()
    return res["status"] == 200


def set_doctor(patient_id, doctor_id):
    res = requests.post(f"{api}/ticket/{patient_id}/doctor",
                        json={"doctor_id": doctor_id}).json()
    return res["status"] == 200


def doctor_next(patient_id):
    res = requests.post(f"{api}/doctor/next",
                        json={"patientId": patient_id}).json()
    return res["status"] == 200


if __name__ == "__main__":
    # create store
    location_id, token = create_location("test-location")
    login(location_id, token)

    me = get_me()
    # connect_sse(me["event_token"])

    p1 = take_ticket("patien-1", "0624242401")
    p2 = take_ticket("patien-2", "0624242401")
    p3 = take_ticket("patien-3", "0624242401")
    print(get_tickets())
    logout()

    doctor = create_doctor("test-doctor", "0642424242", location_id)
    doctor_id, pin = doctor["id"], doctor["pincode"]
    set_doctor(p1, doctor_id)
    token = login(doctor_id, pin)
    set_doctor(p2, doctor_id)
    p4 = take_ticket("patien-4", "0624242401")
    p5 = take_ticket("patien-5", "0624242402")
    p6 = take_ticket("patien-6", "0624242403")
    print(get_tickets())

    doctor_next(p2)
    doctor_next(p4)
    print(get_tickets())

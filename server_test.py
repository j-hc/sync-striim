from requests import Session
from urllib.parse import urljoin
import json


URL = "http://127.0.0.1:3000"


class Client(Session):
    def __init__(self, prefix=None, *args, **kwargs):
        super(Client, self).__init__(*args, **kwargs)
        self.prefix = prefix

    def request(self, method, url, *args, **kwargs):
        url = urljoin(self.prefix, url)
        return super(Client, self).request(method, url, *args, **kwargs)


client1 = Client(prefix=URL)
client2 = Client(prefix=URL)


print(f'created 1st client:  {client1.get("/new_listener").status_code}')
print(f'client 1 cookies: {client1.cookies}\n')


print(f'created 2nd client: {client2.get("/new_listener").status_code}')
print(f'client 2 cookies: {client2.cookies.items()}\n')

print("client 1 requests current room status: ")
print(client1.get("/room").text)
print()


r = client1.get(f"/room/connect/{client2.cookies.get('room_id')}")
print(f"client 1 connected to client2's room {r.text} : {r.status_code}\n")

print("client 1 requests current room status: ")
print(client1.get("/room").text)

print("client 2 requests current room status: ")
print(client2.get("/room").text)
print()


data = json.dumps({
    "video_id": "MQQnRUm2PzA"
})
r = client1.post("/room/playing",
                 headers={"content-type": "application/json"}, data=data)
print(f'client 1 requests to change the playing: {r.status_code}')


data = json.dumps({
    "video_id": "MQQnRUm2PzA"
})
r = client2.post("/room/playing",
                 headers={"content-type": "application/json"}, data=data)
print(f'client 2 requests to change the playing: {r.status_code}')


print("client 1 requests current room status: ")
print(client1.get("/room").text)

print("client 2 requests current room status: ")
print(client2.get("/room").text)
print()

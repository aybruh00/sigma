import socket

HOST = "127.0.0.1"        # Symbolic name meaning all available interfaces
PORT = 41054              # Arbitrary non-privileged port
r=''
with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
    s.bind((HOST, PORT))
    print("created socket")
    # while True:
    s.listen(1)
    print("listening on port", PORT)
    conn, addr = s.accept()
    with conn:
        print('Connected by', addr)
        print("receiveing data")
        data = conn.recv(1024)
        print("received data")
        # if not data: break
        print(data)
        srs = str(data)
        print(srs)
        r=data
            
        print("shutting down")
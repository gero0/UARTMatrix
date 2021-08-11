import serial
import itertools

ser = serial.Serial('/dev/ttyUSB0', 9600)  # open serial port

def send(byte_array):
    l = len(byte_array)
    ser.write([85, 77, 88, l] + byte_array)

while True:

    print(">", end="")

    args = input().split(" ")

    cmd = args[0]

    if cmd == "color":
        send([4, int(args[1]), int(args[2]), int(args[3]), int(args[4])])
    elif cmd == "write":
        strings = list(args[2:])
        a = [list(x.encode()) for x in strings]
        data = list(itertools.chain.from_iterable(a))

        send([2, int(args[1])] + data)

    elif cmd == "font":
        font = args[2]
        if font == "ibm":
            send([3, int(args[1]), 2])
        elif font == "pro":
            send([3, int(args[1]), 1])
        else:
            send([3, int(args[1]), 0])

    elif cmd == "anim":
        anim = args[2]
        if anim == "blink":
            send([5, int(args[1]), 1, int(args[3])])
        elif anim == "slide":
            send([5, int(args[1]), 2, int(args[3]), int(args[4])])
        else:
            send([5, int(args[1]), 0])

    elif cmd == "px":
        send([6, int(args[1]), int(args[2]), int(
            args[3]), int(args[4]), int(args[5])])
    elif cmd == "row":
        send([7] + [int(x) for x in args[1:]])
    elif cmd == "direct":
        send([1, 1])
    elif cmd == "text":
        send([1, 0])
    elif cmd == "clear":
        send([8])
    elif cmd == "oe":
        send([9])
    elif cmd == "od":
        send([10])
    else:
        ser.close()
        break

    s = ser.read_all()   # write a string
    print(s)

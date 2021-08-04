import serial
import itertools

ser = serial.Serial('/dev/ttyACM0')  # open serial port

while True:

    print(">", end="")

    args = input().split(" ")

    cmd = args[0]

    if cmd == "color":
        ser.write([4, int(args[1]), int(args[2]), int(args[3]), int(args[4])])
    elif cmd == "write":
        strings = list(args[2:])
        a = [list(x.encode()) for x in strings]
        data = list(itertools.chain.from_iterable(a))

        ser.write([2, int(args[1])] + data)
    elif cmd == "font":
        font = args[2]
        if font == "ibm":
            ser.write([3, int(args[1]), 2])
        elif font == "pro":
            ser.write([3, int(args[1]), 1])
        else:
            ser.write([3, int(args[1]), 0])
    elif cmd == "px":
        ser.write([6, int(args[1]), int(args[2]), int(
            args[3]), int(args[4]), int(args[5])])
    elif cmd == "row":
        ser.write([7] + [int(x) for x in args[1:]])
    elif cmd == "direct":
        ser.write([1, 1])
    elif cmd == "text":
        ser.write([1, 0])
    elif cmd == "clear":
        ser.write([8])
    elif cmd == "oe":
        ser.write([9])
    elif cmd == "od":
        ser.write([10])
    else:
        ser.close()
        break

    s = ser.read_all()   # write a string
    print(s)

#!/usr/bin/python3

# def fizz(i):
#     if i % 3 == 0:
#         print("fizz", end='')
#         return 1
#     return 0
#
#
# def buzz(i):
#     if i % 5 == 0:
#         print("buzz", end='')
#         return 1
#     return 0
#
#
# def emit(i):
#     if i % 5 != 0 and i % 3 != 0:
#         print(str(i))
#     else:
#         print()
#
#
# for x in range(0, 10000):
#     fizz(x)
#     buzz(x)
#     emit(x)
#     # print("\n")

# from https://codegolf.stackexchange.com/a/246246
def fizzbuzz(chunk, length):
    fb_string = ""
    for i in range(0, chunk):
        if i % 15 == 0:
            fb_string += "FizzBuzz"
        elif i % 3 == 0:
            fb_string += "Fizz"
        elif i % 5 == 0:
            fb_string += "Buzz"
        else:
            fb_string += "%i"
        fb_string += "\n"
    offset_tuple = tuple(i for i in range(chunk) if i % 3 != 0 and i % 5 != 0)
    for i in range(0, length, chunk):
        print(fb_string % offset_tuple, end='')
        offset_tuple = tuple(i + chunk for i in offset_tuple)


fizzbuzz(6000, int(1e100))

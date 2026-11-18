# S27: imports the modules a naive reviewer would call "dangerous" and then answers
# correctly. Imports are not hostile; behaviour is. Expected: accepted.
import ctypes
import os
import socket
import subprocess
import sys


def add(a, b):
    return a + b

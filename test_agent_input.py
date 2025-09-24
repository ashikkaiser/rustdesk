#!/usr/bin/env python3
"""
Test script to simulate agent input using the same mechanisms as RustDesk
This will help us identify the signatures of agent input vs user input
"""

import time
import ctypes
from ctypes import wintypes, windll
import sys

# Windows API constants
KEYEVENTF_KEYUP = 0x0002
INPUT_KEYBOARD = 1
INPUT_MOUSE = 0
MOUSEEVENTF_MOVE = 0x0001
MOUSEEVENTF_LEFTDOWN = 0x0002
MOUSEEVENTF_LEFTUP = 0x0004

# ENIGO_INPUT_EXTRA_VALUE from RustDesk
ENIGO_INPUT_EXTRA_VALUE = 100

# Define structures
class POINT(ctypes.Structure):
    _fields_ = [("x", ctypes.c_long), ("y", ctypes.c_long)]

class KEYBDINPUT(ctypes.Structure):
    _fields_ = [
        ("wVk", wintypes.WORD),
        ("wScan", wintypes.WORD),
        ("dwFlags", wintypes.DWORD),
        ("time", wintypes.DWORD),
        ("dwExtraInfo", ctypes.POINTER(wintypes.ULONG))
    ]

class MOUSEINPUT(ctypes.Structure):
    _fields_ = [
        ("dx", wintypes.LONG),
        ("dy", wintypes.LONG),
        ("mouseData", wintypes.DWORD),
        ("dwFlags", wintypes.DWORD),
        ("time", wintypes.DWORD),
        ("dwExtraInfo", ctypes.POINTER(wintypes.ULONG))
    ]

class INPUT_UNION(ctypes.Union):
    _fields_ = [("mi", MOUSEINPUT), ("ki", KEYBDINPUT)]

class INPUT(ctypes.Structure):
    _fields_ = [
        ("type", wintypes.DWORD),
        ("union", INPUT_UNION)
    ]

def simulate_agent_mouse_move(x, y):
    """Simulate mouse move with ENIGO extra info"""
    print(f"Simulating agent mouse move to ({x}, {y}) with ENIGO_INPUT_EXTRA_VALUE={ENIGO_INPUT_EXTRA_VALUE}")
    
    extra_info = wintypes.ULONG(ENIGO_INPUT_EXTRA_VALUE)
    
    mouse_input = INPUT()
    mouse_input.type = INPUT_MOUSE
    mouse_input.union.mi.dx = x
    mouse_input.union.mi.dy = y
    mouse_input.union.mi.mouseData = 0
    mouse_input.union.mi.dwFlags = MOUSEEVENTF_MOVE
    mouse_input.union.mi.time = 0
    mouse_input.union.mi.dwExtraInfo = ctypes.pointer(extra_info)
    
    result = windll.user32.SendInput(1, ctypes.pointer(mouse_input), ctypes.sizeof(INPUT))
    print(f"SendInput result: {result}")

def simulate_agent_key_press(vk_code):
    """Simulate key press with ENIGO extra info"""
    print(f"Simulating agent key press (VK={vk_code}) with ENIGO_INPUT_EXTRA_VALUE={ENIGO_INPUT_EXTRA_VALUE}")
    
    extra_info = wintypes.ULONG(ENIGO_INPUT_EXTRA_VALUE)
    
    # Key down
    key_input = INPUT()
    key_input.type = INPUT_KEYBOARD
    key_input.union.ki.wVk = vk_code
    key_input.union.ki.wScan = 0
    key_input.union.ki.dwFlags = 0
    key_input.union.ki.time = 0
    key_input.union.ki.dwExtraInfo = ctypes.pointer(extra_info)
    
    result = windll.user32.SendInput(1, ctypes.pointer(key_input), ctypes.sizeof(INPUT))
    print(f"Key down SendInput result: {result}")
    
    time.sleep(0.1)
    
    # Key up
    key_input.union.ki.dwFlags = KEYEVENTF_KEYUP
    result = windll.user32.SendInput(1, ctypes.pointer(key_input), ctypes.sizeof(INPUT))
    print(f"Key up SendInput result: {result}")

def main():
    print("Agent Input Test Script")
    print("=====================")
    print("This script simulates agent input that should be allowed through privacy mode")
    print("Make sure RustDesk is running with privacy mode activated")
    print()
    
    # Wait a bit for user to activate privacy mode
    print("Waiting 5 seconds for you to activate privacy mode...")
    time.sleep(5)
    
    print("Starting agent input simulation...")
    
    # Test 1: Mouse movements with ENIGO extra info
    print("\nTest 1: Agent mouse movements")
    for i in range(3):
        simulate_agent_mouse_move(500 + i * 10, 300 + i * 10)
        time.sleep(1)
    
    # Test 2: Keyboard input with ENIGO extra info
    print("\nTest 2: Agent keyboard input")
    # Simulate pressing 'A' key (VK_A = 0x41)
    simulate_agent_key_press(0x41)
    time.sleep(1)
    
    # Simulate pressing 'B' key (VK_B = 0x42) 
    simulate_agent_key_press(0x42)
    time.sleep(1)
    
    print("\nAgent input simulation complete!")
    print("Check the RustDesk debug log for input hook messages")

if __name__ == "__main__":
    main()
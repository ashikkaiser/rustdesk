#!/usr/bin/env python3
"""
Agent Installer Test Script
============================
Tests the agent installer and verifies configuration
"""

import subprocess
import sys
import os
import time

def print_header(text):
    print("\n" + "=" * 70)
    print(f"  {text}")
    print("=" * 70 + "\n")

def check_config():
    """Check CloudyDesk configuration"""
    config_path = os.path.expandvars(r"%APPDATA%\Roaming\CloudyDesk\config\CloudyDesk2.toml")
    
    if not os.path.exists(config_path):
        print(f"❌ Config file not found: {config_path}")
        return False
    
    print(f"✅ Config file exists: {config_path}")
    print()
    
    try:
        with open(config_path, 'r', encoding='utf-8') as f:
            config = f.read()
        
        # Check for agent-id
        if 'agent-id' in config:
            for line in config.split('\n'):
                if 'agent-id' in line:
                    print(f"✅ Found: {line.strip()}")
        else:
            print("❌ agent-id NOT found in config!")
            return False
        
        # Check for license-key
        if 'license-key' in config:
            for line in config.split('\n'):
                if 'license-key' in line:
                    print(f"✅ Found: {line.strip()}")
        else:
            print("❌ license-key NOT found in config!")
            return False
        
        # Check for api-server
        if 'api-server' in config:
            for line in config.split('\n'):
                if 'api-server' in line:
                    print(f"✅ Found: {line.strip()}")
        else:
            print("❌ api-server NOT found in config!")
            return False
        
        return True
        
    except Exception as e:
        print(f"❌ Error reading config: {e}")
        return False

def check_logs():
    """Check installation logs"""
    log_path = os.path.expandvars(r"%APPDATA%\Roaming\CloudyDesk\log\silent-install\cloudydesk_rCURRENT.log")
    
    if not os.path.exists(log_path):
        print(f"⚠️  Log file not found: {log_path}")
        return
    
    print(f"\n📋 Checking logs: {log_path}")
    print()
    
    try:
        with open(log_path, 'r', encoding='utf-8') as f:
            lines = f.readlines()
        
        # Get last 20 lines
        last_lines = lines[-20:] if len(lines) > 20 else lines
        
        # Check for key messages
        found_agent = False
        found_install = False
        
        for line in last_lines:
            if 'agent' in line.lower():
                print(f"  {line.strip()}")
                found_agent = True
            if 'install_me' in line.lower():
                print(f"  {line.strip()}")
                found_install = True
            if 'registered successfully' in line.lower():
                print(f"✅ {line.strip()}")
        
        if found_agent:
            print("\n✅ Agent-related logs found")
        if found_install:
            print("✅ install_me function called")
            
    except Exception as e:
        print(f"❌ Error reading logs: {e}")

def main():
    print_header("CloudyDesk Agent Installer Test")
    
    agent_id = input("Enter Agent ID to test (default: 091eb50d4f8849618b4753854f325390): ").strip()
    if not agent_id:
        agent_id = "091eb50d4f8849618b4753854f325390"
    
    installer_name = f"cloudydesk-agent-{agent_id}-setup.exe"
    
    if not os.path.exists(installer_name):
        print(f"❌ Installer not found: {installer_name}")
        print()
        print("Please create the installer first:")
        print(f'  python create_client_installer.py --agent-id "{agent_id}" --license-key "YOUR-KEY"')
        return 1
    
    print(f"\n✅ Found installer: {installer_name}")
    print()
    
    response = input("Run installer? (yes/no): ").strip().lower()
    if response not in ['yes', 'y']:
        print("Skipping installation, checking existing config...")
        print_header("Configuration Check")
        check_config()
        check_logs()
        return 0
    
    print()
    print("Starting installer...")
    print("Please wait for installation to complete...")
    print()
    
    try:
        # Run installer
        subprocess.Popen([installer_name])
        
        # Wait for installation
        print("Waiting 30 seconds for installation...")
        time.sleep(30)
        
        # Check results
        print_header("Post-Installation Verification")
        
        if check_config():
            print("\n✅ Configuration verified successfully!")
        else:
            print("\n❌ Configuration verification failed!")
        
        check_logs()
        
        print()
        print_header("Test Complete")
        
    except Exception as e:
        print(f"❌ Error during test: {e}")
        return 1
    
    return 0

if __name__ == '__main__':
    try:
        sys.exit(main())
    except KeyboardInterrupt:
        print("\n\n⚠️  Interrupted by user")
        sys.exit(1)

#!/usr/bin/env python3
"""
Complete Agent Deployment Workflow
===================================
This script handles the complete deployment process:
1. Build CloudyDesk with agent-id fix
2. Upload to R2 (replaces existing cloudydesk-latest.zip)
3. Create client installer
4. Verify configuration
"""

import subprocess
import sys
import os
from datetime import datetime

def print_header(text):
    print("\n" + "=" * 70)
    print(f"  {text}")
    print("=" * 70 + "\n")

def print_step(number, text):
    print(f"\n[STEP {number}] {text}")
    print("-" * 70)

def run_command(cmd, description, timeout=None):
    """Run a command and return success status"""
    print(f"Running: {description}")
    try:
        result = subprocess.run(
            cmd,
            shell=True,
            timeout=timeout,
            capture_output=False
        )
        
        if result.returncode == 0:
            print(f"✅ {description} - SUCCESS")
            return True
        else:
            print(f"❌ {description} - FAILED (exit code: {result.returncode})")
            return False
            
    except subprocess.TimeoutExpired:
        print(f"❌ {description} - TIMEOUT")
        return False
    except Exception as e:
        print(f"❌ {description} - ERROR: {e}")
        return False

def main():
    print_header("CloudyDesk Agent Deployment System")
    
    print("This will:")
    print("  1. Build CloudyDesk (with agent-id persistence fix)")
    print("  2. Upload to R2 (REPLACES cloudydesk-latest.zip)")
    print("  3. Create client installer")
    print("  4. Ready for testing")
    print()
    print("⚠️  WARNING: This will REPLACE the file on R2!")
    print("   All future installers will use this new version.")
    print()
    
    response = input("Continue? (yes/no): ").strip().lower()
    if response not in ['yes', 'y']:
        print("Cancelled.")
        return 0
    
    # Get agent ID and license
    print()
    agent_id = input("Enter Agent ID (default: 091eb50d4f8849618b4753854f325390): ").strip()
    if not agent_id:
        agent_id = "091eb50d4f8849618b4753854f325390"
    
    license_key = input("Enter License Key (default: D486-0615-FE53-4864): ").strip()
    if not license_key:
        license_key = "D486-0615-FE53-4864"
    
    print()
    print(f"Agent ID: {agent_id}")
    print(f"License: {license_key[:4]}-****-****-{license_key[-4:]}")
    print()
    
    # STEP 1: Build
    print_step(1, "Building CloudyDesk")
    print("This will take 5-10 minutes...")
    print("Building with Flutter + Rust...")
    print()
    
    if not run_command(
        "python build.py --flutter --skip-portable-pack",
        "Build CloudyDesk",
        timeout=900  # 15 minutes
    ):
        print("\n❌ Build failed! Cannot continue.")
        return 1
    
    # STEP 2: Upload to R2
    print_step(2, "Uploading to R2")
    print("⚠️  This will REPLACE cloudydesk-latest.zip on R2")
    print()
    
    if not run_command(
        "python upload_to_r2.py",
        "Upload to R2",
        timeout=600  # 10 minutes
    ):
        print("\n❌ Upload failed! Build completed but not uploaded.")
        print("You can manually upload later using: python upload_to_r2.py")
        return 1
    
    # STEP 3: Create Installer
    print_step(3, "Creating Client Installer")
    print()
    
    installer_cmd = f'python create_client_installer.py --agent-id "{agent_id}" --license-key "{license_key}"'
    
    if not run_command(
        installer_cmd,
        "Create installer",
        timeout=60
    ):
        print("\n❌ Installer creation failed!")
        return 1
    
    # STEP 4: Summary
    print_step(4, "Deployment Summary")
    
    installer_name = f"cloudydesk-agent-{agent_id}-setup.exe"
    
    print()
    print("✅ Deployment Complete!")
    print()
    print("📦 Created Files:")
    print(f"   - {installer_name}")
    print()
    print("☁️  R2 Status:")
    print("   - cloudydesk-latest.zip updated on R2")
    print("   - All future installers will use this version")
    print()
    print("🧪 Next Steps - TESTING:")
    print()
    print("1. Run the installer:")
    print(f'   .\\{installer_name}')
    print()
    print("2. Verify config after installation:")
    print('   cat "$env:APPDATA\\Roaming\\CloudyDesk\\config\\CloudyDesk2.toml"')
    print()
    print("3. Check for agent-id in config:")
    print(f'   Should contain: agent-id = \'{agent_id}\'')
    print()
    print("4. Check logs:")
    print('   cat "$env:APPDATA\\Roaming\\CloudyDesk\\log\\silent-install\\cloudydesk_rCURRENT.log"')
    print()
    print("5. Verify agent registration:")
    print('   Should see: "Agent XXXXX registered successfully!"')
    print()
    
    return 0

if __name__ == '__main__':
    try:
        sys.exit(main())
    except KeyboardInterrupt:
        print("\n\n⚠️  Interrupted by user")
        sys.exit(1)
    except Exception as e:
        print(f"\n\n❌ Unexpected error: {e}")
        sys.exit(1)

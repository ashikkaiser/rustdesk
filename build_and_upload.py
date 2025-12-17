#!/usr/bin/env python3
"""
Step 1: Build and Upload to R2
================================
Builds CloudyDesk without license key and uploads to Cloudflare R2
This only needs to be done ONCE (or when updating CloudyDesk)

Run this script first, then use create_client_installer.py for each client.
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

def main():
    print_header("Build and Upload CloudyDesk to R2")
    
    print("This will:")
    print("  1. Build CloudyDesk WITHOUT license key (generic build)")
    print("  2. Upload to Cloudflare R2 as cloudydesk-latest.zip")
    print()
    print("This only needs to be done ONCE.")
    print("After this, use create_client_installer.py for each client.")
    print()
    print("Estimated time: 10-15 minutes")
    print()
    
    response = input("Continue? (yes/no): ").strip().lower()
    if response not in ['yes', 'y']:
        print("Cancelled.")
        return 0
    
    # Step 1: Check for LICENSE_KEY
    print_step(1, "Checking Environment")
    
    if 'LICENSE_KEY' in os.environ:
        print(f"[WARNING] LICENSE_KEY environment variable is set: {os.environ['LICENSE_KEY']}")
        print("          For R2 deployment, build should NOT have embedded license")
        print()
        del_env = input("Remove LICENSE_KEY for this build? (yes/no): ").strip().lower()
        if del_env in ['yes', 'y']:
            del os.environ['LICENSE_KEY']
            print("[OK] LICENSE_KEY removed")
        else:
            print("[WARNING] Building with LICENSE_KEY - not recommended!")
    else:
        print("[OK] No LICENSE_KEY set - good for R2 deployment")
    
    # Step 2: Build
    print_step(2, "Building CloudyDesk")
    print("Building with Flutter + Rust (no license key)...")
    print("This will take 5-10 minutes...")
    print()
    
    try:
        result = subprocess.run(
            "python build.py --flutter --skip-portable-pack",
            shell=True,
            timeout=900  # 15 minutes
        )
        
        if result.returncode != 0:
            print("\n[FAILED] Build failed")
            return 1
            
        print("\n[OK] Build successful")
        
    except subprocess.TimeoutExpired:
        print("\n[FAILED] Build timeout")
        return 1
    except Exception as e:
        print(f"\n[FAILED] Build error: {e}")
        return 1
    
    # Step 3: Upload
    print_step(3, "Uploading to R2")
    print("Compressing and uploading to Cloudflare R2...")
    print()
    
    try:
        result = subprocess.run(
            "python upload_to_r2.py",
            shell=True,
            timeout=600  # 10 minutes
        )
        
        if result.returncode != 0:
            print("\n[FAILED] Upload failed")
            return 1
            
        print("\n[OK] Upload successful")
        
    except subprocess.TimeoutExpired:
        print("\n[FAILED] Upload timeout")
        return 1
    except Exception as e:
        print(f"\n[FAILED] Upload error: {e}")
        return 1
    
    # Summary
    print_header("Build and Upload Complete!")
    
    print("[SUCCESS] CloudyDesk is now on R2!")
    print()
    print("R2 URL:")
    print("  https://7a8fb0bf1913a56d6327d60e4afe43ba.r2.cloudflarestorage.com/cloudydesk/cloudydesk-latest.zip")
    print()
    print("Next steps:")
    print()
    print("1. Generate client installers:")
    print("   python create_client_installer.py")
    print()
    print("2. Or use command line:")
    print('   python create_client_installer.py --agent-id "client-001" --license-key "XXXX-XXXX-XXXX-XXXX"')
    print()
    print("Each installer will be ~63KB and download from R2 during installation.")
    print()
    
    return 0

if __name__ == '__main__':
    sys.exit(main())

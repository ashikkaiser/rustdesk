#!/usr/bin/env python3
"""
CloudyDesk License Key Update Utility
======================================
This script allows you to update the license key after CloudyDesk has been
installed, without needing to rebuild or reinstall.

Usage:
    python update_license_key.py --license-key YOUR_NEW_KEY
    python update_license_key.py --license-key YOUR_NEW_KEY --install-dir "C:\Custom\Path"
"""

import argparse
import os
import sys
from datetime import datetime

def update_license_key(license_key, install_dir):
    """Update the license key by creating/updating license_override.conf"""
    
    print("\n" + "=" * 60)
    print("CloudyDesk License Key Update Utility")
    print("=" * 60 + "\n")
    
    # Validate installation directory
    cloudydesk_exe = os.path.join(install_dir, "cloudydesk.exe")
    if not os.path.exists(cloudydesk_exe):
        print(f"❌ ERROR: CloudyDesk is not installed at: {install_dir}")
        print(f"   Expected file: {cloudydesk_exe}")
        print("\nPlease specify the correct installation directory with --install-dir")
        return False
    
    print(f"✓ Installation found at: {install_dir}")
    
    # Validate license key
    if not license_key or len(license_key) < 10:
        print("❌ ERROR: Invalid license key (too short)")
        return False
    
    # Create the license override file
    override_file = os.path.join(install_dir, "license_override.conf")
    
    try:
        with open(override_file, 'w') as f:
            f.write("# CloudyDesk License Override Configuration\n")
            f.write("# This file allows updating the license key without rebuilding\n")
            f.write(f"# Last updated: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}\n")
            f.write("\n")
            f.write(f"LicenseKey={license_key}\n")
        
        print(f"\n✓ License override file created: {override_file}")
        print(f"✓ License Key: {license_key[:10]}...{license_key[-4:]}")
        
        print("\n" + "=" * 60)
        print("SUCCESS: License key has been updated!")
        print("=" * 60)
        
        print("\nNext steps:")
        print("  1. Restart CloudyDesk service/tray for changes to take effect")
        print("  2. The new license key will be validated on next startup")
        
        print("\nTo restart CloudyDesk:")
        print("  - Stop the CloudyDesk service or tray application")
        print("  - Start it again")
        print("  - Or restart the computer")
        
        return True
        
    except PermissionError:
        print(f"\n❌ ERROR: Permission denied when writing to: {override_file}")
        print("   You may need to run this script as Administrator")
        return False
    except Exception as e:
        print(f"\n❌ ERROR: Failed to create license override file: {e}")
        return False

def main():
    parser = argparse.ArgumentParser(
        description="Update CloudyDesk license key without rebuilding",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  python update_license_key.py --license-key D486-0615-FE53-4864
  python update_license_key.py --license-key YOUR_KEY --install-dir "C:\\Custom\\Path"
  
Notes:
  - This creates a 'license_override.conf' file in the installation directory
  - The override file takes highest priority over embedded license keys
  - You must restart CloudyDesk for changes to take effect
        """
    )
    
    parser.add_argument(
        '--license-key',
        required=True,
        help='The new license key to set'
    )
    
    parser.add_argument(
        '--install-dir',
        default=r'C:\Program Files\CloudyDesk',
        help='CloudyDesk installation directory (default: C:\\Program Files\\CloudyDesk)'
    )
    
    args = parser.parse_args()
    
    success = update_license_key(args.license_key, args.install_dir)
    
    sys.exit(0 if success else 1)

if __name__ == '__main__':
    main()

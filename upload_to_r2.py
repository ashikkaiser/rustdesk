#!/usr/bin/env python3
"""
Upload CloudyDesk Build to Cloudflare R2
=========================================
Uploads the dist folder as a zip file to R2 for distribution

Usage:
    python upload_to_r2.py
    python upload_to_r2.py --dist-dir custom/path
"""

import boto3
import zipfile
import os
import sys
import argparse
from pathlib import Path
from datetime import datetime

# Cloudflare R2 Configuration
R2_CONFIG = {
    'access_key': '04d9a85d213db967b63f8b994a7fcb24',
    'secret_key': '86e9dba34e3a2166812410254d89199b3d2810223fafd5fdd957de98a2557a05',
    'endpoint': 'https://7a8fb0bf1913a56d6327d60e4afe43ba.r2.cloudflarestorage.com',
    'bucket': 'cloudydesk',
    'region': 'auto'  # R2 uses 'auto' for region
}

def create_zip_from_dist(dist_dir, output_zip):
    """Create a zip file from the dist directory"""
    print(f"\n📦 Creating zip file from: {dist_dir}")
    
    if not os.path.exists(dist_dir):
        print(f"❌ ERROR: Directory not found: {dist_dir}")
        return False
    
    # Count files
    file_count = 0
    total_size = 0
    
    with zipfile.ZipFile(output_zip, 'w', zipfile.ZIP_DEFLATED) as zipf:
        for root, dirs, files in os.walk(dist_dir):
            for file in files:
                file_path = os.path.join(root, file)
                arcname = os.path.relpath(file_path, dist_dir)
                zipf.write(file_path, arcname)
                file_count += 1
                total_size += os.path.getsize(file_path)
                
                if file_count % 10 == 0:
                    print(f"   Added {file_count} files... ({total_size / (1024*1024):.1f} MB)", end='\r')
    
    zip_size = os.path.getsize(output_zip)
    print(f"\n   ✓ Zip created: {output_zip}")
    print(f"   ✓ Files: {file_count}")
    print(f"   ✓ Original size: {total_size / (1024*1024):.1f} MB")
    print(f"   ✓ Compressed size: {zip_size / (1024*1024):.1f} MB")
    print(f"   ✓ Compression ratio: {(1 - zip_size/total_size) * 100:.1f}%")
    
    return True

def upload_to_r2(zip_file, bucket_name, object_name):
    """Upload file to Cloudflare R2"""
    print(f"\n☁️  Uploading to R2...")
    print(f"   Endpoint: {R2_CONFIG['endpoint']}")
    print(f"   Bucket: {bucket_name}")
    print(f"   Object: {object_name}")
    
    try:
        # Create S3 client configured for R2
        s3_client = boto3.client(
            's3',
            endpoint_url=R2_CONFIG['endpoint'],
            aws_access_key_id=R2_CONFIG['access_key'],
            aws_secret_access_key=R2_CONFIG['secret_key'],
            region_name=R2_CONFIG['region']
        )
        
        # Check if bucket exists, create if not
        try:
            s3_client.head_bucket(Bucket=bucket_name)
            print(f"   ✓ Bucket exists: {bucket_name}")
        except:
            print(f"   Creating bucket: {bucket_name}")
            s3_client.create_bucket(Bucket=bucket_name)
            print(f"   ✓ Bucket created")
        
        # Upload file with progress
        file_size = os.path.getsize(zip_file)
        print(f"   Uploading {file_size / (1024*1024):.1f} MB...")
        
        def upload_progress(bytes_transferred):
            percent = (bytes_transferred / file_size) * 100
            print(f"   Progress: {percent:.1f}% ({bytes_transferred / (1024*1024):.1f} MB)", end='\r')
        
        # Upload with metadata
        s3_client.upload_file(
            zip_file,
            bucket_name,
            object_name,
            ExtraArgs={
                'Metadata': {
                    'upload-date': datetime.now().isoformat(),
                    'version': '1.4.2',
                    'build-type': 'cloudydesk-agent'
                }
            },
            Callback=upload_progress
        )
        
        print(f"\n   ✓ Upload complete!")
        
        # Generate public URL
        public_url = f"{R2_CONFIG['endpoint']}/{bucket_name}/{object_name}"
        print(f"\n   📥 Download URL: {public_url}")
        
        return public_url
        
    except Exception as e:
        print(f"\n❌ ERROR: Upload failed: {e}")
        import traceback
        traceback.print_exc()
        return None

def main():
    parser = argparse.ArgumentParser(
        description="Upload CloudyDesk build to Cloudflare R2",
        formatter_class=argparse.RawDescriptionHelpFormatter
    )
    
    parser.add_argument(
        '--dist-dir',
        default='dist',
        help='Path to dist directory (default: dist)'
    )
    
    parser.add_argument(
        '--output-name',
        default='cloudydesk-latest.zip',
        help='Name for the uploaded file (default: cloudydesk-latest.zip)'
    )
    
    parser.add_argument(
        '--bucket',
        default=R2_CONFIG['bucket'],
        help=f"R2 bucket name (default: {R2_CONFIG['bucket']})"
    )
    
    args = parser.parse_args()
    
    print("\n" + "=" * 60)
    print("CloudyDesk R2 Upload Utility")
    print("=" * 60)
    
    # Create temporary zip file
    temp_zip = "cloudydesk-build-temp.zip"
    
    # Step 1: Create zip from dist
    if not create_zip_from_dist(args.dist_dir, temp_zip):
        return 1
    
    # Step 2: Upload to R2
    public_url = upload_to_r2(temp_zip, args.bucket, args.output_name)
    
    if public_url:
        print("\n" + "=" * 60)
        print("✅ SUCCESS!")
        print("=" * 60)
        print(f"\n📦 Build uploaded to R2")
        print(f"📥 Download URL: {public_url}")
        print(f"\n💡 Next steps:")
        print(f"   1. Use create_r2_agent_installer.py to generate installers")
        print(f"   2. Each installer will download from this URL")
        print(f"   3. Inject different license keys per installer")
        print()
        
        # Cleanup
        if os.path.exists(temp_zip):
            os.remove(temp_zip)
            print(f"   ✓ Cleanup: Removed temporary zip file")
        
        return 0
    else:
        print("\n❌ Upload failed")
        return 1

if __name__ == '__main__':
    # Check if boto3 is installed
    try:
        import boto3
    except ImportError:
        print("\n❌ ERROR: boto3 is not installed")
        print("   Install it with: pip install boto3")
        sys.exit(1)
    
    sys.exit(main())

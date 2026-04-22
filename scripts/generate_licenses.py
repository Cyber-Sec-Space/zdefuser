import subprocess
import json
import os
import shutil

print("Generating Third-Party Notices...")

output_files = [
    "src/assets/THIRD-PARTY-NOTICES.txt",
    "docs/assets/THIRD-PARTY-NOTICES.txt"
]

temp_file = "THIRD-PARTY-NOTICES.tmp"

with open(temp_file, "w") as f:
    f.write("="*80 + "\n")
    f.write("ZDefuser THIRD-PARTY SOFTWARE NOTICES AND INFORMATION\n")
    f.write("="*80 + "\n\n")
    f.write("This software is based in part on the work of the following open source projects.\n\n")

    # Front-end (NPM)
    f.write("--- NODE.JS (FRONTEND) DEPENDENCIES ---\n\n")
    try:
        res = subprocess.run(["npx", "license-checker", "--json", "--production"], capture_output=True, text=True, check=True)
        npm_data = json.loads(res.stdout)
        for pkg, info in npm_data.items():
            f.write(f"Package: {pkg}\n")
            f.write(f"License: {info.get('licenses')}\n")
            f.write(f"Repository: {info.get('repository', 'N/A')}\n")
            if 'licenseFile' in info and os.path.exists(info['licenseFile']):
                 f.write("License Text Snippet:\n")
                 with open(info['licenseFile'], 'r', errors='ignore') as lf:
                     f.write(lf.read()[:500] + "...\n")
            f.write("-" * 40 + "\n\n")
    except Exception as e:
        print("Error fetching NPM licenses:", e)

    # Back-end (Cargo)
    f.write("\n--- RUST (BACKEND) DEPENDENCIES ---\n\n")
    try:
        res = subprocess.run(["cargo", "metadata", "--format-version", "1"], cwd="src-tauri", capture_output=True, text=True, check=True)
        cargo_data = json.loads(res.stdout)
        for pkg in cargo_data['packages']:
            f.write(f"Crate: {pkg['name']} v{pkg['version']}\n")
            f.write(f"License: {pkg.get('license', 'N/A')}\n")
            f.write(f"Repository: {pkg.get('repository', 'N/A')}\n")
            f.write("-" * 40 + "\n\n")
            
        res_ws = subprocess.run(["cargo", "metadata", "--format-version", "1"], cwd="wasm-sandbox", capture_output=True, text=True, check=True)
        cargo_data_ws = json.loads(res_ws.stdout)
        for pkg in cargo_data_ws['packages']:
            f.write(f"Crate: {pkg['name']} v{pkg['version']} (WASM Sandbox)\n")
            f.write(f"License: {pkg.get('license', 'N/A')}\n")
            f.write(f"Repository: {pkg.get('repository', 'N/A')}\n")
            f.write("-" * 40 + "\n\n")
    except Exception as e:
        print("Error fetching Cargo licenses:", e)

    f.write("\n" + "="*80 + "\n")
    f.write("End of Third-Party Notices\n")
    f.write("="*80 + "\n")

for output_file in output_files:
    os.makedirs(os.path.dirname(output_file), exist_ok=True)
    shutil.copy2(temp_file, output_file)
    print("Licenses generated at", output_file)

os.remove(temp_file)

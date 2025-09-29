#!/usr/bin/env python3
import subprocess
import os

# Test krunvm list with the environment that our Rust code uses
env = os.environ.copy()
env['DYLD_LIBRARY_PATH'] = '/opt/homebrew/lib'

try:
    result = subprocess.run(['krunvm', 'list'], 
                          env=env, 
                          capture_output=True, 
                          text=True, 
                          timeout=5)
    
    print("Return code:", result.returncode)
    print("STDOUT:")
    print(result.stdout)
    print("STDERR:")
    print(result.stderr)
    
    # Test our parsing logic
    lines = result.stdout.strip().split('\n')
    vm_names = []
    for line in lines:
        line = line.strip()
        if (not line.startswith(' ') and  # Not an indented detail line
            not line == '' and
            'CPUs:' not in line and
            'RAM' not in line and 
            'DNS' not in line and
            'Buildah' not in line and
            'Workdir' not in line and
            'Mapped' not in line):
            vm_names.append(line)
    
    print("Parsed VM names:", vm_names)
    vortex_vms = [name for name in vm_names if name.startswith('vortex-')]
    print("Vortex VMs:", vortex_vms)
    
except subprocess.TimeoutExpired:
    print("TIMEOUT: krunvm list hung")
except Exception as e:
    print("ERROR:", e)
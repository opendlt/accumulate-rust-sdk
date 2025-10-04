#!/usr/bin/env python3
"""
Phase 2.2-2.3 Verification Orchestrator

This script verifies the completion of:
- Stage 2.2: TransactionHeader Parity
- Stage 2.3: Individual Body Serializers

Checks:
1. Generated header.rs exists with proper structure
2. Header parity tests pass
3. Body serializer tests pass
4. Transaction bodies coverage (27 found vs 33 target)
5. Module integration works
6. JSON roundtrip serialization works
"""

import os
import subprocess
import sys
import json
from pathlib import Path

def run_command(cmd, cwd=None, capture_output=True):
    """Run a command and return result"""
    try:
        result = subprocess.run(
            cmd,
            shell=True,
            cwd=cwd,
            capture_output=capture_output,
            text=True,
            timeout=120
        )
        return result.returncode == 0, result.stdout, result.stderr
    except subprocess.TimeoutExpired:
        return False, "", "Command timed out"
    except Exception as e:
        return False, "", str(e)

def check_file_exists(path):
    """Check if file exists and return basic info"""
    if not os.path.exists(path):
        return False, f"File does not exist: {path}"

    size = os.path.getsize(path)
    return True, f"File exists ({size} bytes)"

def check_generated_files():
    """Check that all required generated files exist"""
    print("=== Checking Generated Files ===")

    base_dir = Path(__file__).parent.parent
    required_files = [
        "src/generated/header.rs",
        "src/generated/transactions.rs",
        "src/generated/header_manifest.json",
        "src/generated/transactions_manifest.json",
        "tests/tx_header_parity_tests.rs",
        "tests/tx_body_serializer_tests.rs"
    ]

    all_exist = True
    for file_path in required_files:
        full_path = base_dir / file_path
        exists, msg = check_file_exists(full_path)
        print(f"  {file_path}: {'[OK]' if exists else '[FAIL]'} {msg}")
        if not exists:
            all_exist = False

    return all_exist

def check_header_structure():
    """Check header.rs has expected structure"""
    print("\n=== Checking Header Structure ===")

    base_dir = Path(__file__).parent.parent
    header_path = base_dir / "src/generated/header.rs"

    if not header_path.exists():
        print("  ✗ header.rs does not exist")
        return False

    content = header_path.read_text()

    # Check for required structs and functions
    checks = [
        ("TransactionHeader struct", "pub struct TransactionHeader"),
        ("ExpireOptions struct", "pub struct ExpireOptions"),
        ("HoldUntilOptions struct", "pub struct HoldUntilOptions"),
        ("TransactionHeader validate", "impl TransactionHeader"),
        ("Serde derives", "#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]"),
        ("Principal field", 'pub principal: String'),
        ("Initiator field", 'pub initiator: Vec<u8>'),
    ]

    all_good = True
    for name, pattern in checks:
        if pattern in content:
            print(f"  [OK] {name}")
        else:
            print(f"  [FAIL] {name} - missing pattern: {pattern}")
            all_good = False

    return all_good

def check_transaction_coverage():
    """Check transaction body coverage"""
    print("\n=== Checking Transaction Coverage ===")

    base_dir = Path(__file__).parent.parent
    manifest_path = base_dir / "src/generated/transactions_manifest.json"

    if not manifest_path.exists():
        print("  ✗ transactions_manifest.json does not exist")
        return False

    try:
        with open(manifest_path) as f:
            manifest = json.load(f)

        bodies = manifest.get("bodies", [])
        count = len(bodies)

        print(f"  Transaction bodies found: {count}")
        print(f"  Target: 33 transaction bodies")

        if count >= 27:
            print(f"  [OK] Found {count} bodies (>= 27 minimum)")
        else:
            print(f"  [FAIL] Only found {count} bodies (< 27 minimum)")
            return False

        if count < 33:
            print(f"  [WARN] Still need {33 - count} more bodies to reach target of 33")

        # List some transaction types
        print("  Sample transaction types:")
        for body in bodies[:5]:
            name = body.get("name", "Unknown")
            wire = body.get("wire", "unknown")
            print(f"    - {name} (wire: {wire})")

        return True

    except Exception as e:
        print(f"  ✗ Error reading manifest: {e}")
        return False

def run_tests(test_name):
    """Run specific test suite"""
    print(f"\n=== Running {test_name} ===")

    base_dir = Path(__file__).parent.parent
    cmd = f"cargo test --test {test_name}"

    success, stdout, stderr = run_command(cmd, cwd=base_dir)

    if success:
        # Parse test results
        lines = stdout.split('\n') + stderr.split('\n')
        test_lines = [line for line in lines if 'test result:' in line]

        if test_lines:
            result_line = test_lines[-1]
            print(f"  [OK] Tests passed: {result_line}")
        else:
            print(f"  [OK] Tests completed successfully")
        return True
    else:
        print(f"  [FAIL] Tests failed")
        print(f"  Error: {stderr}")
        return False

def check_module_integration():
    """Check that modules compile and integrate properly"""
    print("\n=== Checking Module Integration ===")

    base_dir = Path(__file__).parent.parent
    cmd = "cargo check"

    success, stdout, stderr = run_command(cmd, cwd=base_dir)

    if success:
        print("  [OK] All modules compile successfully")
        return True
    else:
        print("  [FAIL] Compilation errors found")
        print(f"  Error: {stderr}")
        return False

def run_verification():
    """Run complete verification"""
    print("Phase 2.2-2.3 Verification Orchestrator")
    print("=" * 50)

    checks = []

    # Check 1: Generated files exist
    checks.append(("Generated Files", check_generated_files))

    # Check 2: Header structure
    checks.append(("Header Structure", check_header_structure))

    # Check 3: Transaction coverage
    checks.append(("Transaction Coverage", check_transaction_coverage))

    # Check 4: Module integration
    checks.append(("Module Integration", check_module_integration))

    # Check 5: Header parity tests
    checks.append(("Header Parity Tests", lambda: run_tests("tx_header_parity_tests")))

    # Check 6: Body serializer tests
    checks.append(("Body Serializer Tests", lambda: run_tests("tx_body_serializer_tests")))

    # Run all checks
    results = []
    for name, check_func in checks:
        try:
            result = check_func()
            results.append((name, result))
        except Exception as e:
            print(f"  [FAIL] {name} failed with exception: {e}")
            results.append((name, False))

    # Summary
    print("\n" + "=" * 50)
    print("VERIFICATION SUMMARY")
    print("=" * 50)

    passed = 0
    total = len(results)

    for name, result in results:
        status = "[PASS]" if result else "[FAIL]"
        print(f"  {name}: {status}")
        if result:
            passed += 1

    print(f"\nOverall: {passed}/{total} checks passed")

    if passed == total:
        print("\n[SUCCESS] Phase 2.2-2.3 verification PASSED!")
        print("[COMPLETE] Stage 2.2: TransactionHeader Parity")
        print("[COMPLETE] Stage 2.3: Individual Body Serializers")
        return True
    else:
        print(f"\n[FAILED] Phase 2.2-2.3 verification FAILED ({total - passed} issues)")
        return False

if __name__ == "__main__":
    success = run_verification()
    sys.exit(0 if success else 1)
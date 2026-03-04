import requests
import concurrent.futures
import time
import random
from collections import Counter

# Configuration
URL = "http://localhost:3000/v1/quantum-verify"
THREADS = 20
TOTAL_REQUESTS = 100

# Mock PQC Constants (Aligned with Dilithium2)
PK_SIZE = 1312
SIG_SIZE = 2420

def generate_quantum_payload(is_replay=False, idx=0):
    """
    Generates a payload that simulates a real Post-Quantum signed transaction.
    """
    # 1. High-Entropy Data (Simulating an encrypted message)
    # This must be random to pass your Rust EntropyScanner
    message_data = [random.randint(0, 255) for _ in range(128)]
    
    # 2. Nonce Logic (Testing Replay Protection)
    nonce = "REPLAY_ATTACK_VECTOR_001" if is_replay else f"unique_tx_{idx}_{time.time_ns()}"
    
    # 3. Simulated Valid Structure
    # In a real scenario, 'signature' would be created using the Secret Key
    # For testing the 'logic flow', we send bytes that match the required length
    return {
        "nonce": nonce,
        "data": message_data,
        "signature": [random.randint(0, 255) for _ in range(SIG_SIZE)],
        "public_key": [random.randint(0, 255) for _ in range(PK_SIZE)]
    }

def send_request(is_replay=False, idx=0):
    payload = generate_quantum_payload(is_replay, idx)
    try:
        response = requests.post(URL, json=payload, timeout=10)
        # We categorize the response for the final report
        return response.status_code, response.json().get("message", "No Message")
    except Exception as e:
        return "ERROR", str(e)

print("🛡️  Starting Quantum-Fortress Sentinel Stress Test...")
print(f"📡 Target: {URL}")
print(f"🚀 Dispatching {TOTAL_REQUESTS} requests...")



start_time = time.perf_counter()

results = []
with concurrent.futures.ThreadPoolExecutor(max_workers=THREADS) as executor:
    # 50% Unique Requests, 50% Replay Attacks
    futures = [executor.submit(send_request, is_replay=(i >= TOTAL_REQUESTS//2), idx=i) for i in range(TOTAL_REQUESTS)]
    for f in concurrent.futures.as_completed(futures):
        results.append(f.result())

duration = time.perf_counter() - start_time

# Analysis
status_codes = Counter([r[0] for r in results])
messages = Counter([r[1] for r in results])

print("\n" + "="*45)
print("📊 PEN-TEST EXECUTION SUMMARY")
print("="*45)
print(f"⏱️  Duration:           {duration:.2f} seconds")
print(f"✅ Successful Codes:    {dict(status_codes)}")
print(f"💬 Server Responses:    {dict(messages)}")
print("="*45)

# Verification Logic
if messages.get("Replay attack detected") == TOTAL_REQUESTS // 2:
    print("🏆 VERDICT: Replay Protection is 100% OPERATIONAL.")
else:
    print("⚠️  VERDICT: Potential vulnerability in Replay Protection logic.")

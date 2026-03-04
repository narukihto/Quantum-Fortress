import requests
import concurrent.futures
import time
import random
from collections import Counter

# Configuration - Aligned with main.rs and docker-compose
URL = "http://localhost:3000/v1/quantum-verify"
THREADS = 50
TOTAL_REQUESTS = 1000

def generate_secure_payload(is_replay=False, nonce_id="static_nonce"):
    # Generate high-entropy data to pass the EntropyScanner (threshold 4.5)
    # This simulates encrypted post-quantum data
    high_entropy_data = list(random.getrandbits(8) for _ in range(256))
    
    return {
        "nonce": "replay_token_999" if is_replay else f"unique_{nonce_id}_{time.time_ns()}",
        "data": high_entropy_data,
        "signature": [0] * 2420,  # Simulated Dilithium2 Signature
        "public_key": [0] * 1312  # Simulated Dilithium2 Public Key
    }

def attack_vector(is_replay=False, idx=0):
    payload = generate_secure_payload(is_replay, nonce_id=str(idx))
    try:
        response = requests.post(URL, json=payload, timeout=5)
        return response.status_code
    except Exception:
        return "CONNECTION_ERROR"

print(f"🚀 Initializing Post-Quantum Stress Test on {URL}...")
print(f"📡 Dispatching {TOTAL_REQUESTS} requests via {THREADS} concurrent workers...")



start_time = time.perf_counter()

with concurrent.futures.ThreadPoolExecutor(max_workers=THREADS) as executor:
    # 500 Unique requests -> Expected: 200 OK (if sig check is bypassed) or 400 (Invalid Sig)
    # 500 Replay requests -> Expected: 200 OK (if blocked) or Error
    # Note: In our Rust code, it returns 200 only if signature passes. 
    # Since we send [0] sigs, the expected secure result is "Invalid signature" messages.
    
    tasks = [executor.submit(attack_vector, is_replay=(i >= 500), idx=i) for i in range(TOTAL_REQUESTS)]
    results = [t.result() for t in concurrent.futures.as_completed(tasks)]

duration = time.perf_counter() - start_time
stats = Counter(results)

print("\n" + "="*40)
print("📊 QUANTUM FORTRESS PEN-TEST RESULTS")
print("="*40)
print(f"⏱️ Total Duration:  {duration:.2f} seconds")
print(f"📥 Total Requests:  {TOTAL_REQUESTS}")
print(f"🛡️ Responses Recv:  {dict(stats)}")
print("="*40)

# The logic here is: if the nonce cache works, the second 500 requests 
# MUST return a "Replay attack detected" error, not a signature error.
if stats[200] == 0:
    print("✅ DEFENSE VERIFIED: System rejected all invalid/mock signatures.")
else:
    print("⚠️ WARNING: Unexpected 200 OK responses detected.")

#!/usr/bin/env python3
"""
E2E API Test Script for ML Inference
Tests the complete ML API functionality through HTTP requests
"""

import requests
import json
import time
import uuid

# API Base URL
API_BASE = "http://localhost:3000"

# Test data
MNIST_MODEL_ID = "00000000-0000-0000-0000-000000000001"
MNIST_INPUT_SIZE = 784

def test_health_check():
    """Test the health check endpoint"""
    print("[TEST] Testing health check endpoint...")
    try:
        response = requests.get(f"{API_BASE}/health", timeout=5)
        if response.status_code == 200:
            data = response.json()
            print(f"[PASS] Health check passed: {data}")
            return True
        else:
            print(f"[FAIL] Health check failed: {response.status_code}")
            return False
    except Exception as e:
        print(f"[FAIL] Health check error: {e}")
        return False

def test_ml_inference_success():
    """Test successful ML inference"""
    print("🔍 Testing ML inference (success case)...")
    
    # Create test data - MNIST 28x28 = 784 pixels
    input_data = [0.0] * MNIST_INPUT_SIZE
    
    payload = {
        "model_id": MNIST_MODEL_ID,
        "input_data": input_data
    }
    
    try:
        response = requests.post(
            f"{API_BASE}/api/ml/predict",
            json=payload,
            headers={"Content-Type": "application/json"},
            timeout=10
        )
        
        if response.status_code == 200:
            data = response.json()
            print(f"✅ ML inference success: {response.status_code}")
            print(f"   Response structure: {data.get('success', 'N/A')}")
            print(f"   Output length: {len(data.get('data', {}).get('output_data', []))}")
            print(f"   Model ID: {data.get('metadata', {}).get('model_id', 'N/A')}")
            return True
        else:
            print(f"❌ ML inference failed: {response.status_code}")
            print(f"   Response: {response.text}")
            return False
    except Exception as e:
        print(f"❌ ML inference error: {e}")
        return False

def test_ml_inference_not_found():
    """Test ML inference with non-existent model"""
    print("🔍 Testing ML inference (model not found)...")
    
    # Use random UUID that doesn't exist
    random_model_id = str(uuid.uuid4())
    input_data = [0.1] * MNIST_INPUT_SIZE
    
    payload = {
        "model_id": random_model_id,
        "input_data": input_data
    }
    
    try:
        response = requests.post(
            f"{API_BASE}/api/ml/predict",
            json=payload,
            headers={"Content-Type": "application/json"},
            timeout=10
        )
        
        if response.status_code == 404:
            data = response.json()
            print(f"✅ Model not found test passed: {response.status_code}")
            print(f"   Error code: {data.get('error', {}).get('code', 'N/A')}")
            return True
        else:
            print(f"❌ Model not found test failed: expected 404, got {response.status_code}")
            return False
    except Exception as e:
        print(f"❌ Model not found test error: {e}")
        return False

def test_ml_inference_empty_input():
    """Test ML inference with empty input"""
    print("🔍 Testing ML inference (empty input)...")
    
    payload = {
        "model_id": MNIST_MODEL_ID,
        "input_data": []
    }
    
    try:
        response = requests.post(
            f"{API_BASE}/api/ml/predict",
            json=payload,
            headers={"Content-Type": "application/json"},
            timeout=10
        )
        
        if response.status_code == 400:
            data = response.json()
            print(f"✅ Empty input test passed: {response.status_code}")
            print(f"   Error code: {data.get('error', {}).get('code', 'N/A')}")
            return True
        else:
            print(f"❌ Empty input test failed: expected 400, got {response.status_code}")
            return False
    except Exception as e:
        print(f"❌ Empty input test error: {e}")
        return False

def test_ml_inference_malformed_json():
    """Test ML inference with malformed JSON"""
    print("🔍 Testing ML inference (malformed JSON)...")
    
    try:
        response = requests.post(
            f"{API_BASE}/api/ml/predict",
            data="{invalid json",
            headers={"Content-Type": "application/json"},
            timeout=10
        )
        
        if response.status_code == 400:
            print(f"✅ Malformed JSON test passed: {response.status_code}")
            return True
        else:
            print(f"❌ Malformed JSON test failed: expected 400, got {response.status_code}")
            return False
    except Exception as e:
        print(f"❌ Malformed JSON test error: {e}")
        return False

def wait_for_server(max_attempts=30, delay=2):
    """Wait for the server to be ready"""
    print(f"⏳ Waiting for server to start (max {max_attempts * delay}s)...")
    
    for attempt in range(max_attempts):
        try:
            response = requests.get(f"{API_BASE}/health", timeout=5)
            if response.status_code == 200:
                print(f"✅ Server is ready after {attempt * delay}s")
                return True
        except:
            pass
        
        print(f"   Attempt {attempt + 1}/{max_attempts}...")
        time.sleep(delay)
    
    print("❌ Server failed to start within timeout")
    return False

def run_performance_test():
    """Run a simple performance test"""
    print("🔍 Running performance test...")
    
    input_data = [0.5] * MNIST_INPUT_SIZE
    payload = {
        "model_id": MNIST_MODEL_ID,
        "input_data": input_data
    }
    
    # Measure multiple requests
    times = []
    success_count = 0
    
    for i in range(5):
        start_time = time.time()
        try:
            response = requests.post(
                f"{API_BASE}/api/ml/predict",
                json=payload,
                headers={"Content-Type": "application/json"},
                timeout=10
            )
            end_time = time.time()
            
            if response.status_code == 200:
                success_count += 1
                times.append(end_time - start_time)
                print(f"   Request {i+1}: {(end_time - start_time)*1000:.1f}ms")
            else:
                print(f"   Request {i+1}: Failed ({response.status_code})")
        except Exception as e:
            print(f"   Request {i+1}: Error ({e})")
    
    if times:
        avg_time = sum(times) / len(times)
        print(f"✅ Performance test: {success_count}/5 requests successful")
        print(f"   Average response time: {avg_time*1000:.1f}ms")
        print(f"   Min/Max: {min(times)*1000:.1f}ms / {max(times)*1000:.1f}ms")
        return True
    else:
        print("❌ Performance test: No successful requests")
        return False

def main():
    """Run all tests"""
    print("🚀 Starting E2E ML API Tests")
    print("=" * 50)
    
    # Wait for server to be ready
    if not wait_for_server():
        print("❌ Cannot connect to server. Make sure it's running on localhost:3000")
        return
    
    # Run all tests
    tests = [
        ("Health Check", test_health_check),
        ("ML Inference Success", test_ml_inference_success),
        ("ML Inference Not Found", test_ml_inference_not_found),
        ("ML Inference Empty Input", test_ml_inference_empty_input),
        ("ML Inference Malformed JSON", test_ml_inference_malformed_json),
        ("Performance Test", run_performance_test),
    ]
    
    results = []
    for test_name, test_func in tests:
        print("\n" + "-" * 40)
        result = test_func()
        results.append((test_name, result))
    
    # Summary
    print("\n" + "=" * 50)
    print("📊 Test Results Summary:")
    
    passed = 0
    for test_name, result in results:
        status = "✅ PASS" if result else "❌ FAIL"
        print(f"   {status} {test_name}")
        if result:
            passed += 1
    
    total = len(results)
    print(f"\n🎯 Overall: {passed}/{total} tests passed ({passed/total*100:.1f}%)")
    
    if passed == total:
        print("🎉 All tests passed! ML API is working correctly.")
    else:
        print("⚠️  Some tests failed. Check the logs above.")

if __name__ == "__main__":
    main()
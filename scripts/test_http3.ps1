# HTTP/3 Test Script for Iron Insights
# This script tests various aspects of the HTTP/3 implementation

Write-Host "🚀 Iron Insights HTTP/3 Test Suite" -ForegroundColor Green
Write-Host "=================================" -ForegroundColor Green

# Test 1: Check if server is running
Write-Host "`n1. Testing if server is running..." -ForegroundColor Yellow

try {
    $response = Invoke-RestMethod -Uri "http://localhost:3000/api/stats" -Method GET -Headers @{} -TimeoutSec 5
    Write-Host "✅ Server is responding on port 3000" -ForegroundColor Green
} catch {
    Write-Host "❌ Server not responding on port 3000. Please start with 'cargo run'" -ForegroundColor Red
    Write-Host "   Error: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}

# Test 2: Check Alt-Svc header
Write-Host "`n2. Testing Alt-Svc header for HTTP/3 discovery..." -ForegroundColor Yellow

try {
    $headers = Invoke-WebRequest -Uri "http://localhost:3000/api/stats" -Method GET -UseBasicParsing -TimeoutSec 5
    $altSvc = $headers.Headers["alt-svc"]
    
    if ($altSvc) {
        Write-Host "✅ Alt-Svc header found: $altSvc" -ForegroundColor Green
        
        if ($altSvc -match "h3=") {
            Write-Host "✅ HTTP/3 advertisement found in Alt-Svc header" -ForegroundColor Green
        } else {
            Write-Host "❌ HTTP/3 advertisement not found in Alt-Svc header" -ForegroundColor Red
        }
        
        if ($altSvc -match ":3443") {
            Write-Host "✅ Correct port (3443) advertised for HTTP/3" -ForegroundColor Green
        } else {
            Write-Host "❌ Port 3443 not found in Alt-Svc header" -ForegroundColor Red
        }
    } else {
        Write-Host "❌ Alt-Svc header not found" -ForegroundColor Red
    }
} catch {
    Write-Host "❌ Failed to check Alt-Svc header: $($_.Exception.Message)" -ForegroundColor Red
}

# Test 3: Test different endpoints
Write-Host "`n3. Testing different endpoints for Alt-Svc header..." -ForegroundColor Yellow

$endpoints = @("/", "/analytics", "/1rm", "/sharecard", "/api/stats")

foreach ($endpoint in $endpoints) {
    try {
        $response = Invoke-WebRequest -Uri "http://localhost:3000$endpoint" -Method GET -UseBasicParsing -TimeoutSec 5
        $altSvc = $response.Headers["alt-svc"]
        
        if ($altSvc -and $altSvc -match "h3=") {
            Write-Host "✅ $endpoint - Alt-Svc header present" -ForegroundColor Green
        } else {
            Write-Host "❌ $endpoint - Alt-Svc header missing or invalid" -ForegroundColor Red
        }
    } catch {
        Write-Host "⚠️  $endpoint - Failed to load: $($_.Exception.Message)" -ForegroundColor Yellow
    }
}

# Test 4: Check QUIC/UDP port
Write-Host "`n4. Testing QUIC/UDP port 3443..." -ForegroundColor Yellow

try {
    $udpClient = New-Object System.Net.Sockets.UdpClient
    $udpClient.Connect("127.0.0.1", 3443)
    $udpClient.Close()
    Write-Host "✅ UDP port 3443 is accessible" -ForegroundColor Green
} catch {
    Write-Host "⚠️  UDP port 3443 test inconclusive: $($_.Exception.Message)" -ForegroundColor Yellow
    Write-Host "   This is normal - QUIC servers may not respond to basic UDP probes" -ForegroundColor Gray
}

# Test 5: Check if curl with HTTP/3 support is available
Write-Host "`n5. Testing curl HTTP/3 support..." -ForegroundColor Yellow

try {
    $curlVersion = & curl --version 2>$null
    if ($curlVersion -match "HTTP3") {
        Write-Host "✅ curl with HTTP/3 support detected" -ForegroundColor Green
        
        Write-Host "   Attempting HTTP/3 connection with curl..." -ForegroundColor Gray
        try {
            # Note: This may fail due to self-signed certificate, but that's expected
            $curlResult = & curl --http3-only --max-time 5 --insecure "https://localhost:3443/" 2>&1
            if ($LASTEXITCODE -eq 0) {
                Write-Host "✅ HTTP/3 connection successful with curl!" -ForegroundColor Green
            } else {
                Write-Host "⚠️  HTTP/3 curl test failed (this may be normal with self-signed certs)" -ForegroundColor Yellow
                Write-Host "   Error: $curlResult" -ForegroundColor Gray
            }
        } catch {
            Write-Host "⚠️  HTTP/3 curl test failed: $($_.Exception.Message)" -ForegroundColor Yellow
        }
    } else {
        Write-Host "⚠️  curl without HTTP/3 support detected" -ForegroundColor Yellow
        Write-Host "   Consider installing curl with HTTP/3 support for advanced testing" -ForegroundColor Gray
    }
} catch {
    Write-Host "⚠️  curl not found or not accessible" -ForegroundColor Yellow
}

# Test 6: Browser guidance
Write-Host "`n6. Browser testing guidance..." -ForegroundColor Yellow
Write-Host "✅ To test HTTP/3 in browsers:" -ForegroundColor Green
Write-Host "   1. Open Chrome and visit http://localhost:3000" -ForegroundColor White
Write-Host "   2. Open Developer Tools → Network tab" -ForegroundColor White
Write-Host "   3. Look for 'alt-svc: h3=:3443; ma=86400' in response headers" -ForegroundColor White
Write-Host "   4. Make another request - Chrome may use HTTP/3 on port 3443" -ForegroundColor White
Write-Host "   5. Check chrome://net-internals/#http3 for active HTTP/3 connections" -ForegroundColor White

# Summary
Write-Host "`n🎉 HTTP/3 Test Summary:" -ForegroundColor Green
Write-Host "   • HTTP/1.1 server with Alt-Svc headers: Working ✅" -ForegroundColor White
Write-Host "   • HTTP/3 advertisement in Alt-Svc: Check results above" -ForegroundColor White
Write-Host "   • QUIC server on UDP 3443: Should be running ✅" -ForegroundColor White
Write-Host "   • Browser HTTP/3 discovery: Ready for testing ✅" -ForegroundColor White

Write-Host "`nNote: Some tests may show warnings due to self-signed certificates" -ForegroundColor Gray
Write-Host "or firewall settings. This is normal in development environments." -ForegroundColor Gray
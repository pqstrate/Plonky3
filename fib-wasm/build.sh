#!/bin/bash

# Build the WASM module
wasm-pack build --target web --out-dir pkg

# Create a simple HTML example if it doesn't exist
if [ ! -f "index.html" ]; then
    cat > index.html << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Plonky3 Fibonacci WASM Prover</title>
    <style>
        body {
            font-family: Arial, sans-serif;
            max-width: 800px;
            margin: 0 auto;
            padding: 20px;
        }
        .container {
            background-color: #f5f5f5;
            padding: 20px;
            border-radius: 8px;
            margin: 10px 0;
        }
        input, button {
            padding: 8px;
            margin: 5px;
            font-size: 16px;
        }
        button {
            background-color: #007cba;
            color: white;
            border: none;
            border-radius: 4px;
            cursor: pointer;
        }
        button:hover {
            background-color: #005a8b;
        }
        .result {
            background-color: #e8f5e8;
            border: 1px solid #4caf50;
            padding: 10px;
            margin: 10px 0;
            border-radius: 4px;
        }
        .error {
            background-color: #ffe8e8;
            border: 1px solid #f44336;
            padding: 10px;
            margin: 10px 0;
            border-radius: 4px;
        }
    </style>
</head>
<body>
    <h1>Plonky3 Fibonacci WASM Prover</h1>
    
    <div class="container">
        <h3>Prover Information</h3>
        <div id="info"></div>
    </div>

    <div class="container">
        <h3>Calculate Fibonacci Number</h3>
        <input type="number" id="fib-n" placeholder="Enter n" value="10" min="0" max="50">
        <button onclick="calculateFib()">Calculate F(n)</button>
        <div id="fib-result"></div>
    </div>

    <div class="container">
        <h3>Generate and Verify Proof</h3>
        <p>Note: Proof generation uses powers of 2 for trace size.</p>
        <input type="number" id="proof-n" placeholder="Enter n (power of 2)" value="8" min="1" max="16">
        <input type="number" id="expected" placeholder="Expected F(n)" value="21">
        <button onclick="generateProof()">Generate Proof</button>
        <div id="proof-result"></div>
    </div>

    <script type="module">
        import init, { fibonacci, prove_fibonacci, get_prover_info } from './pkg/fib_wasm.js';
        
        async function run() {
            await init();
            
            // Display prover info
            document.getElementById('info').innerHTML = '<pre>' + get_prover_info() + '</pre>';
            
            // Make functions global
            window.fibonacci = fibonacci;
            window.prove_fibonacci = prove_fibonacci;
            window.get_prover_info = get_prover_info;
        }
        
        window.calculateFib = function() {
            const n = parseInt(document.getElementById('fib-n').value);
            const result = fibonacci(n);
            document.getElementById('fib-result').innerHTML = 
                `<div class="result">F(${n}) = ${result}</div>`;
        }
        
        window.generateProof = function() {
            const n = parseInt(document.getElementById('proof-n').value);
            const expected = parseInt(document.getElementById('expected').value);
            
            document.getElementById('proof-result').innerHTML = '<p>Generating proof... This may take a few seconds.</p>';
            
            try {
                // Use setTimeout to allow UI to update
                setTimeout(() => {
                    try {
                        const result = prove_fibonacci(n, expected);
                        document.getElementById('proof-result').innerHTML = 
                            `<div class="result">${result}</div>`;
                    } catch (error) {
                        document.getElementById('proof-result').innerHTML = 
                            `<div class="error">Error: ${error}</div>`;
                    }
                }, 100);
            } catch (error) {
                document.getElementById('proof-result').innerHTML = 
                    `<div class="error">Error: ${error}</div>`;
            }
        }
        
        run();
    </script>
</body>
</html>
EOF
fi

echo "WASM build complete! Open index.html in a web server to test."
echo "You can use: python3 -m http.server 8000"
let display = document.getElementById('display');
let currentInput = '0';
let previousInput = '';
let operator = '';

function updateDisplay() {
    display.value = currentInput;
}

function appendNumber(num) {
    if (currentInput === '0' || currentInput === 'Error') {
        currentInput = num;
    } else if (currentInput.length < 15) {
        currentInput += num;
    }
    updateDisplay();
}

function appendOperator(op) {
    if (previousInput !== '' && operator !== '') {
        calculate();
    }
    previousInput = currentInput;
    operator = op;
    currentInput = '0';
}

function appendDecimal() {
    if (!currentInput.includes('.')) {
        currentInput += '.';
    }
    updateDisplay();
}

function clearDisplay() {
    currentInput = '0';
    previousInput = '';
    operator = '';
    updateDisplay();
}

function calculate() {
    if (previousInput === '' || operator === '') return;
    
    let num1 = parseFloat(previousInput);
    let num2 = parseFloat(currentInput);
    let result;
    
    switch (operator) {
        case '+':
            result = num1 + num2;
            break;
        case '-':
            result = num1 - num2;
            break;
        case '*':
            result = num1 * num2;
            break;
        case '/':
            if (num2 === 0) {
                currentInput = 'Error';
                previousInput = '';
                operator = '';
                updateDisplay();
                return;
            }
            result = num1 / num2;
            break;
        default:
            return;
    }
    
    result = Math.round(result * 1000000000000) / 1000000000000;
    
    if (result.toString().length > 15) {
        currentInput = result.toExponential(5);
    } else {
        currentInput = result.toString();
    }
    
    previousInput = '';
    operator = '';
    updateDisplay();
}

document.addEventListener('keydown', (e) => {
    if (e.key >= '0' && e.key <= '9') {
        appendNumber(e.key);
    } else if (e.key === '+') {
        appendOperator('+');
    } else if (e.key === '-') {
        appendOperator('-');
    } else if (e.key === '*') {
        appendOperator('*');
    } else if (e.key === '/') {
        appendOperator('/');
    } else if (e.key === '.') {
        appendDecimal();
    } else if (e.key === 'Enter' || e.key === '=') {
        calculate();
    } else if (e.key === 'Escape' || e.key === 'c') {
        clearDisplay();
    } else if (e.key === 'Backspace') {
        if (currentInput.length > 1) {
            currentInput = currentInput.slice(0, -1);
        } else {
            currentInput = '0';
        }
        updateDisplay();
    }
});

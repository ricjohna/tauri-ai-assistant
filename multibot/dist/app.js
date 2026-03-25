let invoke, listen;
let charName = 'MultiBot';
let thinkingInterval = null;
let dotIndex = 0;
let idleTimer = null;
let idleTimeout = 20000;
let isConfigured = false;

document.addEventListener('DOMContentLoaded', async () => {
    await initTauri();
    setupEventListeners();
    await checkSetup();
});

async function initTauri() {
    if (window.__TAURI__) {
        invoke = window.__TAURI__.core.invoke;
    } else {
        console.error('Tauri not loaded');
        addDebugLog('Error: Tauri not loaded');
    }
}

async function checkSetup() {
    if (!invoke) {
        hideOverlay();
        loadGreetingFallback();
        return;
    }
    
    try {
        isConfigured = await invoke('is_configured');
        addDebugLog(`Configured: ${isConfigured}`);
        
        if (isConfigured) {
            hideOverlay();
            await loadGreeting();
            updateIdleTimer();
        } else {
            showOverlay();
        }
    } catch (error) {
        addDebugLog(`Setup check error: ${error}`);
        isConfigured = false;
        showOverlay();
    }
}

function showOverlay() {
    const overlay = document.getElementById('setup-overlay');
    if (overlay) overlay.classList.add('visible');
}

function hideOverlay() {
    const overlay = document.getElementById('setup-overlay');
    if (overlay) overlay.classList.remove('visible');
}

function loadGreetingFallback() {
    addMessage('MultiBot', 'Hello! How can I help you?', 'ai');
}

async function loadGreeting() {
    if (!invoke) {
        loadGreetingFallback();
        return;
    }
    
    try {
        const personality = await invoke('get_personality');
        charName = personality.name || 'MultiBot';
        document.getElementById('char-name').textContent = charName;
        addMessage(charName, personality.greeting || 'Hello!', 'ai');
        addDebugLog('Greeting loaded successfully');
    } catch (error) {
        console.error('Error loading personality:', error);
        addDebugLog(`Error loading personality: ${error}`);
        addMessage('MultiBot', 'Hello! How can I help you?', 'ai');
    }
}

function setupEventListeners() {
    const input = document.getElementById('message-input');
    const sendBtn = document.getElementById('send-btn');
    const debugCheckbox = document.getElementById('debug-checkbox');
    const debugWindow = document.getElementById('debug-window');
    const debugClose = document.getElementById('debug-close');
    const debugClear = document.getElementById('debug-clear');
    
    const tabChat = document.getElementById('tab-chat');
    const tabSettings = document.getElementById('tab-settings');
    const saveSettingsBtn = document.getElementById('save-settings-btn');
    const setupSaveBtn = document.getElementById('setup-save-btn');

    if (sendBtn) {
        sendBtn.addEventListener('click', sendMessage);
    }
    
    if (input) {
        input.addEventListener('keypress', (e) => {
            if (e.key === 'Enter') sendMessage();
        });
        input.focus();
    }

    if (debugCheckbox && debugWindow) {
        debugCheckbox.addEventListener('change', () => {
            debugWindow.classList.toggle('visible', debugCheckbox.checked);
        });
    }

    if (debugClose) {
        debugClose.addEventListener('click', () => {
            if (debugCheckbox) debugCheckbox.checked = false;
            debugWindow.classList.remove('visible');
        });
    }

    if (debugClear) {
        debugClear.addEventListener('click', () => {
            document.getElementById('debug-content').textContent = '';
        });
    }

    const debugHeader = document.querySelector('.debug-header');
    if (debugHeader && debugWindow) {
        let isDragging = false;
        let dragOffsetX = 0;
        let dragOffsetY = 0;

        debugHeader.addEventListener('mousedown', (e) => {
            isDragging = true;
            dragOffsetX = e.clientX - debugWindow.offsetLeft;
            dragOffsetY = e.clientY - debugWindow.offsetTop;
            debugWindow.style.transition = 'none';
        });

        document.addEventListener('mousemove', (e) => {
            if (!isDragging) return;
            debugWindow.style.left = (e.clientX - dragOffsetX) + 'px';
            debugWindow.style.top = (e.clientY - dragOffsetY) + 'px';
            debugWindow.style.right = 'auto';
            debugWindow.style.bottom = 'auto';
        });

        document.addEventListener('mouseup', () => {
            isDragging = false;
            debugWindow.style.transition = '';
        });
    }

    if (tabChat && tabSettings) {
        tabChat.addEventListener('click', () => {
            tabChat.classList.add('active');
            tabSettings.classList.remove('active');
            document.getElementById('chat-view').classList.remove('hidden');
            document.getElementById('settings-view').classList.add('hidden');
        });
        
        tabSettings.addEventListener('click', async () => {
            tabSettings.classList.add('active');
            tabChat.classList.remove('active');
            document.getElementById('settings-view').classList.remove('hidden');
            document.getElementById('chat-view').classList.add('hidden');
            await loadSettingsToForm();
        });
    }

    if (saveSettingsBtn) {
        saveSettingsBtn.addEventListener('click', saveSettings);
    }

    if (setupSaveBtn) {
        setupSaveBtn.addEventListener('click', handleSetupSave);
    }
    
    addDebugLog('Event listeners initialized');
}

async function handleSetupSave() {
    const apiKeyInput = document.getElementById('setup-api-key');
    const apiKey = apiKeyInput?.value.trim();
    
    if (!apiKey || apiKey === '') {
        alert('Please enter your API key');
        return;
    }
    
    const saveBtn = document.getElementById('setup-save-btn');
    if (saveBtn) saveBtn.disabled = true;
    saveBtn.textContent = 'Saving...';
    
    try {
        const personality = await invoke('get_personality');
        
        await invoke('save_settings', {
            apiKey: apiKey,
            personality: personality
        });
        
        isConfigured = true;
        hideOverlay();
        await loadGreeting();
        updateIdleTimer();
        addDebugLog('Settings saved successfully');
    } catch (error) {
        alert('Error saving settings: ' + error);
        addDebugLog(`Save error: ${error}`);
    }
    
    if (saveBtn) {
        saveBtn.disabled = false;
        saveBtn.textContent = 'Get Started';
    }
}

async function loadSettingsToForm() {
    if (!invoke) return;
    
    try {
        const settings = await invoke('load_settings');
        const p = settings.personality;
        
        document.getElementById('api-key').value = settings.api_key || '';
        document.getElementById('bot-name').value = p.name || '';
        document.getElementById('bot-creator').value = p.creator || '';
        document.getElementById('bot-greeting').value = p.greeting || '';
        document.getElementById('bot-system-prompt').value = p.system_prompt || '';
        document.getElementById('bot-description').value = p.description || '';
        
        const style = p.speaking_style || {};
        document.getElementById('use-emoji').checked = style.use_emoji !== false;
        document.getElementById('casual-speech').checked = style.casual_speech !== false;
        document.getElementById('exclamation-heavy').checked = style.exclamation_heavy !== false;
        
        document.getElementById('catchphrases').value = (p.catchphrases || []).join(', ');
        document.getElementById('idle-messages').value = (p.idle_messages || []).join(', ');
        
        const emo = p.emotional_responses || {};
        document.getElementById('excited-response').value = emo.excited_response || '';
        document.getElementById('sad-response').value = emo.sad_response || '';
        document.getElementById('confused-response').value = emo.confused_response || '';
        document.getElementById('angry-response').value = emo.angry_response || '';
        document.getElementById('love-response').value = emo.love_response || '';
        
        addDebugLog('Settings loaded to form');
    } catch (error) {
        addDebugLog(`Load settings error: ${error}`);
    }
}

async function saveSettings() {
    if (!invoke) return;
    
    const saveBtn = document.getElementById('save-settings-btn');
    const statusSpan = document.getElementById('save-status');
    
    if (saveBtn) saveBtn.disabled = true;
    
    try {
        const personality = {
            name: document.getElementById('bot-name').value || 'MultiBot',
            creator: document.getElementById('bot-creator').value || '',
            description: document.getElementById('bot-description').value || '',
            avatar: '',
            greeting: document.getElementById('bot-greeting').value || '',
            system_prompt: document.getElementById('bot-system-prompt').value || '',
            traits: [],
            speaking_style: {
                use_emoji: document.getElementById('use-emoji').checked,
                casual_speech: document.getElementById('casual-speech').checked,
                exclamation_heavy: document.getElementById('exclamation-heavy').checked,
                max_response_length: 500
            },
            emotional_responses: {
                excited_keywords: [],
                excited_response: document.getElementById('excited-response').value || '',
                confused_keywords: [],
                confused_response: document.getElementById('confused-response').value || '',
                sad_keywords: [],
                sad_response: document.getElementById('sad-response').value || '',
                angry_keywords: [],
                angry_response: document.getElementById('angry-response').value || '',
                love_keywords: [],
                love_response: document.getElementById('love-response').value || ''
            },
            catchphrases: parseListInput(document.getElementById('catchphrases').value),
            catchphrase_chance: 0.1,
            idle_messages: parseListInput(document.getElementById('idle-messages').value),
            idle_timeout_seconds: 60,
            conversation_history_limit: 10
        };
        
        const apiKey = document.getElementById('api-key').value;
        
        await invoke('save_settings', {
            apiKey: apiKey,
            personality: personality
        });
        
        charName = personality.name;
        document.getElementById('char-name').textContent = charName;
        
        if (statusSpan) {
            statusSpan.textContent = 'Saved!';
            statusSpan.className = 'save-status success';
            setTimeout(() => {
                statusSpan.textContent = '';
            }, 2000);
        }
        
        await updateIdleTimer();
        addDebugLog('Settings saved successfully');
    } catch (error) {
        if (statusSpan) {
            statusSpan.textContent = 'Error: ' + error;
            statusSpan.className = 'save-status error';
        }
        addDebugLog(`Save error: ${error}`);
    }
    
    if (saveBtn) saveBtn.disabled = false;
}

function parseListInput(value) {
    if (!value) return [];
    return value.split(',').map(s => s.trim()).filter(s => s.length > 0);
}

async function sendMessage() {
    if (!invoke) {
        addDebugLog('Error: Tauri not available');
        return;
    }
    
    if (!isConfigured) {
        addMessage('System', 'Please set up your API key in Settings first!', 'system');
        return;
    }
    
    const input = document.getElementById('message-input');
    const sendBtn = document.getElementById('send-btn');
    const message = input.value.trim();
    
    if (!message) return;
    
    addDebugLog(`Sending message: ${message}`);
    
    addMessage('You', message, 'user');
    
    input.value = '';
    input.disabled = true;
    if (sendBtn) sendBtn.disabled = true;
    
    showThinking();
    resetIdleTimer();
    
    try {
        const messages = await invoke('process_message', { userInput: message });
        
        messages.forEach(msg => {
            if (msg.msg_type !== 'user') {
                addMessage(msg.sender, msg.message, msg.msg_type || msg.type);
            }
        });
        addDebugLog(`Received ${messages.length} messages`);
    } catch (error) {
        console.error('Error sending message:', error);
        addDebugLog(`Error: ${error}`);
        addMessage('System', `Error: ${error}`, 'system');
    }
    
    hideThinking();
    input.disabled = false;
    if (sendBtn) sendBtn.disabled = false;
    input.focus();
}

function addMessage(sender, message, type) {
    const chatArea = document.getElementById('chat-area');
    if (!chatArea) return;
    
    const messageEl = document.createElement('div');
    messageEl.className = `message ${type || 'ai'}`;
    
    if (type !== 'user') {
        const nameEl = document.createElement('div');
        nameEl.className = 'sender-name';
        nameEl.textContent = sender;
        messageEl.appendChild(nameEl);
    }
    
    const bubbleEl = document.createElement('div');
    bubbleEl.className = 'bubble';
    bubbleEl.textContent = message;
    messageEl.appendChild(bubbleEl);
    
    chatArea.appendChild(messageEl);
    messageEl.scrollIntoView({ behavior: 'smooth', block: 'end' });
}

function showThinking() {
    const indicator = document.getElementById('thinking-indicator');
    const statusText = document.getElementById('status-text');
    
    if (indicator) indicator.classList.add('active');
    if (statusText) {
        statusText.textContent = 'Thinking...';
        statusText.className = 'status-working';
    }
    
    dotIndex = 0;
    thinkingInterval = setInterval(() => {
        dotIndex = (dotIndex + 1) % 3;
        const dots = ['●○○', '○●○', '○○●'];
        const dotsEl = indicator?.querySelector('.thinking-dots');
        if (dotsEl) dotsEl.textContent = dots[dotIndex];
    }, 500);
}

function hideThinking() {
    const indicator = document.getElementById('thinking-indicator');
    const statusText = document.getElementById('status-text');
    
    if (indicator) indicator.classList.remove('active');
    if (statusText) {
        statusText.textContent = 'Idle';
        statusText.className = 'status-idle';
    }
    
    if (thinkingInterval) {
        clearInterval(thinkingInterval);
        thinkingInterval = null;
    }
}

function resetIdleTimer() {
    if (idleTimer) {
        clearTimeout(idleTimer);
    }
    idleTimer = setTimeout(sendIdleMessage, idleTimeout);
}

async function sendIdleMessage() {
    if (!invoke || !isConfigured) return;
    
    addDebugLog('Idle timeout - sending idle message');
    try {
        const message = await invoke('get_idle_message');
        addMessage(message.sender, message.message, message.msg_type || message.type);
        resetIdleTimer();
    } catch (error) {
        addDebugLog(`Idle error: ${error}`);
        resetIdleTimer();
    }
}

async function updateIdleTimer() {
    if (!invoke) return;
    
    try {
        const personality = await invoke('get_personality');
        idleTimeout = (personality.idle_timeout_seconds || 60) * 1000;
        charName = personality.name || 'MultiBot';
        document.getElementById('char-name').textContent = charName;
    } catch (error) {
        addDebugLog(`Idle timer config error: ${error}`);
    }
    resetIdleTimer();
}

function addDebugLog(message) {
    const debugContent = document.getElementById('debug-content');
    if (!debugContent) return;
    
    const timestamp = new Date().toLocaleTimeString('en-US', { hour12: false });
    debugContent.textContent += `[${timestamp}] ${message}\n`;
    debugContent.scrollTop = debugContent.scrollHeight;
}
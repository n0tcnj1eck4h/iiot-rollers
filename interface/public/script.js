
document.addEventListener('DOMContentLoaded', function() {
    // DOM Elements
    const positionSlider = document.getElementById('position-slider');
    const speedSlider = document.getElementById('speed-slider');
    const applyBtn = document.getElementById('apply-btn');
    const refreshBtn = document.getElementById('refresh-btn');
    const presetButtons = document.querySelectorAll('.preset');
    const blindsPosition = document.getElementById('blinds-position');
    const positionDisplay = document.getElementById('position-display');
    const currentPositionDisplay = document.getElementById('current-position');
    const currentSpeedDisplay = document.getElementById('current-speed');
    const lastUpdateDisplay = document.getElementById('last-update');
    const connectionStatus = document.getElementById('connection-status');

    // API Configuration
    const API_BASE_URL = 'http://localhost:3000';
    const BLINDS_CONTROL_ENDPOINT = `${API_BASE_URL}/api/blinds/control`;
    const BLINDS_STATUS_ENDPOINT = `${API_BASE_URL}/api/blinds/status`;

    // Update blinds visualization
    function updateBlindsVisualization(position) {
        blindsPosition.style.height = `${100 - position}%`;
        positionDisplay.textContent = `${position}%`;
    }

    // Update status display
    function updateStatusDisplay(data) {
        currentPositionDisplay.textContent = `${data.position}%`;
        currentSpeedDisplay.textContent = data.speed ? `${data.speed}%` : 'Not set';
        lastUpdateDisplay.textContent = new Date().toLocaleTimeString();
        
        // Add animation
        const elements = [currentPositionDisplay, currentSpeedDisplay, lastUpdateDisplay];
        elements.forEach(el => {
            el.classList.add('fade-in');
            setTimeout(() => el.classList.remove('fade-in'), 500);
        });
    }

    // Set connection status
    function setConnectionStatus(connected) {
        connectionStatus.classList.remove('connected', 'disconnected');
        connectionStatus.classList.add(connected ? 'connected' : 'disconnected');
        connectionStatus.querySelector('.status-text').textContent = 
            connected ? 'Connected' : 'Disconnected';
    }

    // Send command to server
    async function sendBlindsCommand(position, speed) {
        try {
            const response = await fetch(BLINDS_CONTROL_ENDPOINT, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    position: parseInt(position),
                    speed: speed ? parseInt(speed) : null
                })
            });

            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }

            const data = await response.json();
            updateBlindsVisualization(data.position);
            setConnectionStatus(true);
            return data;
        } catch (error) {
            console.error('Error sending command:', error);
            setConnectionStatus(false);
            throw error;
        }
    }

    // Get blinds status
    async function getBlindsStatus() {
        try {
            const response = await fetch(BLINDS_STATUS_ENDPOINT);
            
            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }

            const data = await response.json();
            updateStatusDisplay(data);
            setConnectionStatus(true);
            return data;
        } catch (error) {
            console.error('Error getting status:', error);
            setConnectionStatus(false);
            throw error;
        }
    }

    // Event Listeners
    positionSlider.addEventListener('input', function() {
        updateBlindsVisualization(this.value);
    });

    applyBtn.addEventListener('click', async function() {
        this.disabled = true;
        this.innerHTML = '<i class="fas fa-spinner fa-spin"></i> Applying...';
        
        try {
            await sendBlindsCommand(positionSlider.value, speedSlider.value);
        } finally {
            this.disabled = false;
            this.innerHTML = 'Apply';
        }
    });

    refreshBtn.addEventListener('click', async function() {
        this.disabled = true;
        this.innerHTML = '<i class="fas fa-spinner fa-spin"></i> Refreshing...';
        
        try {
            await getBlindsStatus();
        } finally {
            this.disabled = false;
            this.innerHTML = '<i class="fas fa-sync-alt"></i> Refresh Status';
        }
    });

    presetButtons.forEach(button => {
        button.addEventListener('click', function() {
            const position = this.dataset.position;
            positionSlider.value = position;
            updateBlindsVisualization(position);
            sendBlindsCommand(position, speedSlider.value);
        });
    });

    // Initialize
    updateBlindsVisualization(positionSlider.value);
    getBlindsStatus();
    
    // Simulate connection status for demo (remove in production)
    setConnectionStatus(true);
});

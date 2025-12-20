/**
 * Spinning Wheel Implementation
 */

class SpinningWheel {
    constructor(canvasId, containerId) {
        this.canvas = document.getElementById(canvasId);
        this.container = document.getElementById(containerId);
        this.ctx = this.canvas.getContext('2d');

        // Wheel properties
        this.radius = 250;
        this.canvas.width = this.radius * 2;
        this.canvas.height = this.radius * 2;

        this.items = [];
        this.currentRotation = 0;
        this.isSpinning = false;
        this.isShowingResult = false;
        this.spinDuration = 4000; // 4 seconds
        this.hideTimeout = null;
        this.resultTimeout = null;
        this.colors = [
            '#FF6B6B', '#4ECDC4', '#45B7D1', '#FFA07A',
            '#98D8C8', '#F7DC6F', '#BB8FCE', '#85C1E2',
            '#F8B739', '#52C7B8', '#E74C3C', '#3498DB'
        ];
    }

    /**
     * Set the items for the wheel
     */
    setItems(items) {
        this.items = items;
        this.draw();
    }

    /**
     * Draw the wheel
     */
    draw() {
        if (this.items.length === 0) return;

        const ctx = this.ctx;
        const centerX = this.radius;
        const centerY = this.radius;
        const sliceAngle = (2 * Math.PI) / this.items.length;

        // Clear canvas
        ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);

        // Save context state
        ctx.save();
        ctx.translate(centerX, centerY);
        ctx.rotate(this.currentRotation);

        // Draw slices
        this.items.forEach((item, index) => {
            const startAngle = index * sliceAngle - Math.PI / 2;
            const endAngle = startAngle + sliceAngle;
            const color = this.colors[index % this.colors.length];

            // Draw slice
            ctx.beginPath();
            ctx.moveTo(0, 0);
            ctx.arc(0, 0, this.radius, startAngle, endAngle);
            ctx.fillStyle = color;
            ctx.fill();
            ctx.strokeStyle = '#ffffff';
            ctx.lineWidth = 3;
            ctx.stroke();

            // Draw text
            ctx.save();
            ctx.rotate(startAngle + sliceAngle / 2);
            ctx.textAlign = 'center';
            ctx.fillStyle = '#ffffff';
            ctx.font = 'bold 20px Arial';
            ctx.shadowColor = 'rgba(0, 0, 0, 0.5)';
            ctx.shadowBlur = 3;
            ctx.fillText(item, this.radius * 0.65, 5);
            ctx.restore();
        });

        // Draw center circle
        ctx.beginPath();
        ctx.arc(0, 0, 30, 0, 2 * Math.PI);
        ctx.fillStyle = '#ffffff';
        ctx.fill();
        ctx.strokeStyle = '#000000';
        ctx.lineWidth = 3;
        ctx.stroke();

        ctx.restore();
    }

    /**
     * Spin the wheel and return the result
     */
    async spin() {
        if (this.isSpinning || this.items.length === 0) return null;

        this.isSpinning = true;
        this.container.classList.remove('hidden');

        // Random number of full rotations (3-6) plus a random offset
        const fullRotations = 3 + Math.random() * 3;
        const randomOffset = Math.random() * 2 * Math.PI;
        const totalRotation = fullRotations * 2 * Math.PI + randomOffset;

        const startTime = Date.now();
        const startRotation = this.currentRotation;

        return new Promise((resolve) => {
            const animate = () => {
                const elapsed = Date.now() - startTime;
                const progress = Math.min(elapsed / this.spinDuration, 1);

                // Easing function (ease-out)
                const easeOut = 1 - Math.pow(1 - progress, 3);

                this.currentRotation = startRotation + totalRotation * easeOut;
                this.draw();

                if (progress < 1) {
                    requestAnimationFrame(animate);
                } else {
                    // Normalize rotation
                    this.currentRotation = this.currentRotation % (2 * Math.PI);

                    // Calculate winner
                    const sliceAngle = (2 * Math.PI) / this.items.length;
                    // Pointer is at the top, so we need to adjust for rotation
                    const normalizedRotation = (2 * Math.PI - (this.currentRotation % (2 * Math.PI))) % (2 * Math.PI);
                    const winnerIndex = Math.floor(normalizedRotation / sliceAngle);
                    const winner = this.items[winnerIndex];

                    // Set isShowingResult BEFORE setting isSpinning to false
                    // This prevents the timing gap where both flags are false
                    this.isShowingResult = true;
                    this.isSpinning = false;
                    resolve(winner);
                }
            };

            animate();
        });
    }

    /**
     * Show the result
     */
    showResult(result) {
        // isShowingResult is already set to true in spin() before this is called
        const resultDiv = document.getElementById('wheel-result');
        resultDiv.textContent = result;
        resultDiv.classList.remove('hidden');

        // Clear any existing result timeout
        if (this.resultTimeout) {
            clearTimeout(this.resultTimeout);
        }

        this.resultTimeout = setTimeout(() => {
            resultDiv.classList.add('fade-out');
            setTimeout(() => {
                resultDiv.classList.add('hidden');
                resultDiv.classList.remove('fade-out');
                this.isShowingResult = false;
            }, 500);
        }, 3000);
    }

    /**
     * Check if the wheel is busy (spinning or showing result)
     */
    isBusy() {
        return this.isSpinning || this.isShowingResult;
    }

    /**
     * Hide the wheel
     */
    hide() {
        // Clear any existing hide timeout
        if (this.hideTimeout) {
            clearTimeout(this.hideTimeout);
        }

        this.hideTimeout = setTimeout(() => {
            this.container.classList.add('hidden');
        }, 4000);
    }

    /**
     * Spin and show result
     */
    async spinAndShow(wheelData) {
        // Don't start a new spin if one is already in progress or showing result
        if (this.isBusy()) {
            console.warn('Wheel is busy (spinning or showing result), ignoring new spin request');
            return null;
        }

        // Reset wheel state for new spin (clears any pending timeouts)
        this.reset();

        const result = await this.spin();
        if (result) {
            this.showResult(result);
            this.hide();

            // Send result back to backend
            if (window.sendToBackend) {
                const action = wheelData?.actions?.[result];
                window.sendToBackend({
                    type: 'wheel_result',
                    result: result,
                    action: action
                });
            }
        }
        return result;
    }

    /**
     * Reset wheel state
     */
    reset() {
        // Clear any pending timeouts
        if (this.hideTimeout) {
            clearTimeout(this.hideTimeout);
            this.hideTimeout = null;
        }
        if (this.resultTimeout) {
            clearTimeout(this.resultTimeout);
            this.resultTimeout = null;
        }

        // Hide result if visible
        const resultDiv = document.getElementById('wheel-result');
        resultDiv.classList.add('hidden');
        resultDiv.classList.remove('fade-out');
        resultDiv.textContent = '';

        // Reset state flags
        this.isShowingResult = false;

        // Reset container visibility
        this.container.classList.remove('hidden');
    }
}

// Export for use in main.js
window.SpinningWheel = SpinningWheel;

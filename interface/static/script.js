let shutters = [];

function renderShutters() {
  const dashboard = document.getElementById("shutterDashboard");
  dashboard.innerHTML = "";

  shutters.forEach((shutter) => {
    const widget = document.createElement("div");
    widget.className = "shutter-widget";
    widget.innerHTML = `
                    <div class="widget-header">
                        <div class="shutter-name">
                            <div class="shutter-icon">
                                <i class="fas fa-blinds"></i>
                            </div>
                            ${shutter.name}
                        </div>
                        <button class="action-btn delete-btn" onclick="deleteShutter(${shutter.id})">
                            <i class="fas fa-trash"></i>
                        </button>
                    </div>
                    
                    <div class="shutter-visual">
                        <div class="shutter-lines" id="visual-${shutter.id}"></div>
                    </div>
                    
                    <div class="percentage-display">${shutter.position}%</div>
                    
                    <div class="slider-container">
                        <input type="range" min="0" max="100" value="${shutter.position}" 
                            class="slider" id="slider-${shutter.id}" 
                            oninput="updateShutterPosition(${shutter.id}, this.value)">
                    </div>
                    
                    <div class="preset-buttons">
                        <button class="preset-btn" onclick="setShutterPosition(${shutter.id}, 0)">Zamknij</button>
                        <button class="preset-btn" onclick="setShutterPosition(${shutter.id}, 100)">Otwórz</button>
                    </div>
                    
                `;
    dashboard.appendChild(widget);
    updateShutterVisual(shutter.id, shutter.position);
  });
}

function updateShutterPosition(id, position) {
  const shutter = shutters.find((s) => s.id === id);
  shutter.position = position;
  document.querySelector(`#slider-${id}`).value = position;
  document.querySelector(
    `#visual-${id}`,
  ).parentNode.nextElementSibling.textContent = `${position}%`;
  updateShutterVisual(id, position);
}

// Set shutter position (with apply button)
function setShutterPosition(id, position) {
  const shutter = shutters.find((s) => s.id === id);
  if (shutter) {
    shutter.position = parseInt(position);
    renderShutters();
  }
}

function updateShutterVisual(id, position) {
  const visual = document.getElementById(`visual-${id}`);
  if (!visual) return;

  visual.innerHTML = "";

  const closedPercentage = 100 - position;
  const numLines = 15;

  for (let i = 0; i < numLines; i++) {
    const line = document.createElement("div");
    line.className = "shutter-line";

    const topPosition = (i / (numLines - 1)) * 100;
    line.style.top = `${topPosition}%`;

    const minHeight = 2;
    const maxHeight = 10;
    const height =
      minHeight + (maxHeight - minHeight) * (closedPercentage / 100);
    line.style.height = `${height}px`;

    const opacity = 0.2 + 0.8 * (closedPercentage / 100);
    line.style.opacity = opacity;

    visual.appendChild(line);
  }
}

function deleteShutter(id) {
  if (confirm("Are you sure you want to remove this shutter?")) {
    shutters = shutters.filter((shutter) => shutter.id !== id);
    renderShutters();
  }
}

document
  .getElementById("addShutterForm")
  .addEventListener("submit", async function (e) {
    e.preventDefault();

    const name = document.getElementById("shutterName").value;
    const status_topic = document.getElementById("shutterStatus").value;
    const command_topic = document.getElementById("shutterCommand").value;

    const shutter = {
      name,
      status_topic,
      command_topic,
    };

    const myHeaders = new Headers();
    myHeaders.append("Content-Type", "application/json");
    await fetch("/shutters", {
      method: "POST",
      body: JSON.stringify(shutter),
      headers: myHeaders,
    });
    window.location.reload();

    // shutters.push(newShutter);
    // renderShutters();

    // this.reset();
    // document.getElementById("initialPosition").value = 50;
  });

window.onload = async function () {
  const res = await fetch("/shutters");
  shutters = await res.json();
  renderShutters();
};

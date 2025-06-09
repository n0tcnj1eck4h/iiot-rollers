const myHeaders = new Headers();
myHeaders.append("Content-Type", "application/json");

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
                            onmouseup="updateShutterPosition(${shutter.id}, this.value, true)"
                            onmousedown="ignore.add(${shutter.id})">
                    </div>
                    
                    <div class="preset-buttons">
                        <button class="preset-btn" onclick="updateShutterPosition(${shutter.id}, 0, true)">Zamknij</button>
                        <button class="preset-btn" onclick="updateShutterPosition(${shutter.id}, 100, true)">Otwórz</button>
                    </div>
                    
                `;
    dashboard.appendChild(widget);
    updateShutterVisual(shutter.id, shutter.position);
  });
}

const ignore = new Set();
async function updateShutterPosition(id, position, update) {
  const shutter = shutters.find((s) => s.id === id);
  shutter.position = position;
  // document.querySelector(`#slider-${id}`).value = position;
  if (!update) {
    document.querySelector(
      `#visual-${id}`,
    ).parentNode.nextElementSibling.textContent = `${position}%`;
    updateShutterVisual(id, position);
  }

  if (update) {
    ignore.add(id);
    setTimeout(() => {
      ignore.delete(id);
    }, 5000);

    const req = {
      id: id,
      position: parseInt(position),
    };

    await fetch("/shutters", {
      method: "PUT",
      body: JSON.stringify(req),
      headers: myHeaders,
    });
  }
}

// Set shutter position (with apply button)
async function setShutterPosition(id, position) {
  const shutter = shutters.find((s) => s.id === id);
  if (shutter) {
    shutter.position = parseInt(position);
    renderShutters();
  }
}

async function updateShutterVisual(id, position) {
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
    const height = minHeight +
      (maxHeight - minHeight) * (closedPercentage / 100);
    line.style.height = `${height}px`;

    const opacity = 0.2 + 0.8 * (closedPercentage / 100);
    line.style.opacity = opacity;

    visual.appendChild(line);
  }
}

async function deleteShutter(id) {
  if (confirm("Are you sure you want to remove this shutter?")) {
    await fetch("/shutters", {
      method: "DELETE",
      body: JSON.stringify({ id: id }),
      headers: myHeaders,
    });
    window.location.reload();
    // shutters = shutters.filter((shutter) => shutter.id !== id);
    // renderShutters();
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

  setInterval(async () => {
    const res = await fetch("/shutters");
    shutters = await res.json();
    for (const shutter of shutters) {
      await updateShutterPosition(shutter.id, shutter.position, false);
    }
    // renderShutters();
  }, 100);
};

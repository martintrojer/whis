import {
  showBubble,
  hideBubble,
  isBubbleVisible,
  hasOverlayPermission,
  requestOverlayPermission,
  setBubbleState,
} from "tauri-plugin-floating-bubble";

let bubbleVisible = false;
let currentToggle = document.querySelector("#btn-toggle-bubble") as HTMLButtonElement;

async function updatePermissionStatus() {
  const statusEl = document.querySelector("#permission-status") as HTMLElement;
  const indicatorEl = document.querySelector("#permission-indicator") as HTMLElement;
  const btnEl = document.querySelector("#btn-permission") as HTMLButtonElement;

  try {
    const { granted } = await hasOverlayPermission();
    statusEl.textContent = granted ? "Granted" : "Not granted";
    statusEl.className = `status ${granted ? "granted" : ""}`;
    indicatorEl.className = `indicator ${granted ? "granted" : ""}`;
    btnEl.style.display = granted ? "none" : "block";
  } catch (e) {
    statusEl.textContent = "N/A";
    statusEl.className = "status";
    indicatorEl.className = "indicator";
    btnEl.style.display = "none";
  }
}

async function updateBubbleToggle() {
  try {
    const { visible } = await isBubbleVisible();
    bubbleVisible = visible;
    currentToggle.dataset.state = visible ? "on" : "off";
  } catch (e) {
    bubbleVisible = false;
    currentToggle.dataset.state = "off";
  }
}

async function handleRequestPermission() {
  try {
    await requestOverlayPermission();
    await updatePermissionStatus();
  } catch (e) {
    console.error("Permission request failed:", e);
  }
}

async function handleToggleBubble() {
  try {
    if (bubbleVisible) {
      await hideBubble();
      bubbleVisible = false;
    } else {
      await showBubble({
        size: 60,
        startX: 0,
        startY: 200,
        iconResourceName: "ic_bubble_white",
        background: "#1C1C1C",
        states: {
          white: { iconResourceName: "ic_bubble_white" },
          yellow: { iconResourceName: "ic_bubble_yellow" },
          red: { iconResourceName: "ic_bubble_red" },
        },
      });
      bubbleVisible = true;
    }
    currentToggle.dataset.state = bubbleVisible ? "on" : "off";
  } catch (e) {
    console.error("Toggle bubble failed:", e);
    await updateBubbleToggle();
  }
}

async function handleSetState(state: string) {
  try {
    await setBubbleState(state);
  } catch (e) {
    console.error("Set state failed:", e);
  }
}

window.addEventListener("DOMContentLoaded", async () => {
  await updatePermissionStatus();
  await updateBubbleToggle();

  document.querySelector("#btn-permission")?.addEventListener("click", handleRequestPermission);
  currentToggle?.addEventListener("click", handleToggleBubble);

  const segments = document.querySelectorAll(".segment");
  segments.forEach((seg) => {
    seg.addEventListener("click", () => {
      segments.forEach((s) => s.classList.remove("active"));
      seg.classList.add("active");
      const state = seg.getAttribute("data-state");
      if (state) handleSetState(state);
    });
  });
});

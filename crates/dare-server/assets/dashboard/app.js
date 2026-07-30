(function () {
  "use strict";

  var SECTION_KEYS = ["dag", "gates", "cost", "bestOfN", "guard", "drift"];

  function setStatus(message, isError) {
    var el = document.getElementById("status");
    if (!el) return;
    el.textContent = message;
    if (isError) {
      el.classList.add("is-error");
    } else {
      el.classList.remove("is-error");
    }
  }

  function clearNode(node) {
    while (node.firstChild) {
      node.removeChild(node.firstChild);
    }
  }

  function formatPrimitive(value) {
    if (value === null) return "null";
    if (typeof value === "string") return value;
    if (typeof value === "number" || typeof value === "boolean") {
      return String(value);
    }
    return String(value);
  }

  function appendEmpty(panel, label) {
    var p = document.createElement("p");
    p.className = "empty";
    p.textContent = label;
    panel.appendChild(p);
  }

  function appendValue(container, value, depth) {
    if (value === null || typeof value !== "object") {
      var span = document.createElement("span");
      span.className = "val";
      span.textContent = formatPrimitive(value);
      container.appendChild(span);
      return;
    }

    if (Array.isArray(value)) {
      if (value.length === 0) {
        var emptyArr = document.createElement("span");
        emptyArr.className = "val";
        emptyArr.textContent = "[]";
        container.appendChild(emptyArr);
        return;
      }
      var list = document.createElement("div");
      list.className = depth > 0 ? "nested" : "";
      value.forEach(function (item, index) {
        var row = document.createElement("div");
        row.className = "kv";
        var keyEl = document.createElement("span");
        keyEl.className = "key";
        keyEl.textContent = "[" + String(index) + "]";
        var valWrap = document.createElement("div");
        appendValue(valWrap, item, depth + 1);
        row.appendChild(keyEl);
        row.appendChild(valWrap);
        list.appendChild(row);
      });
      container.appendChild(list);
      return;
    }

    var keys = Object.keys(value);
    if (keys.length === 0) {
      var emptyObj = document.createElement("span");
      emptyObj.className = "val";
      emptyObj.textContent = "{}";
      container.appendChild(emptyObj);
      return;
    }

    var grid = document.createElement("div");
    grid.className = depth > 0 ? "kv nested" : "kv";
    keys.forEach(function (key) {
      var row = document.createElement("div");
      row.className = "kv";
      var keyEl = document.createElement("span");
      keyEl.className = "key";
      keyEl.textContent = key;
      var valWrap = document.createElement("div");
      appendValue(valWrap, value[key], depth + 1);
      row.appendChild(keyEl);
      row.appendChild(valWrap);
      grid.appendChild(row);
    });
    container.appendChild(grid);
  }

  function renderSection(key, data) {
    var panel = document.querySelector('.panel[data-section="' + key + '"]');
    if (!panel) return;
    clearNode(panel);

    if (data === undefined || data === null) {
      appendEmpty(panel, "No data");
      return;
    }

    if (typeof data === "object" && !Array.isArray(data) && Object.keys(data).length === 0) {
      appendEmpty(panel, "Empty");
      return;
    }

    appendValue(panel, data, 0);
  }

  function renderSnapshot(snapshot) {
    var data = snapshot && typeof snapshot === "object" ? snapshot : {};
    SECTION_KEYS.forEach(function (key) {
      renderSection(key, data[key]);
    });
  }

  function loadTelemetry() {
    setStatus("Loading telemetry…", false);

    fetch("/api/telemetry")
      .then(function (response) {
        if (!response.ok) {
          throw new Error("HTTP " + String(response.status) + " " + response.statusText);
        }
        return response.json();
      })
      .then(function (snapshot) {
        renderSnapshot(snapshot);
        setStatus("Telemetry updated", false);
      })
      .catch(function (err) {
        var message =
          err && err.message
            ? "Failed to load telemetry: " + err.message
            : "Failed to load telemetry";
        setStatus(message, true);
        SECTION_KEYS.forEach(function (key) {
          var panel = document.querySelector('.panel[data-section="' + key + '"]');
          if (!panel) return;
          clearNode(panel);
          appendEmpty(panel, "Unavailable");
        });
      });
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", loadTelemetry);
  } else {
    loadTelemetry();
  }
})();

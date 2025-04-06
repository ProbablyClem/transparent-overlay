const invoke = window.__TAURI__.core.invoke;

const div_input = document.querySelector("#input_wrapper");
const url_input = document.querySelector("input#url");
const select_monitor = document.getElementById("monitor");
const launch_button = document.querySelector("button#launch");
const tooltip = document.getElementById("tooltip");
let timeout_id;

select_monitor.addEventListener("change", function () {
  this.blur();
});

url_input.addEventListener("keyup", (e) => {
  if (e.key === "Enter") launch_button.click();
});

launch_button.addEventListener("click", () => {
  invoke("url_is_parsable", { url: url_input.value }).then((result) => {
    if (!result) {
      if (timeout_id) clearTimeout(timeout_id);

      div_input.classList.toggle("input-error", true);
      tooltip.classList.toggle("invisible", false);
      tooltip.classList.toggle("tooltip-open", true);
      timeout_id = setTimeout(() => {
        tooltip.classList.toggle("tooltip-open", false);
      }, 2000);
    } else {
      div_input.classList.toggle("input-error", false);
      tooltip.classList.toggle("invisible", true);
      tooltip.classList.toggle("tooltip-open", false);

      invoke("save_config", {
        config: getConfig(),
      });

      invoke("open_livechat_window", {
        config: getConfig(),
      }).then(() => {
        invoke("close_config_window");
      });
    }
  });
});

function getConfig() {
  return {
    url: url_input.value,
    monitor: JSON.parse(select_monitor.value),
  };
}

url_input.addEventListener("focus", () => {
  tooltip.classList.toggle("tooltip-open", false);
});

window.addEventListener("load", () => {
  invoke("get_config").then((config) => {
    url_input.value = config.url;
  });
  invoke("get_available_monitors").then(async (res) => {
    const monitor_name = (await invoke("get_config"))["monitor"]["name"];
    for (let i = 0; i < res.length; i++) {
      let opt = document.createElement("option");
      opt.value = JSON.stringify(res[i]);
      opt.innerHTML = `&nbsp;Display #${i + 1} - ${res[i].name}`;
      monitor_name == res[i].name ? opt.setAttribute("selected", true) : "";
      select_monitor.appendChild(opt);
    }
  });
});

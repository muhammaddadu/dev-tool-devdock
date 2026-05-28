// DevDock landing — interactive menubar mock.
// Keeps the demo grounded in real product behaviour: search filters by
// label/port/cwd/project; clicking a status dot toggles run state; the
// refresh button reshuffles uptimes; the buckets collapse.

const SERVICES = [
  { label: "web", project: "acme/web", port: 3000, pid: 53211, cmd: "next dev", cwd: "~/work/acme/web", bucket: "dev", status: "running", age: 142 },
  { label: "api", project: "acme/api", port: 5173, pid: 51728, cmd: "cargo run", cwd: "~/work/acme/api", bucket: "dev", status: "running", age: 87 },
  { label: "postgres", project: "acme",  port: 5432, pid: 45321, cmd: "postgres -D data", cwd: "~/work/acme/data", bucket: "dev", status: "running", age: 9821 },
  { label: "redis", project: "acme",     port: 6379, pid: 46211, cmd: "redis-server", cwd: "~/work/acme/data", bucket: "dev", status: "running", age: 9821 },
  { label: "workers", project: "acme/workers", port: 4000, pid: 0, cmd: "node worker.js", cwd: "~/work/acme/workers", bucket: "dev", status: "error", age: 0 },
  { label: "mailpit", project: "tools",  port: 8025, pid: 47110, cmd: "mailpit", cwd: "~/tools/mailpit", bucket: "tooling", status: "running", age: 320 },
  { label: "rust-analyzer", project: "—", port: 0, pid: 47700, cmd: "rust-analyzer", cwd: "—", bucket: "tooling", status: "running", age: 1800 },
  { label: "Docker", project: "—",      port: 0, pid: 22041, cmd: "com.docker.backend", cwd: "—", bucket: "tooling", status: "running", age: 36400 },
  { label: "rapportd", project: "system", port: 0, pid: 612, cmd: "/usr/sbin/rapportd", cwd: "—", bucket: "system", status: "running", age: 96000 },
  { label: "ControlCenter", project: "system", port: 0, pid: 720, cmd: "ControlCenter", cwd: "—", bucket: "system", status: "running", age: 96000 },
];

function formatAge(seconds) {
  if (seconds < 60) return `${seconds}s`;
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m`;
  if (seconds < 86400) return `${Math.floor(seconds / 3600)}h`;
  return `${Math.floor(seconds / 86400)}d`;
}

function statusClass(status) {
  switch (status) {
    case "starting": return "svc--starting";
    case "stopped": return "svc--stopped";
    case "error": return "svc--error";
    default: return "";
  }
}

function svcMatchesQuery(s, q) {
  if (!q) return true;
  const needle = q.toLowerCase();
  return (
    s.label.toLowerCase().includes(needle) ||
    s.cmd.toLowerCase().includes(needle) ||
    s.cwd.toLowerCase().includes(needle) ||
    s.project.toLowerCase().includes(needle) ||
    String(s.port).includes(needle)
  );
}

function render() {
  const mock = document.querySelector("#mock-dropdown");
  if (!mock) return;

  const search = mock.querySelector("input").value.trim();
  const showBackground = mock.dataset.showBackground === "true";

  const visible = SERVICES.filter((s) => svcMatchesQuery(s, search));
  const dev = visible.filter((s) => s.bucket === "dev");
  const tooling = visible.filter((s) => s.bucket === "tooling");
  const system = visible.filter((s) => s.bucket === "system");
  const backgroundCount = tooling.length + system.length;

  const renderRow = (s, idx) => `
    <div class="svc ${statusClass(s.status)}" data-idx="${idx}" role="button" tabindex="0" aria-label="Toggle ${s.label}">
      <span class="svc__dot" aria-hidden="true"></span>
      <span class="svc__meta">
        <span class="svc__label">${s.label}</span>
        <span class="svc__sub">${s.cmd} · ${s.cwd}</span>
      </span>
      <span class="svc__port">${s.port ? ":" + s.port : "—"}</span>
      <span class="svc__age">${s.status === "running" ? formatAge(s.age) : s.status}</span>
    </div>
  `;

  const sectionHtml = (title, rows) => rows.length === 0 ? "" : `
    <div class="mock__bucket">${title} (${rows.length})</div>
    ${rows.map((s) => renderRow(s, SERVICES.indexOf(s))).join("")}
  `;

  mock.querySelector("#mock-rows").innerHTML =
    sectionHtml("Dev", dev) +
    (showBackground ? sectionHtml("Tooling", tooling) + sectionHtml("System", system) : "");

  const toggle = mock.querySelector("#mock-bucket-toggle");
  toggle.querySelector(".label").textContent =
    showBackground ? "Hide background services" : "Show background services";
  toggle.querySelector(".count").textContent = `${backgroundCount}`;

  // Attach handlers
  mock.querySelectorAll(".svc").forEach((node) => {
    const idx = Number(node.dataset.idx);
    node.addEventListener("click", () => {
      const s = SERVICES[idx];
      if (s.status === "running") {
        s.status = "stopped";
        s.age = 0;
      } else if (s.status === "stopped" || s.status === "error") {
        s.status = "starting";
        setTimeout(() => {
          s.status = "running";
          s.age = 1;
          render();
        }, 700);
      }
      render();
    });
    node.addEventListener("keydown", (e) => {
      if (e.key === "Enter" || e.key === " ") {
        e.preventDefault();
        node.click();
      }
    });
  });
}

function initMock() {
  const mock = document.querySelector("#mock-dropdown");
  if (!mock) return;
  mock.dataset.showBackground = "false";

  mock.querySelector("input").addEventListener("input", render);

  mock.querySelector("#mock-refresh").addEventListener("click", () => {
    const btn = mock.querySelector("#mock-refresh svg");
    btn.classList.add("spin");
    SERVICES.forEach((s) => {
      if (s.status === "running") s.age += Math.floor(Math.random() * 30);
    });
    setTimeout(() => {
      btn.classList.remove("spin");
      render();
    }, 600);
  });

  mock.querySelector("#mock-bucket-toggle").addEventListener("click", () => {
    mock.dataset.showBackground = mock.dataset.showBackground === "true" ? "false" : "true";
    render();
  });

  render();
}

document.addEventListener("DOMContentLoaded", initMock);

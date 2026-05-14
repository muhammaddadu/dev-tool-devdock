import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { ServiceCard } from "./ServiceCard";
import type { ServiceView } from "@/lib/types";

const baseService: ServiceView = {
  id: "svc-1",
  label: "Frontend",
  status: "running",
  source: "managed",
  bucket: "dev",
  port: 3000,
  host: "127.0.0.1",
  url: "http://127.0.0.1:3000",
  pid: 12345,
  processName: "node",
  command: "npm run dev",
  cwd: "/Users/me/code/web",
  projectName: "web",
  projectRoot: "/Users/me/code/web",
  startedAtUnix: null,
  savedId: null,
  logPath: null,
  pinned: false,
  canOpen: true,
  canSave: false,
  canRun: false,
  canStop: true,
  canRestart: true,
  canKill: false,
};

describe("ServiceCard", () => {
  it("renders label, port, command, and cwd", () => {
    render(<ServiceCard service={baseService} />);
    expect(screen.getByText("Frontend")).toBeInTheDocument();
    expect(screen.getByText(":3000")).toBeInTheDocument();
    expect(screen.getByText("npm run dev")).toBeInTheDocument();
    expect(screen.getByText("~/code/web")).toBeInTheDocument();
  });

  it("renders Stop/Restart for a running managed service", () => {
    render(<ServiceCard service={baseService} />);
    expect(screen.getByRole("button", { name: "Stop" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Restart" })).toBeInTheDocument();
  });
});

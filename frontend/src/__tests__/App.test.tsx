import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { App } from "../App";
import type { DashboardState } from "../types";

const invokeMock = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (command: string, args?: unknown) => invokeMock(command, args),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => undefined)),
}));

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({ startDragging: vi.fn() }),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: vi.fn(),
}));

const dashboardState: DashboardState = {
  config: {
    version: 2,
    selectedServices: ["kimi", "codex"],
    selectedTools: ["codex_cli"],
    firstRunCompleted: true,
    credentials: { kimiBackend: "keychain" },
    proxy: {
      kimi: {
        mode: "auto",
        proxyUrl: null,
        autoPorts: [7897, 7890],
        timeoutMs: 250,
      },
      codex: {
        mode: "auto",
        proxyUrl: null,
        autoPorts: [7897, 7890],
        timeoutMs: 250,
      },
    },
  },
  tools: [
    {
      id: "codex_cli",
      name: "Codex CLI",
      toolType: "cli",
      installed: true,
      installPath: "/Users/test/.local/bin/codex",
      launchAs: null,
    },
    {
      id: "vscode",
      name: "VS Code",
      toolType: "ide",
      installed: true,
      installPath: "/Applications/Visual Studio Code.app",
      launchAs: "Visual Studio Code",
    },
  ],
  kimiQuota: {
    service: "kimi",
    displayName: "Kimi Code",
    success: true,
    tiers: [
      { name: "five_hour", utilization: 12, resetsAt: "2026-06-04T10:30:00+08:00" },
      { name: "weekly_limit", utilization: 40, resetsAt: "2026-06-07T18:45:00+08:00" },
    ],
    error: null,
    queriedAt: Date.now(),
    credentialValid: true,
  },
  codexQuota: {
    service: "codex",
    displayName: "Codex",
    success: true,
    tiers: [
      { name: "five_hour", utilization: 91, resetsAt: null },
      { name: "seven_day", utilization: 100, resetsAt: null },
    ],
    error: null,
    queriedAt: Date.now(),
    credentialValid: true,
  },
  kimiEstimates: [
    {
      tier: "weekly_limit",
      estimate: { state: "enough", projectedUtilization: 72 },
    },
  ],
  codexEstimates: [
    {
      tier: "seven_day",
      estimate: {
        state: "not_enough",
        projectedUtilization: 188,
        lastsForSecs: 93_600,
      },
    },
  ],
  proxyStatus: {
    kimi: { status: "direct", proxyUrl: null, message: "Direct" },
    codex: {
      status: "proxy",
      proxyUrl: "http://127.0.0.1:7897",
      message: "Proxy",
    },
  },
};

describe("App", () => {
  beforeEach(() => {
    invokeMock.mockReset();
    invokeMock.mockImplementation((command: string) => {
      if (command === "get_dashboard_state") return Promise.resolve(dashboardState);
      if (command === "set_selected_tools") return Promise.resolve(dashboardState);
      if (command === "save_proxy_settings") return Promise.resolve(dashboardState);
      return Promise.resolve(dashboardState);
    });
  });

  it("renders Kimi and Codex quota cards", async () => {
    render(<App />);

    expect(await screen.findByText("Kimi Code")).toBeInTheDocument();
    expect(screen.getByText("Codex")).toBeInTheDocument();
    expect(screen.getByText("不够")).toBeInTheDocument();
    expect(screen.getByText(/06月04日 .* 重置/)).toBeInTheDocument();
    expect(screen.getByText(/06月07日 .* 重置/)).toBeInTheDocument();
    expect(screen.getByText("Kimi 当前直连")).toBeInTheDocument();
    expect(screen.getByText("Codex 代理已连接")).toBeInTheDocument();
    expect(screen.getByText("本周内预计够用。")).toBeInTheDocument();
    expect(screen.getByText("预计将在 1 天 2 小时 后耗尽。")).toBeInTheDocument();
    expect(screen.queryByText("direct")).not.toBeInTheDocument();
    expect(screen.queryByText("unavailable")).not.toBeInTheDocument();
  });

  it("shows selected and available tools", async () => {
    const user = userEvent.setup();
    render(<App />);

    await user.click(await screen.findByRole("button", { name: "工具" }));
    await user.click(screen.getByRole("button", { name: /工具选择/ }));

    expect(screen.getAllByText("Codex CLI").length).toBeGreaterThan(0);
    expect(screen.getByText("VS Code")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "添加" }));

    await waitFor(() =>
      expect(invokeMock).toHaveBeenCalledWith("set_selected_tools", {
        toolIds: ["codex_cli", "vscode"],
      }),
    );
  });

  it("saves proxy settings from settings tab", async () => {
    const user = userEvent.setup();
    render(<App />);

    await user.click(await screen.findByRole("button", { name: "设置" }));
    await user.click(screen.getByRole("button", { name: "保存代理设置" }));

    await waitFor(() =>
      expect(invokeMock).toHaveBeenCalledWith(
        "save_proxy_settings",
        expect.objectContaining({ settings: dashboardState.config.proxy }),
      ),
    );
  });
});

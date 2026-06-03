import { AlertTriangle, CheckCircle2, Clock, Network } from "lucide-react";
import type {
  ProxyTestResult,
  ServiceQuota,
  SufficiencyState,
  TierEstimateView,
} from "../types";
import { proxyDetailLabel } from "../proxyDisplay";

interface QuotaCardProps {
  title: string;
  iconSrc: string;
  quota?: ServiceQuota | null;
  estimates: TierEstimateView[];
  proxy: ProxyTestResult;
}

const tierLabels: Record<string, string> = {
  five_hour: "5 小时",
  weekly_limit: "7 天",
  seven_day: "7 天",
};

export function QuotaCard({ title, iconSrc, quota, estimates, proxy }: QuotaCardProps) {
  const weekly = quota?.tiers.find((tier) =>
    ["weekly_limit", "seven_day"].includes(tier.name),
  );
  const weeklyEstimate = estimates.find(
    (entry) => entry.tier === weekly?.name,
  )?.estimate;
  const displayState = quota?.success
    ? stateLabel(weeklyEstimate?.state ?? "unknown")
    : quota
      ? "未配置"
      : "等待刷新";

  return (
    <section className="quota-card">
      <div className="quota-header">
        <div className="service-heading">
          <img src={iconSrc} alt="" aria-hidden />
          <div>
            <p className="eyebrow">{title}</p>
            <h3>{displayState}</h3>
          </div>
        </div>
        {quota?.success ? (
          <CheckCircle2 size={18} aria-hidden className="ok" />
        ) : (
          <AlertTriangle size={18} aria-hidden className="warn" />
        )}
      </div>

      {!quota && <p className="muted quota-empty">等待后台首次刷新用量。</p>}
      {quota && !quota.success && (
        <p className="error-copy">{quota.error ?? "用量查询失败"}</p>
      )}
      {quota?.success && (
        <div className="tier-stack">
          {quota.tiers.map((tier) => (
            <div className="tier-row" key={tier.name}>
              <div className="tier-meta">
                <span className="tier-label">
                  <span>{tierLabels[tier.name] ?? tier.name}</span>
                  {tier.resetsAt && (
                    <small>（{formatResetLabel(tier.name, tier.resetsAt)}）</small>
                  )}
                </span>
                <strong>{Math.round(tier.utilization)}%</strong>
              </div>
              <div className="meter" aria-label={`${tier.name} utilization`}>
                <div
                  className={meterClass(tier.utilization)}
                  style={{ width: `${Math.min(tier.utilization, 100)}%` }}
                />
              </div>
            </div>
          ))}
        </div>
      )}

      {quota?.success && weeklyEstimate && (
        <div className="estimate-box">
          <div>
            <span>预计用量</span>
            <strong>
              {weeklyEstimate.projectedUtilization == null
                ? "未知"
                : `${Math.round(weeklyEstimate.projectedUtilization)}%`}
            </strong>
          </div>
          {weeklyEstimate.exhaustedBeforeResetSecs != null && (
            <p>
              已提前 {formatDuration(weeklyEstimate.exhaustedBeforeResetSecs)} 耗尽。
            </p>
          )}
          {weeklyEstimate.exhaustedBeforeResetSecs == null && (
            <p>{estimateHint(weeklyEstimate.state, weeklyEstimate.lastsForSecs)}</p>
          )}
        </div>
      )}

      <div className="proxy-line">
        <Network size={13} aria-hidden />
        <span>{proxyDetailLabel(proxy)}</span>
      </div>
      {quota?.queriedAt && (
        <div className="proxy-line">
          <Clock size={13} aria-hidden />
          <span>更新于 {new Date(quota.queriedAt).toLocaleTimeString()}</span>
        </div>
      )}
    </section>
  );
}

function meterClass(utilization: number): string {
  if (utilization >= 90) return "meter-fill danger";
  if (utilization >= 70) return "meter-fill warn";
  return "meter-fill ok-fill";
}

function stateLabel(state: SufficiencyState): string {
  if (state === "enough") return "够";
  if (state === "tight") return "偏紧";
  if (state === "not_enough") return "不够";
  return "未知";
}

function estimateHint(state: SufficiencyState, lastsForSecs?: number | null): string {
  if (state === "not_enough" && lastsForSecs != null) {
    return `预计将在 ${formatDuration(lastsForSecs)} 后耗尽。`;
  }
  if (state === "tight") {
    return "本周内预计不会耗尽，但余量偏紧。";
  }
  if (state === "enough") {
    return "本周内预计够用。";
  }
  return "等待更多用量数据后估算。";
}

function formatDuration(seconds: number): string {
  const days = Math.floor(seconds / 86400);
  const hours = Math.floor((seconds % 86400) / 3600);
  if (days > 0 && hours > 0) return `${days} 天 ${hours} 小时`;
  if (days > 0) return `${days} 天`;
  return `${hours} 小时`;
}

function formatResetTime(value: string): string {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return "未知时间";
  const month = `${date.getMonth() + 1}`.padStart(2, "0");
  const day = `${date.getDate()}`.padStart(2, "0");
  const hour = `${date.getHours()}`.padStart(2, "0");
  const minute = `${date.getMinutes()}`.padStart(2, "0");
  return `${month}月${day}日 ${hour}:${minute}`;
}

function formatResetLabel(tierName: string, value: string): string {
  if (tierName === "five_hour") {
    return `${formatResetCountdown(value)}后重置`;
  }
  return `${formatResetTime(value)} 重置`;
}

function formatResetCountdown(value: string): string {
  const resetAt = new Date(value).getTime();
  if (Number.isNaN(resetAt)) return "未知时间";
  const seconds = Math.max(0, Math.ceil((resetAt - Date.now()) / 1000));
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.ceil((seconds % 3600) / 60);
  if (hours > 0 && minutes > 0) return `${hours} 小时 ${minutes} 分钟`;
  if (hours > 0) return `${hours} 小时`;
  return `${minutes} 分钟`;
}

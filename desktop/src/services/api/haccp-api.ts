import type {
  HaccpMonitoringRecord,
  HaccpPlanDocument,
  HaccpPlanSummary,
} from "../../domain/types";
import { hasConfiguredService, serviceRequest } from "../transport/websocket-client";

const browserPlans = new Map<string, HaccpPlanDocument[]>();

export async function listHaccpPlans(recipeId: string): Promise<HaccpPlanSummary[]> {
  if (hasConfiguredService())
    return serviceRequest(`/api/v1/recipes/${encodeURIComponent(recipeId)}/haccp`);
  return (browserPlans.get(recipeId) ?? []).map((plan) => ({
    id: plan.id,
    recipeId: plan.recipeId,
    title: plan.title,
    description: plan.description,
    status: plan.status,
    hazardCount: plan.hazards.length,
    ccpCount: plan.ccps.length,
    updatedAt: plan.updatedAt,
  }));
}

export async function getHaccpPlan(planId: string): Promise<HaccpPlanDocument> {
  if (hasConfiguredService()) return serviceRequest(`/api/v1/haccp/${encodeURIComponent(planId)}`);
  for (const plans of browserPlans.values()) {
    const plan = plans.find((item) => item.id === planId);
    if (plan) return structuredClone(plan);
  }
  throw new Error("HACCP plan not found");
}

export async function createHaccpPlan(
  recipeId: string,
  title: string,
  description?: string,
): Promise<HaccpPlanDocument> {
  if (hasConfiguredService())
    return serviceRequest(`/api/v1/recipes/${encodeURIComponent(recipeId)}/haccp`, {
      method: "POST",
      body: JSON.stringify({ title, description }),
    });
  const plan: HaccpPlanDocument = {
    id: crypto.randomUUID(),
    recipeId,
    title,
    description,
    status: "draft",
    hazards: [],
    ccps: [],
    monitoringRecords: [],
    updatedAt: new Date().toISOString(),
  };
  const plans = browserPlans.get(recipeId) ?? [];
  plans.push(plan);
  browserPlans.set(recipeId, plans);
  return structuredClone(plan);
}

export async function saveHaccpPlan(plan: HaccpPlanDocument): Promise<HaccpPlanDocument> {
  if (hasConfiguredService())
    return serviceRequest(`/api/v1/haccp/${encodeURIComponent(plan.id)}`, {
      method: "PUT",
      body: JSON.stringify({
        title: plan.title,
        description: plan.description,
        status: plan.status,
        hazards: plan.hazards,
        ccps: plan.ccps,
      }),
    });
  const plans = browserPlans.get(plan.recipeId) ?? [];
  const index = plans.findIndex((item) => item.id === plan.id);
  if (index < 0) throw new Error("HACCP plan not found");
  const saved = { ...structuredClone(plan), updatedAt: new Date().toISOString() };
  plans[index] = saved;
  browserPlans.set(plan.recipeId, plans);
  return saved;
}

export async function deleteHaccpPlan(planId: string): Promise<void> {
  if (hasConfiguredService())
    return serviceRequest(`/api/v1/haccp/${encodeURIComponent(planId)}`, { method: "DELETE" });
  for (const [recipeId, plans] of browserPlans.entries()) {
    const next = plans.filter((plan) => plan.id !== planId);
    if (next.length !== plans.length) {
      browserPlans.set(recipeId, next);
      return;
    }
  }
}

export async function recordHaccpMonitoring(
  ccpId: string,
  input: {
    measuredValue: string;
    withinLimit: boolean;
    correctiveActionTaken?: string;
    recordedBy?: string;
    notes?: string;
  },
): Promise<HaccpMonitoringRecord> {
  if (hasConfiguredService())
    return serviceRequest(`/api/v1/haccp/ccps/${encodeURIComponent(ccpId)}/records`, {
      method: "POST",
      body: JSON.stringify(input),
    });
  for (const plans of browserPlans.values()) {
    const ccp = plans.flatMap((plan) => plan.ccps).find((item) => item.id === ccpId);
    if (!ccp) continue;
    const record: HaccpMonitoringRecord = {
      id: crypto.randomUUID(),
      ccpId,
      recordedAt: new Date().toISOString(),
      measuredValue: input.measuredValue,
      withinLimit: input.withinLimit,
      correctiveActionTaken: input.correctiveActionTaken,
      recordedBy: input.recordedBy,
      notes: input.notes,
    };
    const plan = plans.find((item) => item.ccps.some((candidate) => candidate.id === ccpId));
    if (plan) plan.monitoringRecords.unshift(record);
    return record;
  }
  throw new Error("CCP not found");
}

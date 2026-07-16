<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from "vue";
import { AlertTriangle, ClipboardCheck, Plus, Save, ShieldCheck, Trash2 } from "lucide-vue-next";
import type {
  HaccpCcp,
  HaccpHazard,
  HaccpPlanDocument,
  HaccpPlanSummary,
} from "../../../domain/types";
import * as api from "../../../services/api";

const props = defineProps<{
  recipeId: string;
  operationSymbols?: string[];
}>();

const plans = ref<HaccpPlanSummary[]>([]);
const selectedPlanId = ref<string | null>(null);
const plan = reactive<HaccpPlanDocument>(emptyPlan(props.recipeId));
const error = ref("");
const saving = ref(false);
const recordCcpId = ref<string | null>(null);
const recordValue = ref("");
const recordWithinLimit = ref(true);
const recordNotes = ref("");

function emptyPlan(recipeId: string): HaccpPlanDocument {
  return {
    id: "",
    recipeId,
    title: "New HACCP plan",
    description: "",
    status: "draft",
    hazards: [],
    ccps: [],
    monitoringRecords: [],
    updatedAt: "",
  };
}

function newHazard(): HaccpHazard {
  return {
    id: crypto.randomUUID(),
    position: plan.hazards.length,
    hazardType: "biological",
    description: "",
    severity: "medium",
    likelihood: "possible",
    preventiveMeasures: "",
    isCcp: false,
  };
}

function newCcp(): HaccpCcp {
  return {
    id: crypto.randomUUID(),
    position: plan.ccps.length,
    name: "",
    criticalLimit: "",
    monitoringProcedure: "",
    monitoringFrequency: "",
    correctiveAction: "",
    verificationProcedure: "",
    responsibleParty: "",
  };
}

const selectedPlanSummary = computed(() =>
  plans.value.find((item) => item.id === selectedPlanId.value),
);

const recentRecords = computed(() =>
  [...plan.monitoringRecords].sort((left, right) =>
    right.recordedAt.localeCompare(left.recordedAt),
  ),
);

async function refreshPlans(): Promise<void> {
  plans.value = await api.listHaccpPlans(props.recipeId);
  if (!selectedPlanId.value && plans.value[0]) {
    await selectPlan(plans.value[0].id);
  }
}

async function selectPlan(planId: string): Promise<void> {
  selectedPlanId.value = planId;
  const loaded = await api.getHaccpPlan(planId);
  Object.assign(plan, loaded);
}

async function createPlan(): Promise<void> {
  try {
    error.value = "";
    const created = await api.createHaccpPlan(props.recipeId, "HACCP plan");
    await refreshPlans();
    await selectPlan(created.id);
  } catch (cause) {
    error.value = cause instanceof Error ? cause.message : String(cause);
  }
}

async function savePlan(): Promise<void> {
  if (!plan.id) return;
  try {
    saving.value = true;
    error.value = "";
    plan.hazards.forEach((hazard, index) => {
      hazard.position = index;
    });
    plan.ccps.forEach((ccp, index) => {
      ccp.position = index;
    });
    const saved = await api.saveHaccpPlan(plan);
    Object.assign(plan, saved);
    await refreshPlans();
  } catch (cause) {
    error.value = cause instanceof Error ? cause.message : String(cause);
  } finally {
    saving.value = false;
  }
}

async function deletePlan(): Promise<void> {
  if (!plan.id) return;
  try {
    await api.deleteHaccpPlan(plan.id);
    selectedPlanId.value = null;
    Object.assign(plan, emptyPlan(props.recipeId));
    await refreshPlans();
  } catch (cause) {
    error.value = cause instanceof Error ? cause.message : String(cause);
  }
}

async function submitRecord(): Promise<void> {
  if (!recordCcpId.value || !recordValue.value.trim()) return;
  try {
    const record = await api.recordHaccpMonitoring(recordCcpId.value, {
      measuredValue: recordValue.value,
      withinLimit: recordWithinLimit.value,
      notes: recordNotes.value || undefined,
    });
    plan.monitoringRecords.unshift(record);
    recordValue.value = "";
    recordNotes.value = "";
    recordWithinLimit.value = true;
  } catch (cause) {
    error.value = cause instanceof Error ? cause.message : String(cause);
  }
}

function ccpName(ccpId: string): string {
  return plan.ccps.find((item) => item.id === ccpId)?.name ?? ccpId;
}

onMounted(refreshPlans);
watch(
  () => props.recipeId,
  async () => {
    selectedPlanId.value = null;
    Object.assign(plan, emptyPlan(props.recipeId));
    await refreshPlans();
  },
);
</script>

<template>
  <section class="panel haccp-panel">
    <header class="panel-header">
      <div>
        <h3><ShieldCheck :size="17" /> HACCP management</h3>
        <small>Hazard analysis, critical control points, and monitoring logs.</small>
      </div>
      <div class="haccp-actions">
        <button @click="createPlan"><Plus :size="15" /> New plan</button>
        <button class="primary" :disabled="!plan.id || saving" @click="savePlan">
          <Save :size="15" /> Save
        </button>
      </div>
    </header>

    <p class="haccp-disclaimer">
      <AlertTriangle :size="14" />
      Software guidance does not replace professional food-safety review or regulatory compliance.
    </p>

    <p v-if="error" class="error">{{ error }}</p>

    <label v-if="plans.length">
      Plan
      <select :value="selectedPlanId ?? ''" @change="selectPlan(($event.target as HTMLSelectElement).value)">
        <option disabled value="">Select a plan</option>
        <option v-for="item in plans" :key="item.id" :value="item.id">
          {{ item.title }} ({{ item.status }})
        </option>
      </select>
    </label>

    <template v-if="plan.id">
      <label>Title<input v-model="plan.title" /></label>
      <label>Description<textarea v-model="plan.description" rows="2" /></label>
      <label>
        Status
        <select v-model="plan.status">
          <option value="draft">Draft</option>
          <option value="active">Active</option>
          <option value="archived">Archived</option>
        </select>
      </label>

      <section class="haccp-section">
        <header>
          <h4>Hazard analysis</h4>
          <button @click="plan.hazards.push(newHazard())"><Plus :size="14" /> Add hazard</button>
        </header>
        <article v-for="(hazard, index) in plan.hazards" :key="hazard.id" class="card">
          <div class="card-header">
            <strong>Hazard {{ index + 1 }}</strong>
            <button @click="plan.hazards.splice(index, 1)"><Trash2 :size="14" /></button>
          </div>
          <label>
            Type
            <select v-model="hazard.hazardType">
              <option value="biological">Biological</option>
              <option value="chemical">Chemical</option>
              <option value="physical">Physical</option>
            </select>
          </label>
          <label>Description<input v-model="hazard.description" /></label>
          <div class="inline-fields">
            <label>
              Severity
              <select v-model="hazard.severity">
                <option value="low">Low</option>
                <option value="medium">Medium</option>
                <option value="high">High</option>
                <option value="critical">Critical</option>
              </select>
            </label>
            <label>
              Likelihood
              <select v-model="hazard.likelihood">
                <option value="unlikely">Unlikely</option>
                <option value="possible">Possible</option>
                <option value="likely">Likely</option>
                <option value="certain">Certain</option>
              </select>
            </label>
          </div>
          <label>Preventive measures<textarea v-model="hazard.preventiveMeasures" rows="2" /></label>
          <label class="checkbox">
            <input v-model="hazard.isCcp" type="checkbox" />
            Requires a critical control point
          </label>
        </article>
        <p v-if="!plan.hazards.length" class="empty">No hazards documented yet.</p>
      </section>

      <section class="haccp-section">
        <header>
          <h4>Critical control points</h4>
          <button @click="plan.ccps.push(newCcp())"><Plus :size="14" /> Add CCP</button>
        </header>
        <article v-for="(ccp, index) in plan.ccps" :key="ccp.id" class="card">
          <div class="card-header">
            <strong>CCP {{ index + 1 }}</strong>
            <button @click="plan.ccps.splice(index, 1)"><Trash2 :size="14" /></button>
          </div>
          <label>Name<input v-model="ccp.name" /></label>
          <label>
            Linked operation
            <select v-model="ccp.operationSymbol">
              <option :value="null">None</option>
              <option v-for="symbol in operationSymbols ?? []" :key="symbol" :value="symbol">
                {{ symbol }}
              </option>
            </select>
          </label>
          <label>Critical limit<input v-model="ccp.criticalLimit" placeholder="e.g. internal temp >= 74 C" /></label>
          <label>Monitoring procedure<textarea v-model="ccp.monitoringProcedure" rows="2" /></label>
          <label>Monitoring frequency<input v-model="ccp.monitoringFrequency" placeholder="e.g. every 30 min" /></label>
          <label>Corrective action<textarea v-model="ccp.correctiveAction" rows="2" /></label>
          <label>Verification<textarea v-model="ccp.verificationProcedure" rows="2" /></label>
          <label>Responsible party<input v-model="ccp.responsibleParty" /></label>
        </article>
        <p v-if="!plan.ccps.length" class="empty">No critical control points defined.</p>
      </section>

      <section class="haccp-section">
        <header>
          <h4><ClipboardCheck :size="15" /> Monitoring log</h4>
        </header>
        <div v-if="plan.ccps.length" class="record-form">
          <label>
            CCP
            <select v-model="recordCcpId">
              <option :value="null">Select CCP</option>
              <option v-for="ccp in plan.ccps" :key="ccp.id" :value="ccp.id">{{ ccp.name }}</option>
            </select>
          </label>
          <label>Measured value<input v-model="recordValue" placeholder="e.g. 76 C" /></label>
          <label class="checkbox">
            <input v-model="recordWithinLimit" type="checkbox" />
            Within critical limit
          </label>
          <label>Notes<textarea v-model="recordNotes" rows="2" /></label>
          <button class="primary" @click="submitRecord">Record observation</button>
        </div>
        <article
          v-for="record in recentRecords.slice(0, 10)"
          :key="record.id"
          class="card"
          :class="{ 'out-of-limit': !record.withinLimit }"
        >
          <strong>{{ ccpName(record.ccpId) }}</strong>
          <small>{{ record.recordedAt }}</small>
          <span>{{ record.measuredValue }} · {{ record.withinLimit ? "within limit" : "out of limit" }}</span>
          <p v-if="record.notes">{{ record.notes }}</p>
        </article>
        <p v-if="!recentRecords.length" class="empty">No monitoring records yet.</p>
      </section>

      <footer class="haccp-footer">
        <small v-if="selectedPlanSummary">
          {{ selectedPlanSummary.hazardCount }} hazards · {{ selectedPlanSummary.ccpCount }} CCPs
        </small>
        <button class="danger" @click="deletePlan"><Trash2 :size="14" /> Delete plan</button>
      </footer>
    </template>

    <p v-else class="empty">Create a HACCP plan to document hazards and control points for this recipe.</p>
  </section>
</template>

<style scoped>
.haccp-panel {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}
.haccp-disclaimer {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  font-size: 0.8rem;
  color: var(--text-muted, #6b7280);
}
.haccp-actions {
  display: flex;
  gap: 0.5rem;
}
.haccp-section {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}
.haccp-section header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.inline-fields {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0.5rem;
}
.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.record-form {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}
.out-of-limit {
  border-color: #dc2626;
}
.haccp-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.checkbox {
  display: flex;
  align-items: center;
  gap: 0.4rem;
}
.error {
  color: #dc2626;
}
</style>

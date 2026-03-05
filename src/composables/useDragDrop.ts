import { ref, readonly } from "vue";

export interface DragEntity {
  id: string;
  name: string;
  kind: string;
  slug: string;
  uuid?: string;
}

export interface DragPrice {
  entityId: string;
  entityName: string;
  entityKind: string;
  entitySlug: string;
  terminalId: string;
  terminalName: string;
  priceType: string;
}

export type DragPayload =
  | { type: "entity"; data: DragEntity }
  | { type: "price"; data: DragPrice };

const payload = ref<DragPayload | null>(null);
const dragging = ref(false);
const ghostX = ref(0);
const ghostY = ref(0);
const ghostLabel = ref("");

const DRAG_THRESHOLD = 6; // px before drag starts

let startX = 0;
let startY = 0;
let pendingPayload: DragPayload | null = null;
let pendingLabel = "";

function onPointerMove(e: PointerEvent) {
  if (pendingPayload && !dragging.value) {
    const dx = e.clientX - startX;
    const dy = e.clientY - startY;
    if (Math.abs(dx) + Math.abs(dy) >= DRAG_THRESHOLD) {
      dragging.value = true;
      payload.value = pendingPayload;
      ghostLabel.value = pendingLabel;
    }
  }
  if (dragging.value) {
    ghostX.value = e.clientX;
    ghostY.value = e.clientY;
  }
}

function onPointerUp() {
  queueMicrotask(() => {
    dragging.value = false;
    payload.value = null;
    ghostLabel.value = "";
    pendingPayload = null;
  });
  document.removeEventListener("pointermove", onPointerMove);
  document.removeEventListener("pointerup", onPointerUp);
}

function beginDrag(e: PointerEvent, p: DragPayload, label: string) {
  pendingPayload = p;
  pendingLabel = label;
  startX = e.clientX;
  startY = e.clientY;
  document.addEventListener("pointermove", onPointerMove);
  document.addEventListener("pointerup", onPointerUp);
}

/** Drag a search result entity (for favorites). */
export function startDrag(e: PointerEvent, entity: DragEntity) {
  beginDrag(e, { type: "entity", data: entity }, entity.name);
}

/** Drag a price entry (for watch list). */
export function startPriceDrag(e: PointerEvent, price: DragPrice) {
  beginDrag(e, { type: "price", data: price }, `${price.entityName} @ ${price.terminalName}`);
}

export function useDragDrop() {
  return {
    payload: readonly(payload),
    dragging: readonly(dragging),
    ghostX: readonly(ghostX),
    ghostY: readonly(ghostY),
    ghostLabel: readonly(ghostLabel),
  };
}

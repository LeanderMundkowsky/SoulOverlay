import { ref, readonly } from "vue";

export interface DragEntity {
  id: string;
  name: string;
  kind: string;
  slug: string;
  uuid?: string;
}

const payload = ref<DragEntity | null>(null);
const dragging = ref(false);
const ghostX = ref(0);
const ghostY = ref(0);

const DRAG_THRESHOLD = 6; // px before drag starts

let startX = 0;
let startY = 0;
let pending: DragEntity | null = null;

function onPointerMove(e: PointerEvent) {
  if (pending && !dragging.value) {
    const dx = e.clientX - startX;
    const dy = e.clientY - startY;
    if (Math.abs(dx) + Math.abs(dy) >= DRAG_THRESHOLD) {
      dragging.value = true;
      payload.value = pending;
    }
  }
  if (dragging.value) {
    ghostX.value = e.clientX;
    ghostY.value = e.clientY;
  }
}

function onPointerUp() {
  // Drop is handled by the target's pointerup; this just cleans up.
  // Use a microtask so the drop target's pointerup fires first.
  queueMicrotask(() => {
    dragging.value = false;
    payload.value = null;
    pending = null;
  });
  document.removeEventListener("pointermove", onPointerMove);
  document.removeEventListener("pointerup", onPointerUp);
}

/** Call from a search-result row's pointerdown handler. */
export function startDrag(e: PointerEvent, entity: DragEntity) {
  pending = entity;
  startX = e.clientX;
  startY = e.clientY;
  document.addEventListener("pointermove", onPointerMove);
  document.addEventListener("pointerup", onPointerUp);
}

export function useDragDrop() {
  return {
    payload: readonly(payload),
    dragging: readonly(dragging),
    ghostX: readonly(ghostX),
    ghostY: readonly(ghostY),
  };
}

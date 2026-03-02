<script setup lang="ts">
const props = defineProps<{
  defaultPx: number;
  side?: "left" | "right";
}>();

const emit = defineEmits<{
  (e: "resize", newPx: number): void;
  (e: "reset"): void;
  (e: "drag-start"): void;
  (e: "drag-end"): void;
}>();

function onMouseDown(startEvent: MouseEvent) {
  startEvent.preventDefault();
  const startX = startEvent.clientX;
  const parent = (startEvent.currentTarget as HTMLElement).parentElement;
  if (!parent) return;
  const startWidth = parent.getBoundingClientRect().width;

  emit("drag-start");
  document.body.style.cursor = "ew-resize";
  document.body.style.userSelect = "none";

  function onMouseMove(e: MouseEvent) {
    const delta = e.clientX - startX;
    const newPx = props.side === "left" ? startWidth - delta : startWidth + delta;
    emit("resize", Math.round(newPx));
  }

  function onMouseUp() {
    emit("drag-end");
    document.body.style.cursor = "";
    document.body.style.userSelect = "";
    window.removeEventListener("mousemove", onMouseMove);
    window.removeEventListener("mouseup", onMouseUp);
  }

  window.addEventListener("mousemove", onMouseMove);
  window.addEventListener("mouseup", onMouseUp);
}

function onDblClick() {
  emit("reset");
}
</script>

<template>
  <div
    class="absolute top-0 h-full w-1 cursor-ew-resize z-10 hover:bg-white/20 transition-colors"
    :class="side === 'left' ? 'left-0' : 'right-0'"
    @mousedown="onMouseDown"
    @dblclick="onDblClick"
  />
</template>
